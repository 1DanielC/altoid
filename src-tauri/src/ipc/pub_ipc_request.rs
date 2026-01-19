use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::EnumString;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, Eq, PartialEq, EnumString)]
pub enum IpcCommand {
    MakeRequest,
    Login,
    Logout,
    GetCamera,
    GetFiles,
    GetSettings,
    UploadFiles
}


pub struct IpcRequest {
    pub command: IpcCommand,
    pub payload: Option<Value>,
}