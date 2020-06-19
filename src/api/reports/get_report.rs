use actix_web::{get, web};
use super::{AuthToken, APIError, ensure_user_team};
use crate::models::*;
use super::{IdFilter, TeamIdFilter};


#[get("/api/v1/report/{id}")]
async fn get_report_v1(
    (info, state, token): (web::Path<IdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<ReportV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Reports.Read");
    
    let id = parse_uuid!(info.id, report ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_team(&state, &token).await?;
    
    state.store.send(GetReport { team: uid, id: id }).await?.map(|report| report.clone().into())
}

#[get("/api/v1/team/{team}/report/{id}")]
async fn get_team_report_v1(
    (info, state, token): (web::Path<TeamIdFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<ReportV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Reports.Read");
    
    let id = parse_uuid!(info.id, report ID);
    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_team(&state, &token).await?;

    state.store.send(GetTeamAssignment { principal_id: uid, team_id: cid }).await??;

    state.store.send(GetReport { team: cid, id: id }).await?.map(|report| report.clone().into())
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_report_v1() {
        test_log_init();

        test_state!(state = [
            StoreReport {
                id: 1,
                team: 0,
                metric: "test".into(),
                value: 2.5,
                ..Default::default()
            }
        ]);

        let content: ReportV1 = test_request!(GET "/api/v1/report/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.team, Some("00000000000000000000000000000000".into()));
        assert_eq!(content.metric, "test".to_string());
        assert_eq!(content.value, 2.5);
    }

    #[actix_rt::test]
    async fn get_team_report_v1() {
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
                metric: "test".into(),
                value: 2.5,
                ..Default::default()
            }
        ]);

        let content: ReportV1 = test_request!(GET "/api/v1/team/00000000000000000000000000000007/report/00000000000000000000000000000001" => OK with content | state = state);
        assert_eq!(content.id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content.team, Some("00000000000000000000000000000007".into()));
        assert_eq!(content.metric, "test".to_string());
        assert_eq!(content.value, 2.5);

        let user = state.store.send(GetUser { email_hash: u128::from_str_radix("05c1de2f5c5e7933bee97a499e818c5e", 16).expect("a valid hash") })
            .await.expect("the actor to have run").expect("the user should exist");
        assert_eq!(user.first_name, "Testy");
        assert_eq!(user.principal_id, 0);
    }
}