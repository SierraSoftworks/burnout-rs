use std::{collections::BTreeMap, sync::Arc};
use crate::models::*;
use crate::api::APIError;
use std::sync::RwLock;
use chrono::prelude::*;
use actix::prelude::*;

pub struct MemoryStore {
    started_at: chrono::DateTime<chrono::Utc>,
    reports: Arc<RwLock<BTreeMap<u128, BTreeMap<u128, Report>>>>,
    teams: Arc<RwLock<BTreeMap<u128, BTreeMap<u128, Team>>>>,
    team_assignments: Arc<RwLock<BTreeMap<u128, BTreeMap<u128, TeamAssignment>>>>,
    users: Arc<RwLock<BTreeMap<u128, User>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            started_at: chrono::Utc::now(),
            reports: Arc::new(RwLock::new(BTreeMap::new())),
            teams: Arc::new(RwLock::new(BTreeMap::new())),
            team_assignments: Arc::new(RwLock::new(BTreeMap::new())),
            users: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl Actor for MemoryStore {
    type Context = Context<Self>;
}

impl Handler<GetHealth> for MemoryStore {
    type Result = Result<Health, APIError>;

    fn handle(&mut self, _: GetHealth, _: &mut Self::Context) -> Self::Result {
        Ok(Health {
            ok: true,
            started_at: self.started_at.clone(),
        })
    }
}

impl Handler<GetReport> for MemoryStore {
    type Result = Result<Report, APIError>;

    fn handle(&mut self, msg: GetReport, _: &mut Self::Context) -> Self::Result {

        let is = self.reports.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.team)
            .ok_or(APIError::new(404, "Not Found", "The team ID you provided could not be found. Please check it and try again."))
            .and_then(|c| 
                c.get(&msg.id).map(|i| i.clone())
                .ok_or(APIError::new(404, "Not Found", "The report ID you provided could not be found. Please check it and try again.")))
            
    }
}

impl Handler<GetReports> for MemoryStore {
    type Result = Result<Vec<Report>, APIError>;

    fn handle(&mut self, msg: GetReports, _: &mut Self::Context) -> Self::Result {

        let is = self.reports.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.team)
            .ok_or(APIError::new(404, "Not Found", "The team ID you provided could not be found. Please check it and try again."))
            .map(|items| items.iter().filter(|(_, i)| {
                match msg.after.clone() {
                    Some(after) => {
                        if i.timestamp < after {
                            return false;
                        }
                    },
                    None => {}
                }

                match msg.metric.clone() {
                    Some(metric) => {
                        if i.metric != metric {
                            return false;
                        }
                    },
                    None => {}
                }

                true
            }).map(|(_id, report)| report.clone()).collect())
    }
}

impl Handler<StoreReport> for MemoryStore {
    type Result = Result<Report, APIError>;

    fn handle(&mut self, msg: StoreReport, _: &mut Self::Context) -> Self::Result {

        let mut is = self.reports.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        let report = Report {
            id: msg.id,
            team_id: msg.team,
            metric: msg.metric.clone(),
            timestamp: msg.timestamp.clone().unwrap_or_else(|| Utc::now()),
            value: msg.value,
        };
        
        is.entry(msg.team)
            .or_insert_with(|| BTreeMap::new())
            .insert(report.id, report.clone());

        Ok(report)
    }
}

impl Handler<RemoveReport> for MemoryStore {
    type Result = Result<(), APIError>;

    fn handle(&mut self, msg: RemoveReport, _: &mut Self::Context) -> Self::Result {

        let mut is = self.reports.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get_mut(&msg.team)
            .ok_or(APIError::new(404, "Not Found", "The team ID you provided could not be found. Please check it and try again."))
            .and_then(|c|
                c.remove(&msg.id)
                .map(|_| ())
                .ok_or(APIError::new(404, "Not Found", "The report ID you provided could not be found. Please check it and try again.")))
            
    }
}

impl Handler<GetTeam> for MemoryStore {
    type Result = Result<Team, APIError>;

    fn handle(&mut self, msg: GetTeam, _: &mut Self::Context) -> Self::Result {

        let is = self.teams.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.principal_id)
            .ok_or(APIError::new(404, "Not Found", "The principal ID you provided could not be found. This likely means that you do not yet have any teams."))
            .and_then(|c| 
                c.get(&msg.id).map(|i| i.clone())
                .ok_or(APIError::new(404, "Not Found", "The team ID you provided could not be found. Please check it and try again.")))
            
    }
}

impl Handler<GetTeams> for MemoryStore {
    type Result = Result<Vec<Team>, APIError>;

    fn handle(&mut self, msg: GetTeams, _: &mut Self::Context) -> Self::Result {
        let is = self.teams.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.principal_id)
            .ok_or(APIError::new(404, "Not Found", "The principal ID you provided could not be found. This probably means that you do not yet have any teams."))
            .map(|items| items.iter()
                .map(|(_id, team)| team.clone()).collect())
    }
}

impl Handler<StoreTeam> for MemoryStore {
    type Result = Result<Team, APIError>;

    fn handle(&mut self, msg: StoreTeam, _: &mut Self::Context) -> Self::Result {

        let mut is = self.teams.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        let team = Team {
            team_id: msg.team_id,
            user_id: msg.principal_id,
            name: msg.name.clone(),
        };
        
        is.entry(msg.principal_id)
            .or_insert_with(|| BTreeMap::new())
            .insert(team.team_id, team.clone());

        Ok(team)
    }
}

impl Handler<RemoveTeam> for MemoryStore {
    type Result = Result<(), APIError>;

    fn handle(&mut self, msg: RemoveTeam, _: &mut Self::Context) -> Self::Result {

        let mut is = self.teams.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get_mut(&msg.principal_id)
            .ok_or(APIError::new(404, "Not Found", "The principal ID you provided could not be found. This likely means that you do not yet have any teams."))
            .and_then(|c|
                c.remove(&msg.id)
                .map(|_| ())
                .ok_or_else(|| {
                    debug!("Could not find a team entry for {} in the current user's team list ({}).", msg.id, msg.principal_id);
                    APIError::new(404, "Not Found", "The team ID you provided could not be found. Please check it and try again.")
                }))
    }
}

impl Handler<GetTeamAssignment> for MemoryStore {
    type Result = Result<TeamAssignment, APIError>;

    fn handle(&mut self, msg: GetTeamAssignment, _: &mut Self::Context) -> Self::Result {

        let is = self.team_assignments.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.team_id)
            .ok_or(APIError::new(403, "Forbidden", "You do not have permission to access this resource."))
            .and_then(|c| 
                c.get(&msg.principal_id).map(|i| i.clone())
                .ok_or(APIError::new(403, "Forbidden", "You do not have permission to access this resource.")))
            
    }
}

impl Handler<GetTeamAssignments> for MemoryStore {
    type Result = Result<Vec<TeamAssignment>, APIError>;

    fn handle(&mut self, msg: GetTeamAssignments, _: &mut Self::Context) -> Self::Result {
        let is = self.team_assignments.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get(&msg.team_id)
            .ok_or_else(|| {
                debug!("Could not find a team entry for {} in role assignments.", msg.team_id);
                APIError::new(404, "Not Found", "The team ID you provided could not be found. Please check it and try again.")
            })
            .map(|items| items.iter()
                .map(|(_id, team)| team.clone()).collect())
    }
}

impl Handler<StoreTeamAssignment> for MemoryStore {
    type Result = Result<TeamAssignment, APIError>;

    fn handle(&mut self, msg: StoreTeamAssignment, _: &mut Self::Context) -> Self::Result {

        let mut is = self.team_assignments.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        let team_assignment = TeamAssignment {
            team_id: msg.team_id,
            user_id: msg.principal_id,
            role: msg.role.clone(),
        };
        
        is.entry(msg.team_id)
            .or_insert_with(|| BTreeMap::new())
            .insert(team_assignment.user_id, team_assignment.clone());

        Ok(team_assignment)
    }
}

impl Handler<RemoveTeamAssignment> for MemoryStore {
    type Result = Result<(), APIError>;

    fn handle(&mut self, msg: RemoveTeamAssignment, _: &mut Self::Context) -> Self::Result {

        let mut is = self.team_assignments.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        is.get_mut(&msg.team_id)
        .ok_or_else(|| {
            debug!("Could not find a team entry for {} in role assignments.", msg.team_id);
            APIError::new(404, "Not Found", "The team ID you provided could not be found. Please check it and try again.")
        })
            .and_then(|c|
                c.remove(&msg.principal_id)
                .map(|_| ())
                .ok_or_else(|| {
                    debug!("Could not find an entry for the user {} in the team role assignments table for {}", msg.principal_id, msg.team_id);
                    APIError::new(404, "Not Found", "The principal ID you provided could not be found. This likely means that you do not yet have any teams.")
                }))
    }
}

impl Handler<GetUser> for MemoryStore {
    type Result = Result<User, APIError>;

    fn handle(&mut self, msg: GetUser, _: &mut Self::Context) -> Self::Result {
        let users = self.users.read()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        users.get(&msg.email_hash)
            .ok_or(APIError::new(404, "Not Found", "No user could be found with the email hash you provided. Please check it and try again."))
            .map(|u| u.clone())
    }
}

impl Handler<StoreUser> for MemoryStore {
    type Result = Result<User, APIError>;

    fn handle(&mut self, msg: StoreUser, _: &mut Self::Context) -> Self::Result {
        let mut users = self.users.write()
            .map_err(|_| APIError::new(500, "Internal Server Error", "The service is currently unavailable, please try again later."))?;

        let user = User {
            principal_id: msg.principal_id,
            email_hash: msg.email_hash,
            first_name: msg.first_name.clone()
        };

        users.insert(msg.email_hash, user.clone());

        Ok(user)
    }
}