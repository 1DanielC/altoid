use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct UserInfo {
    pub email: String,
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
}
