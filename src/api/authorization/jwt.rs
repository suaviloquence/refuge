use std::{
	future::{ready, Future, Ready},
	pin::Pin,
	rc::Rc,
};

use actix_web::{
	dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
	error::{ErrorBadRequest, ErrorUnauthorized},
	http::header::AUTHORIZATION,
	web, HttpMessage,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{
	api::IntoHttpError,
	db::{authorization::Authentication, Executor, Pool},
};

// keys have no impl for Debug
pub struct JwtConfig {
	encoder: EncodingKey,
	decoder: DecodingKey,
	header: Header,
	validation: Validation,
}

impl JwtConfig {
	pub fn new(encoder: EncodingKey, decoder: DecodingKey, algorithm: Algorithm) -> Self {
		Self {
			encoder,
			decoder,
			header: Header {
				alg: algorithm.clone(),
				..Default::default()
			},
			validation: Validation::new(algorithm),
		}
	}

	#[inline]
	pub fn encode(&self, claims: &AuthClaims) -> jsonwebtoken::errors::Result<String> {
		jsonwebtoken::encode(&self.header, claims, &self.encoder)
	}

	#[inline]
	pub fn decode(&self, token: &str) -> jsonwebtoken::errors::Result<AuthClaims> {
		jsonwebtoken::decode(token, &self.decoder, &self.validation).map(|x| x.claims)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
	username: String,
	version: u32,
	#[serde(with = "chrono::serde::ts_seconds")]
	exp: DateTime<Utc>,
}

impl AuthClaims {
	#[inline]
	pub fn username(&self) -> &str {
		&self.username
	}

	#[inline]
	pub fn create(authentication: Authentication) -> Self {
		Self {
			username: authentication.username,
			version: authentication.version,
			exp: Utc::now() + Duration::days(7),
		}
	}

	#[inline]
	pub async fn verify(&self, con: impl Executor<'_>) -> sqlx::Result<bool> {
		let row = sqlx::query!(
			r#"SELECT version as "version: u32" FROM authentication WHERE username = $1"#,
			self.username
		)
		.fetch_optional(con)
		.await?;

		Ok(row.map(|x| x.version >= self.version).unwrap_or_default())
	}
}

pub struct Authenticated;

impl<S, B> Transform<S, ServiceRequest> for Authenticated
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
	S::Future: 'static,
	B: 'static,
{
	type Response = ServiceResponse<B>;
	type Error = actix_web::Error;
	type InitError = ();
	type Transform = AuthenticatedMiddleware<S>;
	type Future = Ready<Result<Self::Transform, Self::InitError>>;

	fn new_transform(&self, service: S) -> Self::Future {
		ready(Ok(AuthenticatedMiddleware {
			service: Rc::new(service),
		}))
	}
}

pub struct AuthenticatedMiddleware<S> {
	service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticatedMiddleware<S>
where
	S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
	S::Future: 'static,
	B: 'static,
{
	type Response = ServiceResponse<B>;
	type Error = actix_web::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + 'static>>;

	forward_ready!(service);

	fn call(&self, req: ServiceRequest) -> Self::Future {
		let srv = Rc::clone(&self.service);

		Box::pin(async move {
			let con = req.app_data::<web::Data<Pool>>().unwrap();
			let cfg = req.app_data::<web::Data<JwtConfig>>().unwrap();

			let token = req
				.headers()
				.get(AUTHORIZATION)
				.ok_or_else(|| ErrorBadRequest("missing Authorization header"))?
				.to_str()
				.into_400()?
				.strip_prefix("Bearer ")
				.ok_or_else(|| ErrorBadRequest("bad Authorization header"))?;

			let claims = cfg
				.decode(token)
				.map_err(|_| ErrorUnauthorized("invalid JWT"))?;

			if !claims.verify(con.as_ref()).await.into_500()? {
				return Err(ErrorUnauthorized("invalid token"));
			}

			req.extensions_mut().insert(claims);

			srv.call(req).await
		})
	}
}
