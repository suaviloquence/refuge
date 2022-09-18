use actix_web::{
	error::{ErrorBadRequest, ErrorUnauthorized},
	web::{self, ServiceConfig},
	HttpResponse, Responder,
};

use serde::Deserialize;

use crate::db::{
	authorization::{Authentication, User},
	Pool,
};

use self::jwt::{AuthClaims, JwtConfig};

use super::IntoHttpError;

pub mod jwt;

#[derive(Debug, Deserialize)]
struct Login {
	username: String,
	password: String,
}

async fn login(
	con: web::Data<Pool>,
	argon: web::Data<argon2::Config<'static>>,
	jwt: web::Data<JwtConfig>,
	data: web::Json<Login>,
) -> impl Responder {
	if data.username.is_empty() || data.password.is_empty() {
		return Err(ErrorBadRequest("username/password cannot be empty"));
	}

	let auth = Authentication::load(&data.username, con.as_ref())
		.await
		.into_500()?
		.ok_or_else(|| "user not found")
		.into_404()?;

	if auth.verify(&data.password, &argon).into_500()? {
		Ok(HttpResponse::Ok().body(jwt.encode(&AuthClaims::create(auth)).into_500()?))
	} else {
		Err(ErrorUnauthorized("invalid credentials"))
	}
}

async fn signup(
	con: web::Data<Pool>,
	argon: web::Data<argon2::Config<'static>>,
	jwt: web::Data<JwtConfig>,
	data: web::Json<Login>,
) -> impl Responder {
	if data.username.is_empty() || data.password.is_empty() {
		return Err(ErrorBadRequest("username/password cannot be empty"));
	}

	let exists = User::load(&data.username, con.as_ref())
		.await
		.into_500()?
		.is_some();

	if exists {
		return Err(ErrorBadRequest("username already in use"));
	}

	let user = User::create(data.0.username, con.as_ref())
		.await
		.into_500()?;

	let auth = Authentication::create(&user, &data.0.password, argon.as_ref(), con.as_ref())
		.await
		.into_500()?;

	Ok(HttpResponse::Ok().body(jwt.encode(&AuthClaims::create(auth)).into_500()?))
}

pub(super) fn configure(cfg: &mut ServiceConfig) {
	cfg.service(web::resource("/login").route(web::post().to(login)))
		.service(web::resource("/signup").route(web::post().to(signup)));
}
