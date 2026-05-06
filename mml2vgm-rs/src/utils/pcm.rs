//! PCM data handling utilities

use crate::MmlResult;

/// PCM sample format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcmFormat {
    U8,
    S8,
    S16LE,
    S16BE,
    F32,
}

/// Convert PCM data between formats.
pub fn convert_pcm(data: &[u8], from: PcmFormat, to: PcmFormat) -> MmlResult<Vec<u8>> {
    use PcmFormat::*;
    if from == to {
        return Ok(data.to_vec());
    }
    match (from, to) {
        (U8, S16LE) => Ok(data.iter()
            .flat_map(|&b| { let s = ((b as i16) - 128) * 256; s.to_le_bytes() })
            .collect()),
        (S8, S16LE) => Ok(data.iter()
            .flat_map(|&b| { let s = (b as i8) as i16 * 256; s.to_le_bytes() })
            .collect()),
        (S16BE, S16LE) => Ok(data.chunks_exact(2)
            .flat_map(|c| i16::from_be_bytes([c[0], c[1]]).to_le_bytes())
            .collect()),
        (S16LE, S16BE) => Ok(data.chunks_exact(2)
            .flat_map(|c| i16::from_le_bytes([c[0], c[1]]).to_be_bytes())
            .collect()),
        (F32, S16LE) => Ok(data.chunks_exact(4)
            .flat_map(|c| {
                let f = f32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                let s = (f.clamp(-1.0, 1.0) * 32767.0) as i16;
                s.to_le_bytes()
            })
            .collect()),
        (S16LE, F32) => Ok(data.chunks_exact(2)
            .flat_map(|c| (i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0).to_le_bytes())
            .collect()),
        _ => Err(crate::MmlError::Compilation(format!(
            "Unsupported PCM conversion: {:?} -> {:?}", from, to
        ))),
    }
}
