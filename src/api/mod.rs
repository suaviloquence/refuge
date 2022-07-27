use actix_web::{
	web::{self, ServiceConfig},
	HttpResponse,
};
use log::info;

mod ping;

pub type Response = actix_web::Result<HttpResponse>;

pub fn configure(cfg: &mut ServiceConfig) {
	cfg.app_data(web::JsonConfig::default().error_handler(|err, _| {
		info!("Error deserializing JSON: {:?}", err);
		actix_web::error::ErrorBadRequest(err)
	}))
	.service(web::scope("/ping").configure(ping::configure));
}
