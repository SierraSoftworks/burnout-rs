use actix_web::{put, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::TeamFilter;

#[put("/api/v1/team/{team}")]
async fn store_team_v1(
    (info, team, state, token): (web::Path<TeamFilter>,
        web::Json<TeamV1>,
        web::Data<GlobalState>, AuthToken),
) -> Result<TeamV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Teams.Write");
    
    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    state.store.send(StoreTeam {
        principal_id: uid,
        team_id: cid,
        name: team.name.clone(),
    }).await?.map(|team| team.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn store_team_v1() {
        test_log_init();

        let content: TeamV1 = test_request!(PUT "/api/v1/team/00000000000000000000000000000001", TeamV1 {
            id: None,
            user_id: None,
            name: "Test Team".into(),
        } => OK with content);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Team".to_string());
    }
}