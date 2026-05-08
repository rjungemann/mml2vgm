//! Sample resolver trait and implementations.
//!
//! The compiler uses a `SampleResolver` when building VGM output that
//! contains PCM data blocks (`'@ P` instruments).  The resolver abstracts
//! where the decoded f32 PCM comes from so the same compiler core works in
//! both the CLI (read WAV from disk) and the browser IDE (pre-decoded f32
//! data provided by the JS side via `AudioContext.decodeAudioData`).

use std::collections::HashMap;
use std::path::PathBuf;

/// Returns pre-decoded f32 PCM samples for a given filename.
///
/// The resolver is called once per `'@ P` instrument during code generation.
/// Return `None` if the sample is unavailable; the compiler will skip the
/// data block and (where the VGM format requires it) emit a warning.
pub trait SampleResolver: Send + Sync {
    fn resolve(&self, name: &str) -> Option<&[f32]>;
}

// ============================================================================
// MemorySampleResolver — used by WASM (JS decodes WAV, passes f32 PCM)
// ============================================================================

/// Sample resolver backed by a pre-decoded in-memory map.
///
/// Used by the WASM binding: the browser JS side calls
/// `AudioContext.decodeAudioData()` and passes the resulting `Float32Array`
/// values to `compile_with_samples` as a JSON object.  The WASM glue
/// deserialises that JSON into a `HashMap<String, Vec<f32>>` and constructs
/// a `MemorySampleResolver` to hand to the compiler.
pub struct MemorySampleResolver {
    map: HashMap<String, Vec<f32>>,
}

impl MemorySampleResolver {
    pub fn new(map: HashMap<String, Vec<f32>>) -> Self {
        Self { map }
    }
}

impl SampleResolver for MemorySampleResolver {
    fn resolve(&self, name: &str) -> Option<&[f32]> {
        // Try exact match first, then case-insensitive (Windows filenames).
        if let Some(v) = self.map.get(name) {
            return Some(v.as_slice());
        }
        let lower = name.to_ascii_lowercase();
        self.map
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == lower)
            .map(|(_, v)| v.as_slice())
    }
}

// ============================================================================
// DiskSampleResolver — used by the CLI
// ============================================================================

/// Sample resolver that reads WAV files from a base directory.
///
/// WAV decoding is not yet implemented; the resolver always returns `None`.
/// A future phase will add a WAV decoder (e.g. via the `hound` crate) so the
/// CLI can embed PCM samples without requiring a separate pre-processing step.
pub struct DiskSampleResolver {
    pub base_dir: PathBuf,
    /// Pre-loaded samples (populated lazily or up-front).
    cache: HashMap<String, Vec<f32>>,
}

impl DiskSampleResolver {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir, cache: HashMap::new() }
    }
}

impl SampleResolver for DiskSampleResolver {
    fn resolve(&self, _name: &str) -> Option<&[f32]> {
        // TODO: locate `base_dir/name`, decode WAV, cache, return slice.
        None
    }
}

// ============================================================================
// NoopSampleResolver — default when no samples are provided
// ============================================================================

/// No-op resolver; always returns `None`.
///
/// Used as the default resolver in `compile_from_source` so existing call
/// sites do not need to be updated.
pub struct NoopSampleResolver;

impl SampleResolver for NoopSampleResolver {
    fn resolve(&self, _name: &str) -> Option<&[f32]> {
        None
    }
}
