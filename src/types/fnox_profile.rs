use crate::FnoxSecret;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FnoxProfile {
    #[serde(default)]
    pub secrets: BTreeMap<String, FnoxSecret>,
}
