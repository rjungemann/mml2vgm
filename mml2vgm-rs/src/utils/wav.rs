//! WAV file I/O utilities (RIFF/PCM16 stereo)

use crate::{MmlError, MmlResult};
use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

/// Write interleaved stereo f32 PCM samples to a 16-bit PCM RIFF/WAV file.
pub fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) -> MmlResult<()> {
    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * bits_per_sample as u32 / 8;
    let block_align: u16 = num_channels * bits_per_sample / 8;
    let data_size = (samples.len() * 2) as u32;
    let riff_size = 36 + data_size;

    let file = File::create(path)
        .map_err(|e| MmlError::Compilation(format!("Cannot create WAV file: {}", e)))?;
    let mut w = BufWriter::new(file);

    w.write_all(b"RIFF").map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&riff_size.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(b"WAVE").map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(b"fmt ").map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&16u32.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&1u16.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&num_channels.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&sample_rate.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&byte_rate.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&block_align.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&bits_per_sample.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(b"data").map_err(|e| MmlError::Compilation(e.to_string()))?;
    w.write_all(&data_size.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;

    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        w.write_all(&v.to_le_bytes()).map_err(|e| MmlError::Compilation(e.to_string()))?;
    }
    Ok(())
}

/// Read a RIFF/PCM16 WAV file and return interleaved f32 samples.
pub fn read_wav(path: &Path) -> MmlResult<Vec<f32>> {
    let mut file = File::open(path)
        .map_err(|e| MmlError::Compilation(format!("Cannot open WAV file: {}", e)))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| MmlError::Compilation(e.to_string()))?;

    if data.len() < 44 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err(MmlError::Compilation("Not a valid RIFF/WAV file".to_string()));
    }

    let mut pos = 12usize;
    let mut pcm_start = 0usize;
    let mut pcm_len = 0usize;
    while pos + 8 <= data.len() {
        let tag = &data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([data[pos+4], data[pos+5], data[pos+6], data[pos+7]]) as usize;
        pos += 8;
        if tag == b"data" {
            pcm_start = pos;
            pcm_len = chunk_size.min(data.len() - pos);
            break;
        }
        pos += chunk_size;
    }

    if pcm_len == 0 {
        return Err(MmlError::Compilation("WAV file has no data chunk".to_string()));
    }

    Ok(data[pcm_start..pcm_start + pcm_len]
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
        .collect())
}
