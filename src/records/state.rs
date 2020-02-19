use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use super::models;

#[derive(Clone)]
pub struct RecordsState {
    pub store: Arc<RwLock<BTreeMap<u128, Vec<models::Record>>>>,
}
