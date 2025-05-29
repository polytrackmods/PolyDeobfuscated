use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub scope_id: usize,
    pub id: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mapping {
    pub original: Identifier,
    pub modified: Identifier,
}
