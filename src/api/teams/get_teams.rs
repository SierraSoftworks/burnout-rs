use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;

#[get("/api/v1/teams")]
async fn get_teams_v1(
    (state, token): (web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<TeamV1>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Teams.Read");

    let uid = parse_uuid!(token.oid, auth token oid);
        
    state.store.send(GetTeams { principal_id: uid }).await?.map(|records| web::Json(records.iter().map(|i| i.clone().into()).collect()))
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_teams_v1() {
        test_log_init();

        test_state!(state = [
            StoreTeam {
                team_id: 1,
                principal_id: 0,
                name: "Test Team".into(),
                ..Default::default()
            }
        ]);

        let content: Vec<TeamV1> = test_request!(GET "/api/v1/teams" => OK with content | state = state);
        assert!(content.len() >= 1);
        assert_eq!(content[0].id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content[0].user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content[0].name, "Test Team".to_string());
    }
}