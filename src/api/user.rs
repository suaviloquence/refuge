use actix_web::{
	error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound},
	web::{self, ServiceConfig},
	HttpResponse, Responder,
};

use serde::{Deserialize, Serialize};

use crate::db::Pool;

#[derive(Debug, Deserialize)]
struct Login<'a> {
	username: &'a str,
	password: &'a str,
}

async fn get(con: web::Data<Pool>, data: web::Data<Login<'_>>) -> impl Responder {
	if data.username.is_empty() || data.password.is_empty() {
		return Err(ErrorBadRequest("username/password cannot be empty"));
	}

	Ok(HttpResponse::Ok().body(()))
}

pub(super) fn configure(cfg: &mut ServiceConfig) {
	cfg.service(web::resource("/").route(web::get().to(get)));
}
