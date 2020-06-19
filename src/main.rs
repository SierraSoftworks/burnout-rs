extern crate actix_web;
extern crate chrono;
#[macro_use] extern crate serde;
extern crate rand;
extern crate serde_json;
extern crate uuid;
#[macro_use] extern crate log;
#[macro_use] extern crate sentry;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate prometheus;

#[macro_use] mod macros;

mod api;
mod models;
mod store;

use actix_cors::Cors;
use actix_web::{middleware, App, HttpServer};
use actix_web_prom::PrometheusMetrics;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let raven = sentry::init((
        "https://fa46d8cff21f41ac8420c18d75acb8b9@o219072.ingest.sentry.io/5281227",
        sentry::ClientOptions {
            release: release_name!(),
            ..Default::default()
        },
    ));

    if raven.is_enabled() {
        sentry::integrations::panic::register_panic_handler();
        sentry::integrations::env_logger::init(None, Default::default());
    }

    let state = models::GlobalState::new();
    let metrics = PrometheusMetrics::new_with_registry(prometheus::default_registry().clone(), "rex", Some("/api/v1/metrics"), None).unwrap();

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(metrics.clone())
            .wrap(middleware::Logger::default())
            .wrap(Cors::new().send_wildcard().finish())
            .configure(api::configure)
    })
        .bind("0.0.0.0:8000")?
        .run()
        .await
        .map_err(|err| {
            error!("The server exited unexpectedly: {}", err);
            sentry::capture_event(sentry::protocol::Event {
                message: Some(format!("Server Exited Unexpectedly: {}", err).into()),
                level: sentry::protocol::Level::Fatal,
                ..Default::default()
            });

            err
        })
}
