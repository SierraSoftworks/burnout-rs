use actix_web::{get, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::TeamUserFilter;

#[get("/api/v1/team/{team}/user/{user}")]
async fn get_team_assignment_v1(
    (info, state, token): (web::Path<TeamUserFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<TeamAssignmentV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "TeamAssignments.Write");
    
    let tid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);
    let tuid = parse_uuid!(info.user, user ID);

    if uid != tuid {
        let role = state.store.send(GetTeamAssignment { team_id: tid, principal_id: uid }).await??;

        if role.role != Role::Manager {
            return Err(APIError::new(403, "Forbidden", "You do not have permission to view or manage the list of members for this team."));
        }
    }

    state.store.send(GetTeamAssignment { team_id: tid, principal_id: tuid }).await?.map(|role| role.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_team_assignment_v1() {
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

        let content: TeamAssignmentV1 = test_request!(GET "/api/v1/team/00000000000000000000000000000001/user/00000000000000000000000000000002" => OK with content | state = state);
        assert_eq!(content.team_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000002".into()));
        assert_eq!(content.role, "Viewer".to_string());
    }

    #[actix_rt::test]
    async fn get_team_assignment_v1_self() {
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

        let content: TeamAssignmentV1 = test_request!(GET "/api/v1/team/00000000000000000000000000000001/user/00000000000000000000000000000000" => OK with content | state = state);
        assert_eq!(content.team_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.role, "Manager".to_string());
    }
}