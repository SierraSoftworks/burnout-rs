use actix_web::{post, web};
use super::{AuthToken, APIError};
use crate::models::*;

#[post("/api/v1/teams")]
async fn new_team_v1(
    (team, state, token): (
        web::Json<TeamV1>,
        web::Data<GlobalState>, AuthToken),
) -> Result<TeamV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Teams.Write");
    
    let uid = parse_uuid!(token.oid, auth token oid);
        
    let team = state.store.send(StoreTeam {
        principal_id: uid,
        team_id: new_id(),
        name: team.name.clone(),
    }).await??;

    state.store.send(StoreTeamAssignment {
        principal_id: uid,
        team_id: team.team_id,
        role: Role::Manager
    }).await??;
    
    Ok(team.into())
}

#[cfg(test)]
mod tests {
    use crate::api::test::*;
    use crate::models::*;

    #[actix_rt::test]
    async fn new_team_v1() {
        test_log_init();
        
        let content: TeamV1 = test_request!(POST "/api/v1/teams", TeamV1 {
            id: None,
            user_id: None,
            name: "Test Team".into(),
        } => CREATED with content);

        assert_ne!(content.id, None);
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Team".to_string());
    }
}