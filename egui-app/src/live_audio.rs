//! Real-time audio engine for on-screen keyboard / MIDI live preview.
//!
//! [`LiveAudioEngine`] keeps a continuous rodio sink fed by a custom [`LiveSource`]
//! that pulls audio from a `LivePlayer` wrapped in `Arc<Mutex<...>>`.
//!
//! `note_on` / `note_off` / `load_source` all lock the mutex from the GUI thread;
//! the audio thread calls `try_lock` in the source iterator and produces silence
//! if the lock is momentarily held.

use mml2vgm::live_player::LivePlayer;
use rodio::{OutputStream, Sink, Source};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Stereo frames per generate_samples call (64 frames ≈ 1.45 ms at 44100 Hz).
const CHUNK_FRAMES: usize = 64;
const CHUNK_SAMPLES: usize = CHUNK_FRAMES * 2; // stereo

// ── custom Source ────────────────────────────────────────────────────────────

struct LiveSource {
    player: Arc<Mutex<Option<LivePlayer>>>,
    chunk: Vec<f32>,
    pos: usize,
}

impl LiveSource {
    fn new(player: Arc<Mutex<Option<LivePlayer>>>) -> Self {
        Self {
            player,
            chunk: vec![0.0f32; CHUNK_SAMPLES],
            pos: CHUNK_SAMPLES, // triggers immediate refill on first pull
        }
    }
}

impl Iterator for LiveSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.chunk.len() {
            self.chunk.fill(0.0);
            // Use try_lock: if the GUI thread holds the lock, output silence this frame.
            if let Ok(mut guard) = self.player.try_lock() {
                if let Some(p) = guard.as_mut() {
                    p.generate_samples(&mut self.chunk, 44100);
                }
            }
            self.pos = 0;
        }
        let s = self.chunk[self.pos];
        self.pos += 1;
        Some(s)
    }
}

impl Source for LiveSource {
    fn current_frame_len(&self) -> Option<usize> {
        None // variable-length frames
    }
    fn channels(&self) -> u16 {
        2
    }
    fn sample_rate(&self) -> u32 {
        44100
    }
    fn total_duration(&self) -> Option<Duration> {
        None // infinite
    }
}

// ── LiveAudioEngine ──────────────────────────────────────────────────────────

/// Continuous-output audio engine for real-time chip-register note playback.
///
/// Owns a rodio `Sink` fed by a `LiveSource` that pulls from an
/// `Arc<Mutex<Option<LivePlayer>>>`.  The engine is created once; the player
/// is replaced on each successful compile via `load_source`.
pub struct LiveAudioEngine {
    player: Arc<Mutex<Option<LivePlayer>>>,
    // Keep the stream alive for the engine's lifetime.
    _stream: OutputStream,
    // The sink stays alive and continuously drains the LiveSource.
    _sink: Sink,
}

impl LiveAudioEngine {
    /// Open the default audio output device and start the live playback sink.
    ///
    /// Returns `None` if no audio output device is available.
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        let player: Arc<Mutex<Option<LivePlayer>>> = Arc::new(Mutex::new(None));
        let sink = Sink::try_new(&handle).ok()?;
        let source = LiveSource::new(player.clone());
        sink.append(source);
        sink.play();
        Some(Self { player, _stream: stream, _sink: sink })
    }

    /// Replace the active `LivePlayer` with one built from `source`.
    ///
    /// Called after each successful compile to load up-to-date instrument
    /// definitions and channel assignments. Any note currently playing is
    /// stopped when the player is replaced.
    pub fn load_source(&self, source: &str) {
        if let Ok(mut guard) = self.player.lock() {
            *guard = LivePlayer::from_source(source, 44100).ok();
        }
    }

    /// Trigger note-on on `channel` (e.g. `"A1"`) with the given MIDI note and velocity.
    pub fn note_on(&self, channel: &str, midi_note: u8, velocity: u8) {
        if let Ok(mut guard) = self.player.lock() {
            if let Some(p) = guard.as_mut() {
                p.note_on(channel, midi_note, velocity);
            }
        }
    }

    /// Trigger note-off on `channel`.
    pub fn note_off(&self, channel: &str) {
        if let Ok(mut guard) = self.player.lock() {
            if let Some(p) = guard.as_mut() {
                p.note_off(channel);
            }
        }
    }
}
