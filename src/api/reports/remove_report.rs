use actix_web::{delete, web};
use super::{AuthToken, APIError, ensure_user_team};
use crate::models::*;
use super::{IdFilter, TeamIdFilter};


#[delete("/api/v1/report/{id}")]
async fn remove_report_v1(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Reports.Write");
    
    let id = parse_uuid!(info.id, report ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    state.store.send(RemoveReport { team: uid, id: id }).await??;
    
    Ok(web::HttpResponse::NoContent().finish())
}

#[delete("/api/v1/team/{team}/report/{id}")]
async fn remove_team_report_v1(
    (info, state, token): (web::Path<TeamIdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::HttpResponse, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Reports.Write");
    
    let id = parse_uuid!(info.id, report ID);
    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_team(&state, &token).await?;

    let role = state.store.send(GetTeamAssignment { principal_id: uid, team_id: cid }).await??;

    match role.role {
        Role::Manager | Role::Member => {
            state.store.send(RemoveReport { team: cid, id: id }).await??;
            
            Ok(web::HttpResponse::NoContent().finish())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to remove an report from this team."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn remove_report_v1() {
        test_log_init();

        test_state!(state = [
            StoreReport {
                id: 1,
                team: 0,
                ..Default::default()
            }
        ]);

        test_request!(DELETE "/api/v1/report/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state.store.send(GetReport {
            team: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The report should not exist anymore");
    }

    #[actix_rt::test]
    async fn remove_team_report_v1() {
        test_log_init();

        test_state!(state = [
            StoreTeam {
                team_id: 7,
                principal_id: 0,
                name: "Test Team".into(),
                ..Default::default()
            },
            StoreTeamAssignment {
                team_id: 7,
                principal_id: 0,
                role: Role::Manager,
            },
            StoreReport {
                id: 1,
                team: 7,
                ..Default::default()
            }
        ]);

        test_request!(DELETE "/api/v1/team/00000000000000000000000000000007/report/00000000000000000000000000000001" => NO_CONTENT | state = state);

        state.store.send(GetReport {
            team: 0,
            id: 1
        }).await.expect("the actor should have run").expect_err("The report should not exist anymore");
    }
}