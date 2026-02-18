//! Configuration constants for the UVC camera node

/// Default camera configuration
pub mod defaults {
    pub const FRAME_RATE: u16 = 30;
    pub const WIDTH: u16 = 640;
    pub const HEIGHT: u16 = 480;
}

/// Camera limits
pub mod limits {
    // No artificial limits - hardware determines valid values
}

/// JPEG encoding settings
pub mod jpeg {
    pub const QUALITY: u8 = 85;
}

/// Frame rate constraints
pub mod frame_rate {
    /// Below this threshold, warn about low frame rate
    pub const LOW_WARNING_THRESHOLD: u16 = 5;
}
