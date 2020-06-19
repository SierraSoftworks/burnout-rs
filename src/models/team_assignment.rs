use actix::prelude::*;
use crate::api::APIError;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Manager,
    Member,
    Viewer,
    Invalid,
}

impl Default for Role {
    fn default() -> Self {
        Role::Viewer
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        match s {
            "Manager" => Role::Manager,
            "Member" => Role::Member,
            "Viewer" => Role::Viewer,
            _ => Role::Invalid
        }
    }
}

impl Into<String> for Role {
    fn into(self) -> String {
        match self {
            Role::Manager => "Manager".into(),
            Role::Member => "Member".into(),
            Role::Viewer => "Viewer".into(),
            Role::Invalid => "INVALID".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TeamAssignment {
    pub user_id: u128,
    pub team_id: u128,
    pub role: Role,
}

actor_message!(GetTeamAssignment(team_id: u128, principal_id: u128) -> TeamAssignment);

actor_message!(GetTeamAssignments(team_id: u128) -> Vec<TeamAssignment>);

actor_message!(StoreTeamAssignment(team_id: u128, principal_id: u128, role: Role) -> TeamAssignment);

actor_message!(RemoveTeamAssignment(team_id: u128, principal_id: u128) -> ());


#[derive(Debug, Serialize, Deserialize)]
pub struct TeamAssignmentV1 {
    #[serde(rename="teamId")]
    pub team_id: Option<String>,
    #[serde(rename="userId")]
    pub user_id: Option<String>,
    pub role: String,
}

json_responder!(TeamAssignmentV1 => (req, model) -> req.url_for("get_team_assignment_v3", &vec![
    model.team_id.clone().expect("a team id"),
    model.user_id.clone().expect("a user id")
]));

impl From<TeamAssignment> for TeamAssignmentV1 {
    fn from(record: TeamAssignment) -> Self {
        Self {
            user_id: Some(format!("{:0>32x}", record.user_id)),
            team_id: Some(format!("{:0>32x}", record.team_id)),
            role: record.role.into(),
        }
    }
}

impl Into<TeamAssignment> for TeamAssignmentV1 {
    fn into(self) -> TeamAssignment {
        TeamAssignment {
            user_id: self.user_id.clone().and_then(|id| u128::from_str_radix(&id, 16).ok()).unwrap_or_default(),
            team_id: self.team_id.clone().and_then(|id| u128::from_str_radix(&id, 16).ok()).unwrap_or_default(),
            role: self.role.as_str().into(),
        }
    }
}

