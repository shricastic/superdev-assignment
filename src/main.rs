use actix_web::{middleware::Logger, App, HttpServer};

mod routes;
mod util;

use routes::{hello::hello, keypair::generate_keypair};
use util::config::init_logger;
use crate::routes::{message::{sign_message, verify_message}, solana::send_sol, token::{create_token, mint_token, send_token}};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    init_logger();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new("%a %r \n in:%{Header}i out:%{Header}o"))
            .service(hello)
            .service(generate_keypair)
            .service(create_token)
            .service(mint_token)
            .service(send_sol)
            .service(send_token)
            .service(sign_message)
            .service(verify_message)

    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
