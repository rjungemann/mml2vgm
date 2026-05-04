//! WAV file I/O utilities
//!
//! Implementation will be done in Phase 5.

use crate::MmlResult;

/// Read WAV file
pub fn read_wav(_path: &std::path::Path) -> MmlResult<Vec<f32>> {
    unimplemented!("WAV reading not yet implemented")
}

/// Write WAV file
pub fn write_wav(_path: &std::path::Path, _samples: &[f32], _sample_rate: u32) -> MmlResult<()> {
    unimplemented!("WAV writing not yet implemented")
}
