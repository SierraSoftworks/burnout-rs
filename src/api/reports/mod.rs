
use actix_web::web;
use super::{AuthToken, APIError, ensure_user_team};

mod new_report;
mod get_reports;
mod get_report;
mod remove_report;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_reports::get_reports_v1)
        .service(get_reports::get_team_reports_v1)
        .service(get_report::get_report_v1)
        .service(get_report::get_team_report_v1)
        .service(new_report::new_report_v1)
        .service(new_report::new_team_report_v1)
        .service(remove_report::remove_report_v1)
        .service(remove_report::remove_team_report_v1);
}

#[derive(Deserialize, Serialize)]
struct IdFilter {
    id: String,
}

#[derive(Deserialize, Serialize)]
struct TeamFilter {
    team: String,
}

#[derive(Deserialize, Serialize)]
struct TeamIdFilter {
    team: String,
    id: String,
}

#[derive(Deserialize)]
pub struct QueryFilter {
    metric: Option<String>,
    after: Option<String>,
}
