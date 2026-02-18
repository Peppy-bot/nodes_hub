use std::time::Instant;
use super::Encoding;

/// Frame identifier (wrapping counter)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FrameId(u32);

impl FrameId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    
    pub fn next(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }
    
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for FrameId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<FrameId> for u32 {
    fn from(id: FrameId) -> Self {
        id.0
    }
}

/// Raw RGB frame data from camera (always RGB8 from nokhwa)
#[derive(Debug, Clone)]
pub struct RawFrame {
    data: Vec<u8>,
    width: u16,
    height: u16,
    timestamp: Instant,
}

impl RawFrame {
    pub fn new(data: Vec<u8>, width: u16, height: u16, timestamp: Instant) -> Self {
        Self {
            data,
            width,
            height,
            timestamp,
        }
    }
    
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    pub fn width(&self) -> u16 {
        self.width
    }
    
    pub fn height(&self) -> u16 {
        self.height
    }
    
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Processed frame ready for emission
#[derive(Debug)]
pub struct Frame {
    data: Vec<u8>,
    width: u16,
    height: u16,
    frame_id: FrameId,
    timestamp: Instant,
    encoding: Encoding,
}

impl Frame {
    pub fn new(
        data: Vec<u8>,
        width: u16,
        height: u16,
        frame_id: FrameId,
        timestamp: Instant,
        encoding: Encoding,
    ) -> Self {
        Self {
            data,
            width,
            height,
            frame_id,
            timestamp,
            encoding,
        }
    }
    
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    pub fn width(&self) -> u16 {
        self.width
    }
    
    pub fn height(&self) -> u16 {
        self.height
    }
    
    pub fn width_u32(&self) -> u32 {
        u32::from(self.width)
    }
    
    pub fn height_u32(&self) -> u32 {
        u32::from(self.height)
    }
    
    pub fn frame_id(&self) -> FrameId {
        self.frame_id
    }
    
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }
    
    pub fn encoding(&self) -> Encoding {
        self.encoding
    }
}
