#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error de/serializing JSON object: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SerdeJsonError {
    message: String,
}
impl From<serde_json::Error> for SerdeJsonError {
    fn from(value: serde_json::Error) -> Self {
        SerdeJsonError { message: format!("{:?}", value) }
    }
}
