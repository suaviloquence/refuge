use actix_web::{
	error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound},
	web::{self, ServiceConfig},
};
use log::info;
use std::fmt;

mod authorization;
mod todo;

pub trait IntoHttpError<T> {
	fn into_400(self) -> actix_web::Result<T>;
	fn into_404(self) -> actix_web::Result<T>;
	fn into_500(self) -> actix_web::Result<T>;
}

impl<T, E: fmt::Display + fmt::Debug + 'static> IntoHttpError<T> for Result<T, E> {
	#[inline]
	fn into_400(self) -> actix_web::Result<T> {
		self.map_err(ErrorBadRequest)
	}

	#[inline]
	fn into_404(self) -> actix_web::Result<T> {
		self.map_err(ErrorNotFound)
	}

	#[inline]
	fn into_500(self) -> actix_web::Result<T> {
		self.map_err(ErrorInternalServerError)
	}
}

pub fn configure(cfg: &mut ServiceConfig) {
	cfg.app_data(web::JsonConfig::default().error_handler(|err, _| {
		info!("Error deserializing JSON: {:?}", err);
		actix_web::error::ErrorBadRequest(err)
	}))
	.service(web::scope("/user").configure(authorization::configure))
	.service(web::scope("/todo").configure(todo::configure));
}

pub(crate) use authorization::jwt::JwtConfig;
