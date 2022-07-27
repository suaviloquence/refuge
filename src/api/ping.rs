use actix_web::{
	error::{ErrorInternalServerError, ErrorNotFound},
	web::{self, ServiceConfig},
	HttpResponse,
};

use serde::Serialize;

use crate::db::Pool;

use super::Response;

#[derive(Debug, Serialize)]
pub struct Pings {
	pings: i64,
}

async fn get(con: web::Data<Pool>) -> Response {
	match sqlx::query_as!(Pings, "SELECT * FROM pings LIMIT 1")
		.fetch_optional(&**con)
		.await
	{
		Ok(Some(pings)) => Ok(HttpResponse::Ok().json(pings)),
		Ok(None) => Err(ErrorNotFound("no pings recorded")),
		Err(e) => Err(ErrorInternalServerError(e)),
	}
}

async fn put(con: web::Data<Pool>) -> Response {
	match sqlx::query_as!(Pings, "SELECT * FROM pings LIMIT 1")
		.fetch_optional(&**con)
		.await
	{
		Ok(Some(mut pings)) => {
			pings.pings += 1;
			sqlx::query!("UPDATE pings SET (pings) = ($1)", pings.pings)
				.execute(&**con)
				.await
				.map(|_| HttpResponse::Ok().json(pings))
				.map_err(ErrorInternalServerError)
		}
		Ok(None) => sqlx::query!("INSERT INTO pings (pings) VALUES ($1)", 1)
			.execute(&**con)
			.await
			.map(|_| HttpResponse::Ok().json(Pings { pings: 1 }))
			.map_err(ErrorInternalServerError),
		Err(e) => Err(ErrorInternalServerError(e)),
	}
}

pub(super) fn configure(cfg: &mut ServiceConfig) {
	cfg.service(
		web::resource("/")
			.route(web::get().to(get))
			.route(web::put().to(put)),
	);
}
