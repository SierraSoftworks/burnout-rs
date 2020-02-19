#[macro_use]
extern crate serde;

extern crate actix_web;
extern crate chrono;
extern crate serde_json;
extern crate uuid;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};

mod api;
mod health;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let health_state = web::Data::new(health::HealthState::new());

    HttpServer::new(move || {
        App::new()
            .app_data(health_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(Cors::new().send_wildcard().allowed_origin("All").finish())
            .configure(health::configure)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
