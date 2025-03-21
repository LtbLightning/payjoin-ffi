#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error parsing the payjoin URI: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PjParseError {
    message: String,
}

impl From<String> for PjParseError {
    fn from(message: String) -> Self {
        PjParseError { message }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("URI doesn't support payjoin: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PjNotSupported {
    message: String,
}

impl From<String> for PjNotSupported {
    fn from(message: String) -> Self {
        PjNotSupported { message }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error parsing URL: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct UrlParseError {
    message: String,
}

impl From<payjoin::ParseError> for UrlParseError {
    fn from(value: payjoin::ParseError) -> Self {
        UrlParseError { message: format!("{:?}", value) }
    }
}
