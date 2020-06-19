#[macro_use] mod macros;

mod auth;
mod error;
mod teams;
mod reports;
mod health;
mod team_assignments;
mod users;
mod utils;

#[cfg(test)] pub mod test;

use actix_web::web;

pub use error::APIError;
pub use auth::AuthToken;
pub use utils::ensure_user_team;

pub fn configure(cfg: &mut web::ServiceConfig) {
    health::configure(cfg);
    teams::configure(cfg);
    team_assignments::configure(cfg);
    reports::configure(cfg);
    users::configure(cfg);
}