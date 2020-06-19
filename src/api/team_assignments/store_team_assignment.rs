use actix_web::{put, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::TeamUserFilter;

#[put("/api/v1/team/{team}/user/{user}")]
async fn store_team_assignment_v1(
    (info, team, state, token): (web::Path<TeamUserFilter>,
        web::Json<TeamAssignmentV1>,
        web::Data<GlobalState>, AuthToken),
) -> Result<TeamAssignmentV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "TeamAssignments.Write");
    
    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);
    let tuid = parse_uuid!(info.user, user ID);
    
    let original_team = state.store.send(GetTeam {
        id: cid,
        principal_id: uid
    }).await??;

    if tuid == uid {
        return Err(APIError::new(400, "Bad Request", "You cannot modify your own role assignment. Please request that another team owner performs this task for you."))
    }

    let role = state.store.send(GetTeamAssignment { team_id: cid, principal_id: uid }).await??;
    match role.role {
        Role::Manager => {
            match state.store.send(GetTeam {
                principal_id: tuid,
                id: cid
            }).await? {
                Ok(_) => {},
                Err(err) if err.code == 404 => {
                    state.store.send(StoreTeam {
                        principal_id: tuid,
                        team_id: cid,
                        name: original_team.name
                    }).await??;
                },
                Err(err) => {
                    return Err(err)
                }
            }

            state.store.send(StoreTeamAssignment {
                principal_id: tuid,
                team_id: cid,
                role: team.role.as_str().into(),
            }).await?.map(|team| team.clone().into())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to view or manage the list of users for this team."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn store_team_assignment_v1() {
        test_log_init();

        test_state!(state = [
            StoreTeam {
                team_id: 1,
                principal_id: 0,
                name: "Test Team".into()
            },
            StoreTeamAssignment {
                team_id: 1,
                principal_id: 0,
                role: Role::Manager,
            }
        ]);

        let content: TeamAssignmentV1 = test_request!(PUT "/api/v1/team/00000000000000000000000000000001/user/00000000000000000000000000000002", TeamAssignmentV1{
            team_id: None,
            user_id: None,
            role: "Manager".into(),
        } => OK with content | state = state);

        assert_eq!(content.team_id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.user_id, Some("00000000000000000000000000000002".into()));
        assert_eq!(content.role, "Manager".to_string());

        let team = state.store.send(GetTeam {
            id: 1,
            principal_id: 2
        }).await.expect("the actor should run").expect("the user should have the new team");
        assert_eq!(team.name, "Test Team");
    }

    #[actix_rt::test]
    async fn store_team_assignment_v3_self() {
        test_log_init();

        test_state!(state = [
            StoreTeam {
                team_id: 1,
                principal_id: 0,
                name: "Test Team".into()
            },
            StoreTeamAssignment {
                team_id: 1,
                principal_id: 0,
                role: Role::Manager,
            }
        ]);

        test_request!(PUT "/api/v1/team/00000000000000000000000000000001/user/00000000000000000000000000000000", TeamAssignmentV1{
            team_id: None,
            user_id: None,
            role: "Viewer".into(),
        } => BAD_REQUEST | state = state);
    }
}