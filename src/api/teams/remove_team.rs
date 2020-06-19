use actix_web::{delete, web};
use super::{AuthToken, APIError};
use crate::models::*;
use super::TeamFilter;

#[delete("/api/v1/team/{team}")]
async fn remove_team_v1(
    (info, state, token): (web::Path<TeamFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Teams.Write");
    
    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);

    state.store.send(RemoveTeam { id: cid, principal_id: uid }).await??;

    state.store.send(RemoveTeamAssignment { team_id: cid, principal_id: uid }).await??;

    Ok(web::HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn remove_team_v1() {
        test_log_init();

        test_state!(state = [
            StoreTeam {
                team_id: 1,
                principal_id: 0,
                name: "Test Team".into(),
                ..Default::default()
            },
            StoreTeamAssignment {
                team_id: 1,
                principal_id: 0,
                role: Role::Manager,
            }
        ]);

        test_request!(DELETE "/api/v1/team/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state.store.send(GetTeam {
            id: 1,
            principal_id: 1
        }).await.expect("the actor should have run").expect_err("The team should not exist anymore");

        state.store.send(GetTeamAssignment {
            team_id: 1,
            principal_id: 1
        }).await.expect("the actor should have run").expect_err("The role assignment should not exist anymore");
    }
}