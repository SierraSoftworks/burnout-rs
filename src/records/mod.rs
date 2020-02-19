use actix_web::{web, get, post, Error}

mod models;
mod state;

pub fn configure(cfg: &mut web::ServiceConfig) {

}

#[post("/api/v1/reports")]
async fn new_report_v1() -> Result<web::Json<None>, Error> {

}