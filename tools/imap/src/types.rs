use serde::{Deserialize, Serialize};

pub type IdentifierDeclarationType = (String, u32, usize, String);

#[derive(Serialize, Deserialize)]
pub struct Mapping {
    pub original: String,
    pub modified: String,
    pub scope_id: u32,
    pub id: usize,
    pub declaration_type: String,
}
