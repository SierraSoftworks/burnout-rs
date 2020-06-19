mod get_team;
mod get_teams;
mod new_team;
mod store_team;
mod remove_team;

use actix_web::web;
use super::{AuthToken, APIError};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_team::get_team_v1)
        .service(get_teams::get_teams_v1)
        .service(new_team::new_team_v1)
        .service(store_team::store_team_v1)
        .service(remove_team::remove_team_v1);
}

#[derive(Deserialize, Serialize)]
struct TeamFilter {
    team: String,
}