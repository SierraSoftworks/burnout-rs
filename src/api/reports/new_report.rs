use actix_web::{post, web};
use super::{AuthToken, APIError, ensure_user_team};
use crate::models::*;
use super::TeamFilter;
use chrono::prelude::*;


#[post("/api/v1/reports")]
async fn new_report_v1(
    (new_report, state, token): (web::Json<ReportV1>, web::Data<GlobalState>, AuthToken),
) -> Result<web::Json<Vec<ReportV1>>, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Reports.Write");
    
    let report: Report = new_report.into_inner().into();
    let uid = parse_uuid!(token.oid, auth token oid);

    ensure_user_team(&state, &token).await?;

    let teams = state.store.send(GetTeams { principal_id: uid }).await??;

    let id = new_id();
    let timestamp = Utc::now();

    let mut reports: Vec<ReportV1> = Vec::new();
    for team in teams {
        let report = state.store.send(StoreReport {
            id: id,
            team: team.team_id,
            metric: report.metric.clone(),
            timestamp: Some(timestamp),
            value: report.value,
        }).await?.map(|report| report.clone().into())?;
        reports.push(report);
    }

    Ok(web::Json(reports))
}

#[post("/api/v1/team/{team}/reports")]
async fn new_team_report_v1(
    (new_report, info, state, token): (web::Json<ReportV1>, web::Path<TeamFilter>, web::Data<GlobalState>, AuthToken),
) -> Result<ReportV1, APIError> {
    require_role!(token, "Administrator", "User");
    require_scope!(token, "Reports.Write");
    
    let report: Report = new_report.into_inner().into();
    let cid = parse_uuid!(info.team, team ID);
    let uid = parse_uuid!(token.oid, auth token oid);
        
    if cid == uid {
        ensure_user_team(&state, &token).await?;
    }

    let role = state.store.send(GetTeamAssignment { principal_id: uid, team_id: cid }).await??;

    match role.role {
        Role::Manager | Role::Member => {
            state.store.send(StoreReport {
                id: new_id(),
                team: cid,
                metric: report.metric.clone(),
                timestamp: Some(Utc::now()),
                value: report.value,
            }).await?.map(|report| report.clone().into())
        },
        _ => Err(APIError::new(403, "Forbidden", "You do not have permission to add an report to this team."))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use crate::api::test::*;

    #[actix_rt::test]
    async fn new_report_v1() {
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
            }
        ]);

        let content: Vec<ReportV1> = test_request!(POST "/api/v1/reports", ReportV1 {
            id: None,
            team: None,
            timestamp: None,
            metric: "test".to_string(),
            value: 2.5,
        } => OK with content | state = state);

        assert_eq!(content.len(), 2);

        for report in content {
            assert_ne!(report.id, None);
            assert_ne!(report.team, None);
            assert_eq!(report.metric, "test".to_string());
            assert_eq!(report.value, 2.5);

            state.store.send(GetReport {
                team: 0,
                id: u128::from_str_radix(report.id.clone().unwrap().as_str(), 16).unwrap(),
            }).await.expect("the actor should have run").expect("The report should exist in the store");
        }
    }

    #[actix_rt::test]
    async fn new_team_report_v1() {
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
            }
        ]);

        let content: ReportV1 = test_request!(POST "/api/v1/team/00000000000000000000000000000007/reports", ReportV1 {
            id: None,
            team: None,
            timestamp: None,
            metric: "test".to_string(),
            value: 2.5,
        } => CREATED with location =~ "/api/v1/team/00000000000000000000000000000007/report/", content | state = state);

        assert_ne!(content.id, None);
        assert_eq!(content.team, Some("00000000000000000000000000000007".into()));
        assert_eq!(content.metric, "test".to_string());
        assert_eq!(content.value, 2.5);

        state.store.send(GetReport {
            team: 7,
            id: u128::from_str_radix(content.id.unwrap().as_str(), 16).unwrap(),
        }).await.expect("the actor should have run").expect("The report should exist in the store");
    }
}