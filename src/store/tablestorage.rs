use crate::models::*;
use crate::api::APIError;
use std::{fmt::Debug, sync::{Arc}};
use chrono::prelude::*;
use actix::prelude::*;
use azure_sdk_storage_table::{CloudTable, Continuation, TableClient, TableEntity};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub struct TableStorage {
    started_at: chrono::DateTime<chrono::Utc>,

    reports: Arc<CloudTable>,
    team_assignments: Arc<CloudTable>,
    teams: Arc<CloudTable>,
    users: Arc<CloudTable>,
}

const URI_CHARACTERS: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'%')
    .add(b'#')
    .add(b'&');

impl TableStorage {
    pub fn new() -> Self {
        let connection_string = std::env::var("TABLE_STORAGE_CONNECTION_STRING").expect("Set the TABLE_STORAGE_CONNECTION_STRING environment variable before starting the server.");

        let client = TableClient::from_connection_string(&connection_string).expect("a valid connection string");
        let reports_table = CloudTable::new(client.clone(), "reports");
        let team_assignments_table = CloudTable::new(client.clone(), "teamassignments");
        let teams_table = CloudTable::new(client.clone(), "teams");
        let users_table = CloudTable::new(client, "users");

        Self {
            started_at: chrono::Utc::now(),

            reports: Arc::new(reports_table),
            teams: Arc::new(teams_table),
            team_assignments: Arc::new(team_assignments_table),
            users: Arc::new(users_table),
        }
    }

    async fn get_single<ST, T>(table: Arc<CloudTable>, partition_key: u128, row_key: u128, not_found_err: APIError) -> Result<T, APIError>
    where
        ST: DeserializeOwned + Clone,
        T: From<TableEntity<ST>> {
        let result = table.get::<ST>(
            &format!("{:0>32x}", partition_key), 
            &format!("{:0>32x}", row_key),
            None
        ).await?;

        result
            .ok_or(not_found_err)
            .map(|r| r.into())
    }

    async fn get_all<ST, T, P>(table: Arc<CloudTable>, query: String, filter: P) -> Result<Vec<T>, APIError>
    where
        ST: Serialize + DeserializeOwned + Clone,
        P: Fn(&TableEntity<ST>) -> bool,
        T: From<TableEntity<ST>>
    {
        let mut continuation = Continuation::start();

        let mut entries: Vec<TableEntity<ST>> = vec![];
        let safe_query = TableStorage::escape_query(query);

        while let Some(mut results) = table.execute_query::<ST>(if safe_query.is_empty() { None } else { Some(safe_query.as_str()) }, &mut continuation).await? {
            entries.append(&mut results);
        }

        Ok(entries.iter().filter(|&e| filter(e)).map(|e| e.clone().into()).collect())
    }

    async fn store_single<ST, T>(table: Arc<CloudTable>, item: TableEntity<ST>) -> Result<T, APIError> 
    where
        ST: Serialize + DeserializeOwned + Clone + Debug,
        T: From<TableEntity<ST>> {
        let result = table.insert_or_update_entity(item).await?;

        Ok(result.into())
    }

    async fn remove_single(table: Arc<CloudTable>, partition_key: u128, row_key: u128) -> Result<(), APIError> {
        table.delete(
            &format!("{:0>32x}", partition_key), 
            &format!("{:0>32x}", row_key),
            None).await?;

        Ok(())
    }

    fn build_report_filter_query(partition_key: u128, metric: Option<String>, after: Option<DateTime<Utc>>) -> String {
        let mut query = format!("$filter=PartitionKey eq '{:0>32x}'", partition_key);
        
        match metric {
            Some(metric) => {
                query = query + format!(" and Metric eq {}", metric).as_str()
            },
            None => {}
        }

        match after {
            Some(after) => {
                query = query + format!(" and Timestamp gt {}", after.to_rfc3339()).as_str()
            },
            None => {}
        }

        query
    }

    fn escape_query(query: String) -> String {
        percent_encoding::percent_encode(query.as_bytes(), URI_CHARACTERS).to_string()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableStorageReport {
    #[serde(rename="Metric")]
    pub metric: String,
    #[serde(rename="Timestamp")]
    pub timestamp: String,
    #[serde(rename="Value")]
    pub value: f32,
}

impl From<TableEntity<TableStorageReport>> for Report {
    fn from(entity: TableEntity<TableStorageReport>) -> Self {
        Self {
            id: u128::from_str_radix(&entity.row_key, 16).unwrap_or_default(),
            team_id: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            metric: entity.payload.metric.clone(),
            timestamp: DateTime::parse_from_rfc3339(entity.payload.timestamp.as_str()).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            value: entity.payload.value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableStorageTeam {
    #[serde(rename="Name")]
    pub name: String,
}

impl From<TableEntity<TableStorageTeam>> for Team {
    fn from(entity: TableEntity<TableStorageTeam>) -> Self {
        Self {
            team_id: u128::from_str_radix(&entity.row_key, 16).unwrap_or_default(),
            user_id: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            name: entity.payload.name.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableStorageTeamAssignment {
    #[serde(rename="Role")]
    pub role: String,
}

impl From<TableEntity<TableStorageTeamAssignment>> for TeamAssignment {
    fn from(entity: TableEntity<TableStorageTeamAssignment>) -> Self {
        Self {
            team_id: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            user_id: u128::from_str_radix(&entity.row_key, 16).unwrap_or_default(),
            role: entity.payload.role.as_str().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TableStorageUser {
    #[serde(rename="PrincipalId")]
    pub principal_id: String,

    #[serde(rename="FirstName")]
    pub first_name: String,
}

impl From<TableEntity<TableStorageUser>> for User {
    fn from(entity: TableEntity<TableStorageUser>) -> Self {
        Self {
            email_hash: u128::from_str_radix(&entity.partition_key, 16).unwrap_or_default(),
            principal_id: u128::from_str_radix(&entity.payload.principal_id, 16).unwrap_or_default(),
            first_name: entity.payload.first_name.as_str().into(),
        }
    }
}

macro_rules! actor_handler {
    ($msg:ty => $res:ty: handler = $handler:item) => {
        impl Handler<$msg> for TableStorage {
            type Result = ResponseActFuture<Self, Result<$res, APIError>>;
            
            $handler
        }
    };

    ($msg:ty|$src:ident => $res:ty: get_single from $table:ident ( $st:ty ) where pk=$pk:expr, rk=$rk:expr; not found = $err:expr) => {
        actor_handler!($msg => $res: handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let work = TableStorage::get_single::<$st, $res>(
                table,
                $pk,
                $rk,
                APIError::new(404, "Not Found", $err));
            Box::new(fut::wrap_future(work))
        });
    };

    ($msg:ty|$src:ident => $res:ty: get_all from $table:ident ( $st:ty ) where query = $query:expr, context = [$($ctx:tt)*], filter = $fid:ident -> $filter:expr) => {
        actor_handler!($msg => Vec<$res>: handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let query = $query;

            $($ctx)*

            let work = TableStorage::get_all::<$st, $res, _>(
                table,
                query,
                move |$fid| $filter
            );

            Box::new(fut::wrap_future(work))
        });
    };

    ($msg:ty|$src:ident => $res:ty: get_random from $table:ident ( $st:ty ) where query = $query:expr, context = [$($ctx:tt)*], filter = $fid:ident -> $filter:expr; not found = $err:expr) => {
        actor_handler!($msg => $res: handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let query = $query;

            $($ctx)*

            let work = TableStorage::get_random::<$st, $res, _>(
                table,
                query,
                move |$fid| $filter,
                APIError::new(404, "Not Found", $err)
            );

            Box::new(fut::wrap_future(work))
        });
    };

    ($msg:ty|$src:ident: remove_single from $table:ident where pk=$pk:expr, rk=$rk:expr) => {
        actor_handler!($msg => (): handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let work = TableStorage::remove_single(
                table,
                $pk,
                $rk);
            Box::new(fut::wrap_future(work))
        });
    };
    
    ($msg:ty|$src:ident => $res:ty: store_single in $table:ident ( $st:ty ) $item:expr) => {
        actor_handler!($msg => $res: handler = fn handle(&mut self, $src: $msg, _: &mut Self::Context) -> Self::Result {
            let table = self.$table.clone();
            let item = $item;
            let work = TableStorage::store_single::<$st, $res>(
                table,
                item
            );

            Box::new(fut::wrap_future(work))
        });
    };
}

impl Actor for TableStorage {
    type Context = Context<Self>;
}

impl Handler<GetHealth> for TableStorage {
    type Result = Result<Health, APIError>;

    fn handle(&mut self, _: GetHealth, _: &mut Self::Context) -> Self::Result {
        Ok(Health {
            ok: true,
            started_at: self.started_at.clone(),
        })
    }
}

actor_handler!(GetReport|msg => Report: get_single from reports(TableStorageReport) where pk=msg.team, rk=msg.id; not found = "The combination of team and record ID you provided could not be found. Please check them and try again.");

actor_handler!(GetReports|msg => Report: get_all from reports(TableStorageReport) where
    query=TableStorage::build_report_filter_query(msg.team, msg.metric.clone(), msg.after.clone()),
    context = [],
    filter=_i -> true);


actor_handler!(StoreReport|msg => Report: store_single in reports(TableStorageReport) TableEntity {
    partition_key: format!("{:0>32x}", msg.team),
    row_key: format!("{:0>32x}", msg.id),
    payload: TableStorageReport {
        metric: msg.metric.clone(),
        timestamp: msg.timestamp.clone().unwrap_or_else(|| Utc::now()).to_rfc3339(),
        value: msg.value,
    },
    etag: None,
    timestamp: None
});

actor_handler!(RemoveReport|msg: remove_single from reports where pk=msg.team, rk=msg.id);

actor_handler!(GetTeam|msg => Team: get_single from teams(TableStorageTeam) where pk=msg.principal_id, rk=msg.id; not found = "The team ID you provided could not be found. Please check them and try again.");

actor_handler!(GetTeams|msg => Team: get_all from teams(TableStorageTeam) where
    query = format!("$filter=PartitionKey eq '{:0>32x}'", msg.principal_id),
    context = [],
    filter = _i -> true);

actor_handler!(StoreTeam|msg => Team: store_single in teams(TableStorageTeam) TableEntity {
    partition_key: format!("{:0>32x}", msg.principal_id),
    row_key: format!("{:0>32x}", msg.team_id),
    payload: TableStorageTeam {
        name: msg.name.clone(),
    },
    etag: None,
    timestamp: None
});

actor_handler!(RemoveTeam|msg: remove_single from teams where pk=msg.principal_id, rk=msg.id);

actor_handler!(GetTeamAssignment|msg => TeamAssignment: get_single from team_assignments(TableStorageTeamAssignment) where pk=msg.team_id, rk=msg.principal_id; not found = "The team ID you provided could not be found. Please check them and try again.");

actor_handler!(GetTeamAssignments|msg => TeamAssignment: get_all from team_assignments(TableStorageTeamAssignment) where
    query = format!("$filter=PartitionKey eq '{:0>32x}'", msg.team_id),
    context = [],
    filter = _i -> true);

actor_handler!(StoreTeamAssignment|msg => TeamAssignment: store_single in team_assignments(TableStorageTeamAssignment) TableEntity {
    partition_key: format!("{:0>32x}", msg.team_id),
    row_key: format!("{:0>32x}", msg.principal_id),
    payload: TableStorageTeamAssignment {
        role: msg.role.into()
    },
    etag: None,
    timestamp: None
}); 

actor_handler!(RemoveTeamAssignment|msg: remove_single from team_assignments where pk=msg.team_id, rk=msg.principal_id);

actor_handler!(GetUser|msg => User: get_single from users(TableStorageUser) where pk=msg.email_hash, rk=msg.email_hash; not found = "The user you are looking for could not be found. Please check that you have entered their email address correctly and try again.");

actor_handler!(StoreUser|msg => User: store_single in users(TableStorageUser) TableEntity {
    partition_key: format!("{:0>32x}", msg.email_hash),
    row_key: format!("{:0>32x}", msg.email_hash),
    payload: TableStorageUser {
        principal_id: format!("{:0>32x}", msg.principal_id),
        first_name: msg.first_name.clone(),
    },
    etag: None,
    timestamp: None
});