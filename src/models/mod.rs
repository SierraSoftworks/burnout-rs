#[macro_use] mod macros;

mod team;
mod report;
mod team_assignment;
mod health;
mod user;

use actix::prelude::*;

pub use team::*;
pub use health::*;
pub use report::*;
pub use team_assignment::*;
pub use user::*;

pub fn new_id() -> u128 {
    let id = uuid::Uuid::new_v4();
    u128::from_be_bytes(*id.as_bytes())
}

#[derive(Clone)]
pub struct GlobalState {
    pub store: Addr<crate::store::Store>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            store: crate::store::Store::new().start(),
        }
    }
}