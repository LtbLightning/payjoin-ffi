use payjoin::receive;

/// The top-level error type for the payjoin receiver
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct Error(receive::Error);

impl From<receive::Error> for Error {
    fn from(value: receive::Error) -> Self {
        Self(value)
    }
}

/// The replyable error type for the payjoin receiver, representing failures need to be
/// returned to the sender.
///
/// The error handling is designed to:
/// 1. Provide structured error responses for protocol-level failures
/// 2. Hide implementation details of external errors for security
/// 3. Support proper error propagation through the receiver stack
/// 4. Provide errors according to BIP-78 JSON error specifications for return
///    after conversion into [`JsonReply`]
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct ReplyableError(receive::ReplyableError);

impl From<receive::ReplyableError> for ReplyableError {
    fn from(value: receive::ReplyableError) -> Self {
        Self(value)
    }
}

/// Error that may occur during a v2 session typestate change
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct SessionError(receive::v2::SessionError);

impl From<receive::v2::SessionError> for SessionError {
    fn from(value: receive::v2::SessionError) -> Self {
        Self(value)
    }
}

/// Error arising due to the specific receiver implementation
///
/// e.g. database errors, network failures, wallet errors
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct ImplementationError(receive::ImplementationError);

impl From<receive::ImplementationError> for ImplementationError {
    fn from(value: receive::ImplementationError) -> Self {
        Self(value)
    }
}

impl From<String> for ImplementationError {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

/// Error that may occur when output substitution fails.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct OutputSubstitutionError(receive::OutputSubstitutionError);

impl From<receive::OutputSubstitutionError> for OutputSubstitutionError {
    fn from(value: receive::OutputSubstitutionError) -> Self {
        Self(value)
    }
}

/// Error that may occur when coin selection fails.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct SelectionError(receive::SelectionError);

impl From<receive::SelectionError> for SelectionError {
    fn from(value: receive::SelectionError) -> Self {
        Self(value)
    }
}

/// Error that may occur when input contribution fails.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct InputContributionError(receive::InputContributionError);

impl From<receive::InputContributionError> for InputContributionError {
    fn from(value: receive::InputContributionError) -> Self {
        Self(value)
    }
}

/// Error validating a PSBT Input
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct PsbtInputError(receive::PsbtInputError);

impl From<receive::PsbtInputError> for PsbtInputError {
    fn from(value: receive::PsbtInputError) -> Self {
        Self(value)
    }
}
