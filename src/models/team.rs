use actix::prelude::*;
use crate::api::APIError;
use super::new_id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub team_id: u128,
    pub user_id: u128,
    pub name: String,
}

actor_message!(GetTeam(id: u128, principal_id: u128) -> Team);

actor_message!(GetTeams(principal_id: u128) -> Vec<Team>);

actor_message!(StoreTeam(team_id: u128, principal_id: u128, name: String) -> Team);

actor_message!(RemoveTeam(id: u128, principal_id: u128) -> ());

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamV1 {
    pub id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub name: String,
}

json_responder!(TeamV1 => (req, model) -> req.url_for("get_team_v1", vec![model.id.clone().expect("a team id")]));

impl From<Team> for TeamV1 {
    fn from(record: Team) -> Self {
        Self {
            id: Some(format!("{:0>32x}", record.team_id)),
            user_id: Some(format!("{:0>32x}", record.user_id)),
            name: record.name.clone(),
        }
    }
}

impl Into<Team> for TeamV1 {
    fn into(self) -> Team {
        Team {
            user_id: self.user_id.clone().and_then(|id| u128::from_str_radix(&id, 16).ok()).unwrap_or_default(),
            team_id: self.id.clone().and_then(|id| u128::from_str_radix(&id, 16).ok()).unwrap_or_else(|| new_id()),
            name: self.name.clone(),
        }
    }
}