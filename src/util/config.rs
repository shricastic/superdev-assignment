use actix_web::{web::JsonConfig, HttpResponse, error};
use crate::util::types::ApiResponse; 
use env_logger::Env;

pub fn init_logger(){
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
}

pub fn json_config() -> JsonConfig {
    JsonConfig::default().error_handler(|err, _req| {
        let msg = format!("{}", err);
        let json = ApiResponse::<()>::Error {
            success: false,
            error: msg,
        };
        error::InternalError::from_response("", HttpResponse::BadRequest().json(json)).into()
    })
}
