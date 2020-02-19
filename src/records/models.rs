use chrono::DateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: u128,
    pub uid: u128,
    pub time: DateTime,
    pub value: f32,
}
