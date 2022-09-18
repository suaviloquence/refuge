use std::env;

use actix_files::{Files, NamedFile};
use actix_web::{web, App, HttpServer};
use anyhow::Context;
use api::JwtConfig;
use jsonwebtoken::{DecodingKey, EncodingKey};

#[macro_use]
extern crate lazy_static;

mod api;
pub mod db;

fn get_env(name: &str) -> anyhow::Result<String> {
	env::var(name).with_context(|| format!("Error loading environment variable {}.", name))
}

lazy_static! {
	static ref ARGON_SECRET: Vec<u8> = get_env("PASSWORD_SECRET")
		.expect("Needs env var PASSWORD_SECRET")
		.into_bytes();
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
	dotenv::dotenv().context("Error loading .env file.")?;
	env_logger::init();

	let con = web::Data::new(
		db::create(&get_env("DATABASE_URL")?)
			.await
			.context("Error connecting to database")?,
	);

	sqlx::migrate!("./migrations")
		.run(con.as_ref())
		.await
		.context("Error running database migrations.")?;

	let argon = web::Data::new(argon2::Config::<'static> {
		secret: &ARGON_SECRET,
		..Default::default()
	});

	let jwt = web::Data::new(JwtConfig::new(
		EncodingKey::from_secret(get_env("JWT_SECRET")?.as_bytes()),
		DecodingKey::from_secret(get_env("JWT_SECRET")?.as_bytes()),
		jsonwebtoken::Algorithm::HS256,
	));

	HttpServer::new(move || {
		App::new()
			.app_data(con.clone())
			.app_data(argon.clone())
			.app_data(jwt.clone())
			.service(web::scope("/api").configure(api::configure))
			.service(Files::new("/static", "./static"))
			.default_service(web::to(|| NamedFile::open_async("./static/index.html")))
	})
	.bind((
		get_env("IP")?,
		get_env("PORT")?
			.parse::<u16>()
			.context("PORT is not a u16")?,
	))?
	.workers(2)
	.run()
	.await?;

	Ok(())
}
