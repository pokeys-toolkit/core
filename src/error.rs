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

    // Enhanced I2C errors (Priority 1)
    #[error("I2C packet too large: {size} bytes (maximum {max_size} bytes). {suggestion}")]
    I2cPacketTooLarge {
        size: usize,
        max_size: usize,
        suggestion: String,
    },

    #[error("I2C timeout")]
    I2cTimeout,

    #[error("I2C bus error")]
    I2cBusError,

    #[error("I2C NACK received")]
    I2cNack,

    #[error("Network timeout")]
    NetworkTimeout,

    #[error("Maximum retries exceeded")]
    MaxRetriesExceeded,

    // Enhanced validation errors (Priority 3)
    #[error("Invalid packet structure: {0}")]
    InvalidPacketStructure(String),

    #[error("Invalid command: 0x{0:02X}")]
    InvalidCommand(u8),

    #[error("Invalid device ID: {0}")]
    InvalidDeviceId(u8),

    #[error("Invalid checksum: expected 0x{expected:02X}, received 0x{received:02X}")]
    InvalidChecksumDetailed { expected: u8, received: u8 },

    // uSPIBridge-specific errors
    #[error("Invalid segment mapping: {0}")]
    InvalidSegmentMapping(String),

    #[error("Segment mapping not supported by device")]
    SegmentMappingNotSupported,

    #[error("Custom pinout configuration error: {0}")]
    CustomPinoutError(String),

    #[error("uSPIBridge command failed: {0}")]
    USPIBridgeCommandFailed(String),

    #[error("Virtual device error: {0}")]
    VirtualDeviceError(String),

    #[error("Invalid virtual device ID: {id} (maximum: {max})")]
    InvalidVirtualDeviceId { id: u8, max: u8 },
}

/// Recovery strategies for different error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    Fail,
    RetryWithDelay(u64), // milliseconds
    RetryWithBackoff,
    ResetAndRetry,
}

impl PoKeysError {
    /// Check if an error is recoverable through retry mechanisms
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            PoKeysError::I2cTimeout
                | PoKeysError::I2cBusError
                | PoKeysError::I2cNack
                | PoKeysError::NetworkTimeout
                | PoKeysError::Timeout
                | PoKeysError::CommunicationError
                | PoKeysError::USPIBridgeCommandFailed(_)
        )
    }

    /// Get the recommended recovery strategy for this error
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            PoKeysError::I2cTimeout => RecoveryStrategy::RetryWithDelay(100),
            PoKeysError::I2cBusError => RecoveryStrategy::ResetAndRetry,
            PoKeysError::I2cNack => RecoveryStrategy::RetryWithBackoff,
            PoKeysError::NetworkTimeout => RecoveryStrategy::RetryWithDelay(200),
            PoKeysError::Timeout => RecoveryStrategy::RetryWithDelay(100),
            PoKeysError::CommunicationError => RecoveryStrategy::RetryWithBackoff,
            _ => RecoveryStrategy::Fail,
        }
    }
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
            // Enhanced I2C errors
            (
                Self::I2cPacketTooLarge {
                    size: s1,
                    max_size: m1,
                    suggestion: sg1,
                },
                Self::I2cPacketTooLarge {
                    size: s2,
                    max_size: m2,
                    suggestion: sg2,
                },
            ) => s1 == s2 && m1 == m2 && sg1 == sg2,
            (Self::I2cTimeout, Self::I2cTimeout) => true,
            (Self::I2cBusError, Self::I2cBusError) => true,
            (Self::I2cNack, Self::I2cNack) => true,
            (Self::NetworkTimeout, Self::NetworkTimeout) => true,
            (Self::MaxRetriesExceeded, Self::MaxRetriesExceeded) => true,
            // Enhanced validation errors
            (Self::InvalidPacketStructure(a), Self::InvalidPacketStructure(b)) => a == b,
            (Self::InvalidCommand(a), Self::InvalidCommand(b)) => a == b,
            (Self::InvalidDeviceId(a), Self::InvalidDeviceId(b)) => a == b,
            (
                Self::InvalidChecksumDetailed {
                    expected: e1,
                    received: r1,
                },
                Self::InvalidChecksumDetailed {
                    expected: e2,
                    received: r2,
                },
            ) => e1 == e2 && r1 == r2,
            // uSPIBridge-specific errors
            (Self::InvalidSegmentMapping(a), Self::InvalidSegmentMapping(b)) => a == b,
            (Self::SegmentMappingNotSupported, Self::SegmentMappingNotSupported) => true,
            (Self::CustomPinoutError(a), Self::CustomPinoutError(b)) => a == b,
            (Self::USPIBridgeCommandFailed(a), Self::USPIBridgeCommandFailed(b)) => a == b,
            (Self::VirtualDeviceError(a), Self::VirtualDeviceError(b)) => a == b,
            (
                Self::InvalidVirtualDeviceId { id: i1, max: m1 },
                Self::InvalidVirtualDeviceId { id: i2, max: m2 },
            ) => i1 == i2 && m1 == m2,
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
            // Enhanced I2C errors
            PoKeysError::I2cPacketTooLarge { .. } => ReturnCode::ErrParameter,
            PoKeysError::I2cTimeout | PoKeysError::I2cBusError | PoKeysError::I2cNack => {
                ReturnCode::ErrTransfer
            }
            PoKeysError::NetworkTimeout => ReturnCode::ErrTransfer,
            PoKeysError::MaxRetriesExceeded => ReturnCode::ErrTransfer,
            // Enhanced validation errors
            PoKeysError::InvalidPacketStructure(_)
            | PoKeysError::InvalidCommand(_)
            | PoKeysError::InvalidDeviceId(_)
            | PoKeysError::InvalidChecksumDetailed { .. } => ReturnCode::ErrParameter,
            // uSPIBridge-specific errors
            PoKeysError::InvalidSegmentMapping(_)
            | PoKeysError::SegmentMappingNotSupported
            | PoKeysError::CustomPinoutError(_)
            | PoKeysError::VirtualDeviceError(_)
            | PoKeysError::InvalidVirtualDeviceId { .. } => ReturnCode::ErrParameter,
            PoKeysError::USPIBridgeCommandFailed(_) => ReturnCode::ErrTransfer,
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
