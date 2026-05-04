//! PCM data handling utilities
//!
//! Implementation will be done in Phase 5.

use crate::MmlResult;

/// PCM format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcmFormat {
    /// 8-bit unsigned PCM
    U8,
    /// 8-bit signed PCM
    S8,
    /// 16-bit little-endian signed PCM
    S16LE,
    /// 16-bit big-endian signed PCM
    S16BE,
    /// 32-bit float PCM
    F32,
}

/// Convert PCM data between formats
pub fn convert_pcm(_data: &[u8], _from: PcmFormat, _to: PcmFormat) -> MmlResult<Vec<u8>> {
    unimplemented!("PCM conversion not yet implemented")
}
