use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::TeamFilter;

#[get("/api/v1/team/{team}")]
async fn get_team_v1(
    (info, state, token): (web::Path<TeamFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<TeamV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Teams.Read");

    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    state.store.send(GetTeam { id: cid, principal_id: uid }).await?.map(|team| team.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_team_v1() {
        test_log_init();

        test_state!(state = [
            StoreTeam {
                team_id: 1,
                principal_id: 0,
                name: "Test Team".into(),
                ..Default::default()
            }
        ]);

        let content: TeamV1 = test_request!(GET "/api/v1/team/00000000000000000000000000000001" => OK with content | state = state);

        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.name, "Test Team".to_string());
    }
}