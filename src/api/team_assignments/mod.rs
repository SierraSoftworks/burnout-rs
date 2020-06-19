mod get_team_assignment;
mod get_team_assignments;
mod store_team_assignment;
mod remove_team_assignment;

use actix_web::web;
use super::{AuthToken, APIError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg 
        .service(get_team_assignment::get_team_assignment_v1)
        .service(get_team_assignments::get_team_assignments_v1)
        .service(store_team_assignment::store_team_assignment_v1)
        .service(remove_team_assignment::remove_team_assignment_v1);
}

#[derive(Deserialize, Serialize)]
struct TeamFilter {
    team: String,
}

#[derive(Deserialize, Serialize)]
struct TeamUserFilter {
    team: String,
    user: String,
}