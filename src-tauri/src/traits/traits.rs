use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::{Error, Value};

/// converts any serializable type to a json value
pub trait ToJson {
    fn to_json(&self) -> Result<Value, Error>;
}

// Blanket impl: applies to all T that implement Serialize
impl<T> ToJson for T
where
    T: Serialize,
{
    fn to_json(&self) -> Result<Value, Error> {
        serde_json::to_value(self)
    }
}
