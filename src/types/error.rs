use std::fmt;

/// Unified error type for the UVC camera node
#[derive(Debug)]
pub enum Error {
    /// Camera hardware errors
    Camera(String),
    
    /// Invalid frame rate value
    InvalidFrameRate(u16),
    
    /// Invalid resolution
    InvalidResolution { width: u16, height: u16 },
    
    /// Image encoding failed
    EncodingError(String),
    
    /// Invalid data length for encoding
    InvalidDataLength { expected: usize, got: usize },
    
    /// Device path parsing failed
    InvalidDevicePath(String),
    
    /// Thread panicked
    ThreadPanic(String),
    
    /// Generic error with context
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Camera(msg) => write!(f, "Camera error: {msg}"),
            Self::InvalidFrameRate(fps) => write!(f, "Invalid frame rate: {fps} fps (must be 1-240)"),
            Self::InvalidResolution { width, height } => {
                write!(f, "Invalid resolution: {width}x{height}")
            }
            Self::EncodingError(msg) => write!(f, "Encoding error: {msg}"),
            Self::InvalidDataLength { expected, got } => {
                write!(f, "Invalid data length: expected {expected} bytes, got {got}")
            }
            Self::InvalidDevicePath(path) => write!(f, "Invalid device path: {path}"),
            Self::ThreadPanic(msg) => write!(f, "Thread panicked: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience Result type using our Error
pub type Result<T> = std::result::Result<T, Error>;

/// Convert from anyhow::Error
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

/// Convert from string errors
impl From<String> for Error {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}
