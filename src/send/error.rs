use payjoin::bitcoin::psbt::PsbtParseError;

/// Error building a Sender from a SenderBuilder.
///
/// This error is unrecoverable.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error initializing the sender: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BuildSenderError {
    message: String,
}

impl From<PsbtParseError> for BuildSenderError {
    fn from(value: PsbtParseError) -> Self {
        BuildSenderError { message: value.to_string() }
    }
}

impl From<payjoin::send::BuildSenderError> for BuildSenderError {
    fn from(value: payjoin::send::BuildSenderError) -> Self {
        BuildSenderError { message: value.to_string() }
    }
}

/// Error returned when request could not be created.
///
/// This error can currently only happen due to programmer mistake.
/// `unwrap()`ing it is thus considered OK in Rust but you may achieve nicer message by displaying
/// it.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error creating the request: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct CreateRequestError {
    message: String,
}

impl From<payjoin::send::v2::CreateRequestError> for CreateRequestError {
    fn from(value: payjoin::send::v2::CreateRequestError) -> Self {
        CreateRequestError { message: value.to_string() }
    }
}

/// Error returned for v2-specific payload encapsulation errors.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error encapsulating the request: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct EncapsulationError {
    message: String,
}

impl From<payjoin::send::v2::EncapsulationError> for EncapsulationError {
    fn from(value: payjoin::send::v2::EncapsulationError) -> Self {
        EncapsulationError { message: value.to_string() }
    }
}

/// Error that may occur when the response from receiver is malformed.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Error validating the receiver response: {message}")]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ValidationError {
    message: String,
}

/// Represent an error returned by Payjoin receiver.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum ResponseError {
    /// `WellKnown` Errors are defined in the [`BIP78::ReceiverWellKnownError`] spec.
    ///
    /// It is safe to display `WellKnown` errors to end users.
    ///
    /// [`BIP78::ReceiverWellKnownError`]: https://github.com/bitcoin/bips/blob/master/bip-0078.mediawiki#user-content-Receivers_well_known_errors
    #[error("A receiver error occurred: ")]
    WellKnown(WellKnownError),

    /// Errors caused by malformed responses.
    #[error("An error occurred due to a malformed response: ")]
    Validation(ValidationError),

    /// `Unrecognized` Errors are NOT defined in the [`BIP78::ReceiverWellKnownError`] spec.
    ///
    /// It is NOT safe to display `Unrecognized` errors to end users as they could be used
    /// maliciously to phish a non technical user. Only display them in debug logs.
    ///
    /// [`BIP78::ReceiverWellKnownError`]: https://github.com/bitcoin/bips/blob/master/bip-0078.mediawiki#user-content-Receivers_well_known_errors
    #[error("An unrecognized error occurred")]
    Unrecognized { error_code: String, message: String },
}

impl From<payjoin::send::ResponseError> for ResponseError {
    fn from(value: payjoin::send::ResponseError) -> Self {
        use payjoin::send::ResponseError::*;

        match value {
            WellKnown(e) => ResponseError::WellKnown(e.into()),
            Validation(e) => ResponseError::Validation(ValidationError { message: e.to_string() }),
            Unrecognized { error_code, message } => {
                ResponseError::Unrecognized { error_code, message }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
pub enum WellKnownError {
    #[error("The payjoin endpoint is not available for now.")]
    Unavailable { message: String },
    #[error("The receiver added some inputs but could not bump the fee of the payjoin proposal.")]
    NotEnoughMoney { message: String },
    #[error("This version of payjoin is not supported. Use version {supported:?}.")]
    VersionUnsupported { message: String, supported: Vec<u64> },
    #[error("The receiver rejected the original PSBT.")]
    OriginalPsbtRejected { message: String },
}

impl From<payjoin::send::WellKnownError> for WellKnownError {
    fn from(value: payjoin::send::WellKnownError) -> Self {
        use payjoin::send::WellKnownError::*;

        match value {
            Unavailable(m) => WellKnownError::Unavailable { message: m },
            NotEnoughMoney(m) => WellKnownError::NotEnoughMoney { message: m },
            VersionUnsupported { message, supported } => {
                WellKnownError::VersionUnsupported { message, supported }
            }
            OriginalPsbtRejected(m) => WellKnownError::OriginalPsbtRejected { message: m },
            _ => panic!("An unexpected error occurred"),
        }
    }
}
