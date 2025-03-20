use payjoin::IntoUrlError;

/// The top-level error type for the payjoin receiver
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum Error {
    /// Errors that can be replied to the sender
    #[error("Replyable error: {0}")]
    ReplyToSender(ReplyableError),
    /// V2-specific errors that are infeasable to reply to the sender
    #[error("Unreplyable error: {message}")]
    V2 { message: String },
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ReplyableError {
    /// Error arising from validation of the original PSBT payload
    #[error("Error while validating original PSBT payload: {message}")]
    Payload { message: String },
    /// Protocol-specific errors for BIP-78 v1 requests (e.g. HTTP request validation, parameter checks)
    #[error("Error while validating V1 request: {message}")]
    V1 { message: String },
    /// Error arising due to the specific receiver implementation
    ///
    /// e.g. database errors, network failures, wallet errors
    #[error("{0}")]
    Implementation(ImplementationError),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error occurred in receiver implementation: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ImplementationError {
    message: String,
}

impl From<payjoin::receive::Error> for Error {
    fn from(value: payjoin::receive::Error) -> Self {
        match value {
            payjoin::receive::Error::ReplyToSender(e) => Error::ReplyToSender(e.into()),
            payjoin::receive::Error::V2(_) => Error::V2 { message: value.to_string() },
            _ => panic!("An unexpected error occurred"),
        }
    }
}

impl From<payjoin::receive::ReplyableError> for ReplyableError {
    fn from(value: payjoin::receive::ReplyableError) -> Self {
        use payjoin::receive::ReplyableError::*;

        match value {
            Payload(_) => ReplyableError::Payload { message: value.to_string() },
            V1(_) => ReplyableError::V1 { message: value.to_string() },
            Implementation(_) => {
                ReplyableError::Implementation(ImplementationError { message: value.to_string() })
            }
        }
    }
}

impl From<payjoin::bitcoin::address::ParseError> for Error {
    fn from(value: payjoin::bitcoin::address::ParseError) -> Self {
        Error::V2 { message: value.to_string() }
    }
}

impl From<IntoUrlError> for Error {
    fn from(value: IntoUrlError) -> Self {
        Error::V2 { message: value.to_string() }
    }
}

/// Error that may occur when output substitution fails.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Output substition error: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct OutputSubstitutionError {
    message: String,
}

impl From<payjoin::receive::OutputSubstitutionError> for OutputSubstitutionError {
    fn from(value: payjoin::receive::OutputSubstitutionError) -> Self {
        OutputSubstitutionError { message: value.to_string() }
    }
}

/// Error that may occur when coin selection fails.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error occurred during coin selection: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct SelectionError {
    message: String,
}

impl From<payjoin::receive::SelectionError> for SelectionError {
    fn from(value: payjoin::receive::SelectionError) -> Self {
        SelectionError { message: value.to_string() }
    }
}

/// Error that may occur when input contribution fails.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Input contribution error: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct InputContributionError {
    message: String,
}

impl From<payjoin::receive::InputContributionError> for InputContributionError {
    fn from(value: payjoin::receive::InputContributionError) -> Self {
        InputContributionError { message: value.to_string() }
    }
}

/// Error validating a PSBT Input
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error validating PSBT input: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PsbtInputError {
    message: String,
}

impl From<payjoin::receive::PsbtInputError> for PsbtInputError {
    fn from(value: payjoin::receive::PsbtInputError) -> Self {
        PsbtInputError { message: value.to_string() }
    }
}

impl From<String> for ImplementationError {
    fn from(message: String) -> Self {
        ImplementationError { message }
    }
}
