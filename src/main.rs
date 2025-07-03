use actix_web::{middleware::Logger, App, HttpServer};

mod routes;
mod util;

use util::config::{init_logger, json_config};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    init_logger();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new("%a %r \n in:%{Header}i out:%{Header}o"))
            .app_data(json_config())
            .configure(routes::init)

    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
