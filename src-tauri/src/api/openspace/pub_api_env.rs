use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    Eq,
    PartialEq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum ApiEnv {
    Local,
    EphDev,
    Dev,
    CAN,
    EU,
    GOV,
    JPN,
    KSA,
    UK,
    US,
    SGP,
}

impl ApiEnv {
    pub fn get_host(&self) -> &'static str {
        match self {
            ApiEnv::Local => "http://localhost:8080",
            ApiEnv::Dev => "https://development.osdevenv.net",
            ApiEnv::CAN => "https://can.openspace.ai",
            ApiEnv::EU => "https://eu.openspace.ai",
            ApiEnv::GOV => "https://gov.openspace.ai",
            ApiEnv::JPN => "https://jpn.openspace.ai",
            ApiEnv::KSA => "https://ksa.openspace.ai",
            ApiEnv::UK => "https://uk.openspace.ai",
            ApiEnv::US => "https://openspace.ai",
            ApiEnv::SGP => "https://sgp.openspace.ai",
            ApiEnv::EphDev => "https://development.osdevenv.net",
        }
    }
}
