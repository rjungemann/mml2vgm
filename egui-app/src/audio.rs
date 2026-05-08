use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::time::{Duration, Instant};

pub struct AudioBuffer {
    /// Interleaved stereo f32 samples at 44100 Hz.
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

pub struct AudioEngine {
    // Must stay alive for the duration of playback.
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,

    buffer: Option<AudioBuffer>,
    /// Wall-clock instant when play was last called.
    play_started: Option<Instant>,
    /// Accumulated pause duration (for position tracking).
    paused_elapsed: Duration,
    looping: bool,
    volume: f32,

    /// Downsampled waveform (N points) for display.
    pub waveform: Vec<f32>,
    /// Separate sink for short on-screen-keyboard preview notes.
    preview_sink: Option<Sink>,
}

impl AudioEngine {
    /// Create the engine, opening the default audio output device.
    /// Returns `None` if no output device is available.
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            stream_handle: handle,
            sink: None,
            buffer: None,
            play_started: None,
            paused_elapsed: Duration::ZERO,
            looping: false,
            volume: 1.0,
            waveform: Vec::new(),
            preview_sink: None,
        })
    }

    /// Load a new PCM buffer. Stops any current playback.
    pub fn load(&mut self, buf: AudioBuffer) {
        self.stop_sink();
        self.build_waveform(&buf.samples, 512);
        self.buffer = Some(buf);
        self.paused_elapsed = Duration::ZERO;
        self.play_started = None;
    }

    pub fn play(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
                self.play_started = Some(Instant::now());
                return;
            }
            if !sink.empty() {
                return;
            }
        }
        // Start from beginning.
        self.paused_elapsed = Duration::ZERO;
        self.start_sink();
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            if !sink.is_paused() {
                sink.pause();
                if let Some(t) = self.play_started.take() {
                    self.paused_elapsed += t.elapsed();
                }
            }
        }
    }

    pub fn stop(&mut self) {
        self.stop_sink();
        self.paused_elapsed = Duration::ZERO;
        self.play_started = None;
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 2.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    pub fn set_loop(&mut self, looping: bool) {
        self.looping = looping;
    }

    pub fn is_playing(&self) -> bool {
        self.sink.as_ref().map(|s| !s.is_paused() && !s.empty()).unwrap_or(false)
    }

    pub fn has_buffer(&self) -> bool {
        self.buffer.is_some()
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn looping(&self) -> bool {
        self.looping
    }

    /// Current playback position in seconds.
    pub fn position_secs(&self) -> f64 {
        let from_play = self.play_started.map(|t| t.elapsed()).unwrap_or(Duration::ZERO);
        (self.paused_elapsed + from_play).as_secs_f64()
    }

    /// Total duration in seconds (0.0 if no buffer loaded).
    pub fn duration_secs(&self) -> f64 {
        self.buffer.as_ref().map(|b| {
            if b.channels == 0 || b.sample_rate == 0 {
                return 0.0;
            }
            b.samples.len() as f64 / (b.channels as f64 * b.sample_rate as f64)
        }).unwrap_or(0.0)
    }

    /// Poll after playback; if the sink emptied and looping is on, restart.
    pub fn tick(&mut self) {
        if self.looping {
            let finished = self.sink.as_ref().map(|s| s.empty()).unwrap_or(false);
            if finished && self.buffer.is_some() {
                self.paused_elapsed = Duration::ZERO;
                self.start_sink();
            }
        }
    }

    // ── private helpers ──────────────────────────────────────────────────────

    fn start_sink(&mut self) {
        let buf = match &self.buffer {
            Some(b) => b,
            None => return,
        };
        let sink = match Sink::try_new(&self.stream_handle) {
            Ok(s) => s,
            Err(_) => return,
        };
        sink.set_volume(self.volume);
        let source = rodio::buffer::SamplesBuffer::new(
            buf.channels,
            buf.sample_rate,
            buf.samples.clone(),
        );
        sink.append(source);
        sink.play();
        self.play_started = Some(Instant::now());
        self.sink = Some(sink);
    }

    fn stop_sink(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        if let Some(t) = self.play_started.take() {
            self.paused_elapsed += t.elapsed();
        }
    }

    /// Returns a window of left-channel samples centred at the current playback
    /// position, suitable for oscilloscope display. Values are in [-1.0, 1.0].
    pub fn scope_samples(&self, window: usize) -> Vec<f32> {
        let buf = match &self.buffer {
            Some(b) => b,
            None => return Vec::new(),
        };
        let channels = buf.channels as usize;
        if channels == 0 || buf.sample_rate == 0 {
            return Vec::new();
        }
        let total_frames = buf.samples.len() / channels;
        let pos_frames = (self.position_secs() * buf.sample_rate as f64) as usize;
        let half = window / 2;
        let start = pos_frames.saturating_sub(half).min(total_frames.saturating_sub(window));
        let end = (start + window).min(total_frames);
        (start..end)
            .map(|f| *buf.samples.get(f * channels).unwrap_or(&0.0))
            .collect()
    }

    /// Play a short PCM clip on a dedicated preview sink (stereo f32, 44100 Hz).
    /// Stops any previously playing preview first.
    pub fn play_preview(&mut self, pcm: Vec<f32>) {
        self.stop_preview();
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            let source = rodio::buffer::SamplesBuffer::new(2, 44100, pcm);
            sink.set_volume(self.volume);
            sink.append(source);
            sink.play();
            self.preview_sink = Some(sink);
        }
    }

    /// Stop any currently playing preview note immediately.
    pub fn stop_preview(&mut self) {
        if let Some(sink) = self.preview_sink.take() {
            sink.stop();
        }
    }

    fn build_waveform(&mut self, samples: &[f32], points: usize) {
        if samples.is_empty() || points == 0 {
            self.waveform = Vec::new();
            return;
        }
        let step = (samples.len() as f32 / points as f32).max(1.0);
        self.waveform = (0..points)
            .map(|i| {
                let start = (i as f32 * step) as usize;
                let end = ((start as f32 + step) as usize).min(samples.len());
                if start >= end {
                    return 0.0;
                }
                samples[start..end].iter().map(|s| s.abs()).fold(0.0f32, f32::max)
            })
            .collect();
    }
}
