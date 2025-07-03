use actix_web::web;

pub mod hello;
pub mod sol;
pub mod keypair;
pub mod token;
pub mod message;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg
        .service(hello::hello)
        .service(keypair::generate_keypair)
        .service(token::create_token)
        .service(token::mint_token)
        .service(token::send_token)
        .service(sol::send_sol)
        .service(message::sign_message)
        .service(message::verify_message);
}
