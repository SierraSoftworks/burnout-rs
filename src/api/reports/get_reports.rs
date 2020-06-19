use actix_web::{get, web};
use super::{AuthToken, APIError, ensure_user_team};
use crate::models::*;
use super::{QueryFilter, TeamFilter};
use chrono::prelude::*;

#[get("/api/v1/reports")]
async fn get_reports_v1(
    (query, state, token): (web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<ReportV1>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Reports.Read");
    
    let uid = parse_uuid!(token.oid, auth token oid);

    ensure_user_team(&state, &token).await?;
    state.store.send(GetTeamAssignment { principal_id: uid, team_id: uid }).await??;

    state.store.send(GetReports {
        team: uid,
        metric: query.metric.clone(), 
        after: query.after.clone().and_then(|after| DateTime::parse_from_rfc3339(after.as_str()).ok()).map(|dt| dt.with_timezone(&Utc)).clone()
    }).await?.map(|reports| web::Json(reports.iter().map(|i| i.clone().into()).collect()))
}

#[get("/api/v1/team/{team}/reports")]
async fn get_team_reports_v1(
    (info, query, state, token): (web::Path<TeamFilter>, web::Query<QueryFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<ReportV1>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Reports.Read");
    
    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    ensure_user_team(&state, &token).await?;
    state.store.send(GetTeamAssignment { principal_id: uid, team_id: cid }).await??;

    state.store.send(GetReports {
        team: cid,
        metric: query.metric.clone(), 
        after: query.after.clone().and_then(|after| DateTime::parse_from_rfc3339(after.as_str()).ok()).map(|dt| dt.with_timezone(&Utc)).clone()
    }).await?.map(|reports| web::Json(reports.iter().map(|i| i.clone().into()).collect()))
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn get_reports_v1() {
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

        let content: Vec<ReportV1> = test_request!(GET "/api/v1/reports" => OK with content | state = state);
        assert!(content.len() >= 1);
        assert_ne!(content[0].id, None);
        assert_eq!(content[0].team, Some("00000000000000000000000000000000".into()));
        assert_eq!(content[0].metric, "test".to_string());
        assert_eq!(content[0].value, 2.5);
    }

    #[actix_rt::test]
    async fn get_team_reports_v1() {
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

        let content: Vec<ReportV1> = test_request!(GET "/api/v1/team/00000000000000000000000000000007/reports" => OK with content | state = state);
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].id, Some("00000000000000000000000000000001".into()));
        assert_eq!(content[0].team, Some("00000000000000000000000000000007".into()));
        assert_eq!(content[0].metric, "test".to_string());
        assert_eq!(content[0].value, 2.5);
    }
}