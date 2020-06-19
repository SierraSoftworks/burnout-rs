use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::TeamFilter;

#[get("/api/v3/team/{team}/users")]
async fn get_team_assignments_v1(
    (state, info, token): (web::Data<GlobalState>, web::Path<TeamFilter>, AuthToken),
) -> Result<web::Json<Vec<TeamAssignmentV1>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "TeamAssignments.Write");
    
    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    let role = state.store.send(GetTeamAssignment { team_id: cid, principal_id: uid }).await??;
    match role.role {
        Role::Manager => state.store.send(GetTeamAssignments { team_id: cid }).await?.map(|roles| web::Json(roles.iter().map(|i| i.clone().into()).collect())),
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to view or manage the list of users for this team."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_team_assignments_v1() {
        test_log_init();

        test_state!(state = [
            StoreTeamAssignment {
                team_id: 1,
                principal_id: 0,
                role: Role::Manager,
            },
            StoreTeamAssignment {
                team_id: 1,
                principal_id: 2,
                role: Role::Viewer,
            }
        ]);

        let content: Vec<TeamAssignmentV1> = test_request!(GET "/api/v3/team/00000000000000000000000000000001/users" => OK with content | state = state);
        assert_eq!(content.len(), 2);

        for role in content {
            assert_eq!(role.team_id, Some("00000000000000000000000000000001".into()));
            match role.user_id.unwrap().as_str() {
                "00000000000000000000000000000000" => {
                    assert_eq!(role.role, "Manager".to_string());
                },
                "00000000000000000000000000000002" => {
                    assert_eq!(role.role, "Viewer".to_string());
                },
                _ => assert!(false)
            }
        }
    }
}