use std::env;

use actix_files::{Files, NamedFile};
use actix_web::{web, App, HttpServer};
use anyhow::Context;

mod api;
pub mod db;

fn get_env(name: &str) -> anyhow::Result<String> {
	env::var(name).with_context(|| format!("Error loading environment variable {}.", name))
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
	dotenv::dotenv().context("Error loading .env file.")?;
	env_logger::init();

	let con = db::create(&get_env("DATABASE_URL")?)
		.await
		.context("Error connecting to database")?;

	sqlx::migrate!("./migrations")
		.run(&con)
		.await
		.context("Error running database migrations.")?;

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::new(con.clone()))
			.service(web::scope("api").configure(api::configure))
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
