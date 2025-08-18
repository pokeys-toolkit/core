//! Error types for PoKeys library

use thiserror::Error;

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, PoKeysError>;

/// Main error type for PoKeys operations
#[derive(Error, Debug)]
pub enum PoKeysError {
    #[error("Generic error")]
    Generic,

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Device not connected")]
    NotConnected,

    #[error("Connection failed")]
    ConnectionFailed,

    #[error("Communication error")]
    CommunicationError,

    #[error("Transfer error: {0}")]
    Transfer(String),

    #[error("Invalid parameter")]
    InvalidParameter,

    #[error("Invalid parameter: {0}")]
    Parameter(String),

    #[error("Operation not supported")]
    NotSupported,

    #[error("Unsupported operation")]
    UnsupportedOperation,

    #[error("Cannot claim USB device")]
    CannotClaimUsb,

    #[error("Cannot connect to device")]
    CannotConnect,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Invalid response")]
    InvalidResponse,

    #[error("Timeout")]
    Timeout,

    #[error("Device enumeration failed")]
    EnumerationFailed,

    #[error("Invalid device handle")]
    InvalidHandle,

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    // Model-related errors
    #[error("Failed to load model from {0}: {1}")]
    ModelLoadError(String, String),

    #[error("Failed to parse model from {0}: {1}")]
    ModelParseError(String, String),

    #[error("Model validation error: {0}")]
    ModelValidationError(String),

    #[error("Failed to create model directory {0}: {1}")]
    ModelDirCreateError(String, String),

    #[error("Failed to read model directory {0}: {1}")]
    ModelDirReadError(String, String),

    #[error("Model watcher error: {0}")]
    ModelWatcherError(String),

    #[error("Pin {0} does not support capability: {1}")]
    UnsupportedPinCapability(u8, String),

    #[error("Missing related capability: Pin {0} with capability {1} requires {2}")]
    MissingRelatedCapability(u8, String, String),

    #[error("Related pin {0} with capability {1} is inactive")]
    RelatedPinInactive(u8, String),

    #[error("Related capability validation failed: {0}")]
    RelatedCapabilityError(String),

    // Pin management errors
    #[error("Pin conflict: {0}")]
    PinConflict(String),

    #[error("Invalid pin: {0}")]
    InvalidPin(u8),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
}

impl PartialEq for PoKeysError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Generic, Self::Generic) => true,
            (Self::DeviceNotFound, Self::DeviceNotFound) => true,
            (Self::NotConnected, Self::NotConnected) => true,
            (Self::ConnectionFailed, Self::ConnectionFailed) => true,
            (Self::CommunicationError, Self::CommunicationError) => true,
            (Self::Transfer(a), Self::Transfer(b)) => a == b,
            (Self::InvalidParameter, Self::InvalidParameter) => true,
            (Self::Parameter(a), Self::Parameter(b)) => a == b,
            (Self::NotSupported, Self::NotSupported) => true,
            (Self::UnsupportedOperation, Self::UnsupportedOperation) => true,
            (Self::CannotClaimUsb, Self::CannotClaimUsb) => true,
            (Self::CannotConnect, Self::CannotConnect) => true,
            (Self::InvalidChecksum, Self::InvalidChecksum) => true,
            (Self::InvalidResponse, Self::InvalidResponse) => true,
            (Self::Timeout, Self::Timeout) => true,
            (Self::EnumerationFailed, Self::EnumerationFailed) => true,
            (Self::InvalidHandle, Self::InvalidHandle) => true,
            (Self::Protocol(a), Self::Protocol(b)) => a == b,
            (Self::InternalError(a), Self::InternalError(b)) => a == b,
            // Model-related errors
            (Self::ModelLoadError(a1, b1), Self::ModelLoadError(a2, b2)) => a1 == a2 && b1 == b2,
            (Self::ModelParseError(a1, b1), Self::ModelParseError(a2, b2)) => a1 == a2 && b1 == b2,
            (Self::ModelValidationError(a), Self::ModelValidationError(b)) => a == b,
            (Self::ModelDirCreateError(a1, b1), Self::ModelDirCreateError(a2, b2)) => {
                a1 == a2 && b1 == b2
            }
            (Self::ModelDirReadError(a1, b1), Self::ModelDirReadError(a2, b2)) => {
                a1 == a2 && b1 == b2
            }
            (Self::ModelWatcherError(a), Self::ModelWatcherError(b)) => a == b,
            (Self::UnsupportedPinCapability(a1, b1), Self::UnsupportedPinCapability(a2, b2)) => {
                a1 == a2 && b1 == b2
            }
            (
                Self::MissingRelatedCapability(a1, b1, c1),
                Self::MissingRelatedCapability(a2, b2, c2),
            ) => a1 == a2 && b1 == b2 && c1 == c2,
            (Self::RelatedPinInactive(a1, b1), Self::RelatedPinInactive(a2, b2)) => {
                a1 == a2 && b1 == b2
            }
            (Self::RelatedCapabilityError(a), Self::RelatedCapabilityError(b)) => a == b,
            // IO errors are not compared
            (Self::Io(_), Self::Io(_)) => false,
            _ => false,
        }
    }
}

/// Return codes matching the original library
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnCode {
    Ok = 0,
    ErrGeneric = -1,
    ErrNotConnected = -5,
    ErrTransfer = -10,
    ErrParameter = -20,
    ErrNotSupported = -30,
    ErrCannotClaimUsb = -100,
    ErrCannotConnect = -101,
}

impl From<PoKeysError> for ReturnCode {
    fn from(error: PoKeysError) -> Self {
        match error {
            PoKeysError::Generic => ReturnCode::ErrGeneric,
            PoKeysError::NotConnected => ReturnCode::ErrNotConnected,
            PoKeysError::Transfer(_) => ReturnCode::ErrTransfer,
            PoKeysError::Parameter(_) | PoKeysError::InvalidParameter => ReturnCode::ErrParameter,
            PoKeysError::NotSupported | PoKeysError::UnsupportedOperation => {
                ReturnCode::ErrNotSupported
            }
            PoKeysError::CannotClaimUsb => ReturnCode::ErrCannotClaimUsb,
            PoKeysError::CannotConnect | PoKeysError::ConnectionFailed => {
                ReturnCode::ErrCannotConnect
            }
            // Model-related errors map to generic or parameter errors
            PoKeysError::UnsupportedPinCapability(_, _)
            | PoKeysError::MissingRelatedCapability(_, _, _)
            | PoKeysError::RelatedPinInactive(_, _)
            | PoKeysError::RelatedCapabilityError(_) => ReturnCode::ErrParameter,
            _ => ReturnCode::ErrGeneric,
        }
    }
}

impl From<ReturnCode> for PoKeysError {
    fn from(code: ReturnCode) -> Self {
        match code {
            ReturnCode::Ok => PoKeysError::Generic, // This shouldn't happen
            ReturnCode::ErrGeneric => PoKeysError::Generic,
            ReturnCode::ErrNotConnected => PoKeysError::NotConnected,
            ReturnCode::ErrTransfer => PoKeysError::Transfer("Transfer failed".to_string()),
            ReturnCode::ErrParameter => PoKeysError::Parameter("Invalid parameter".to_string()),
            ReturnCode::ErrNotSupported => PoKeysError::NotSupported,
            ReturnCode::ErrCannotClaimUsb => PoKeysError::CannotClaimUsb,
            ReturnCode::ErrCannotConnect => PoKeysError::CannotConnect,
        }
    }
}
