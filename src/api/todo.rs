use actix_web::{
	error::{ErrorNotFound, ErrorUnauthorized},
	web, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;

use crate::db::{todo::Todo, Pool};

use super::{
	authorization::jwt::{AuthClaims, Authenticated},
	IntoHttpError,
};

async fn get_all(req: HttpRequest, con: web::Data<Pool>) -> impl Responder {
	let xs = req.extensions();
	let claims = xs.get::<AuthClaims>().unwrap();

	let todos = Todo::get_by_username(claims.username(), con.as_ref())
		.await
		.into_500()?;

	Ok::<_, actix_web::Error>(HttpResponse::Ok().json(todos))
}

#[derive(Debug, Deserialize)]
struct Params {
	id: i64,
}

#[derive(Debug, Deserialize)]
struct Update {
	#[serde(default)]
	text: Option<String>,
	#[serde(default)]
	completed: Option<bool>,
	#[serde(default)]
	cleared: Option<bool>,
}

async fn update(
	req: HttpRequest,
	params: web::Path<Params>,
	body: web::Json<Update>,
	con: web::Data<Pool>,
) -> impl Responder {
	let mut todo = Todo::get_by_id(params.id, con.as_ref())
		.await
		.into_500()?
		.ok_or_else(|| ErrorNotFound("todo not found"))?;

	if todo.username != req.extensions().get::<AuthClaims>().unwrap().username() {
		return Err(ErrorUnauthorized("not your todo"));
	}

	if let Some(text) = body.0.text {
		todo.text = text;
	}

	if let Some(completed) = body.0.completed {
		todo.completed = completed;
	}

	if let Some(cleared) = body.0.cleared {
		todo.cleared = cleared;
	}

	todo.update(con.as_ref()).await.into_500()?;

	Ok(HttpResponse::Ok().json(todo))
}

#[derive(Debug, Deserialize)]
struct Create {
	text: String,
}

async fn create(req: HttpRequest, con: web::Data<Pool>, body: web::Json<Create>) -> impl Responder {
	let username = req
		.extensions()
		.get::<AuthClaims>()
		.unwrap()
		.username()
		.to_owned();

	let todo = Todo::create(username, body.0.text, 0, false, false, con.as_ref())
		.await
		.into_500()?;

	Ok::<_, actix_web::Error>(HttpResponse::Ok().json(todo))
}

async fn clear_all(req: HttpRequest, con: web::Data<Pool>) -> impl Responder {
	let xs = req.extensions();
	let username = xs.get::<AuthClaims>().unwrap().username();

	sqlx::query!(
		"UPDATE todos SET cleared = TRUE WHERE username = $1 AND completed AND NOT cleared",
		username
	)
	.execute(con.as_ref())
	.await
	.into_500()?;

	Ok::<_, actix_web::Error>(
		HttpResponse::Ok().json(
			Todo::get_by_username(username, con.as_ref())
				.await
				.into_500()?,
		),
	)
}

pub(super) fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::scope("")
			.service(web::resource("/clear").route(web::post().to(clear_all)))
			.service(web::resource("/{id}").route(web::put().to(update)))
			.service(
				web::resource("")
					.route(web::get().to(get_all))
					.route(web::post().to(create)),
			)
			.wrap(Authenticated),
	);
}
