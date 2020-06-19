use actix::prelude::*;
use crate::api::APIError;
use super::new_id;
use chrono::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub id: u128,
    pub team_id: u128,
    pub timestamp: DateTime<Utc>,
    pub metric: String,
    pub value: f32
}

actor_message!(GetReport(id: u128, team: u128) -> Report);

actor_message!(GetReports(team: u128, metric: Option<String>, after: Option<DateTime<Utc>>) -> Vec<Report>);

actor_message!(StoreReport(id: u128, team: u128, metric: String, timestamp: Option<DateTime<Utc>>, value: f32) -> Report);

actor_message!(RemoveReport(id: u128, team: u128) -> ());


#[derive(Debug, Serialize, Deserialize)]
pub struct ReportV1 {
    pub team: Option<String>,
    pub id: Option<String>,
    pub timestamp: Option<String>,
    pub metric: String,
    pub value: f32,
}

json_responder!(ReportV1 => (req, model) -> if req.uri().path().contains("/team/") {
    req.url_for("get_team_report_v1", &vec![
        model.team.clone().expect("a team id"),
        model.id.clone().expect("a report id")
    ]) 
} else {
    req.url_for("get_report_v1", vec![model.id.clone().expect("a report id")])
});

impl From<Report> for ReportV1 {
    fn from(report: Report) -> Self {
        Self {
            id: Some(format!("{:0>32x}", report.id)),
            team: Some(format!("{:0>32x}", report.team_id)),
            timestamp: Some(report.timestamp.to_rfc3339()),
            metric: report.metric.clone(),
            value: report.value,
        }
    }
}

impl Into<Report> for ReportV1 {
    fn into(self) -> Report {
        Report {
            id: self.id.clone().and_then(|id| u128::from_str_radix(&id, 16).ok()).unwrap_or_else(|| new_id()),
            team_id: self.team.clone().and_then(|id| u128::from_str_radix(&id, 16).ok()).unwrap_or_default(),
            timestamp: self.timestamp.clone().and_then(|ts| DateTime::parse_from_rfc3339(ts.as_str()).ok()).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|| Utc::now()),
            metric: self.metric.clone(),
            value: self.value.clone(),
        }
    }
}