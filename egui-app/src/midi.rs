use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use std::sync::mpsc::{self, Receiver, Sender};

const CLIENT_NAME: &str = "mml2vgm";

#[derive(Debug, Clone)]
pub enum MidiEvent {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    ControlChange { controller: u8, value: u8 },
}

pub struct MidiManager {
    // Active connections (owned; drop to disconnect).
    input_conn: Option<MidiInputConnection<()>>,
    output_conn: Option<MidiOutputConnection>,

    // Cached port lists (refreshed on demand).
    pub input_port_names: Vec<String>,
    pub output_port_names: Vec<String>,

    // Which port index is currently open.
    pub active_input: Option<usize>,
    pub active_output: Option<usize>,

    // Events arriving from the connected input port.
    event_tx: Sender<MidiEvent>,
    event_rx: Receiver<MidiEvent>,
}

impl MidiManager {
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let mut mgr = Self {
            input_conn: None,
            output_conn: None,
            input_port_names: Vec::new(),
            output_port_names: Vec::new(),
            active_input: None,
            active_output: None,
            event_tx,
            event_rx,
        };
        mgr.refresh_ports();
        mgr
    }

    /// Re-enumerate available ports. Call this occasionally (e.g. when the
    /// MIDI panel is shown) to pick up newly connected devices.
    pub fn refresh_ports(&mut self) {
        self.input_port_names = enumerate_input_ports();
        self.output_port_names = enumerate_output_ports();
    }

    /// Open the input port at `index`. Drops any existing input connection.
    pub fn connect_input(&mut self, index: usize) {
        self.input_conn = None; // drop old connection first
        let names = enumerate_input_port_objects();
        if index >= names.len() {
            self.active_input = None;
            return;
        }
        let tx = self.event_tx.clone();
        let midi_in = match MidiInput::new(CLIENT_NAME) {
            Ok(m) => m,
            Err(_) => { self.active_input = None; return; }
        };
        let port = &names[index];
        match midi_in.connect(port, "mml2vgm-in", move |_ts, msg, _| {
            if let Some(event) = parse_midi(msg) {
                let _ = tx.send(event);
            }
        }, ()) {
            Ok(conn) => {
                self.input_conn = Some(conn);
                self.active_input = Some(index);
            }
            Err(_) => { self.active_input = None; }
        }
    }

    /// Open the output port at `index`. Drops any existing output connection.
    pub fn connect_output(&mut self, index: usize) {
        self.output_conn = None;
        let names = enumerate_output_port_objects();
        if index >= names.len() {
            self.active_output = None;
            return;
        }
        let midi_out = match MidiOutput::new(CLIENT_NAME) {
            Ok(m) => m,
            Err(_) => { self.active_output = None; return; }
        };
        let port = &names[index];
        match midi_out.connect(port, "mml2vgm-out") {
            Ok(conn) => {
                self.output_conn = Some(conn);
                self.active_output = Some(index);
            }
            Err(_) => { self.active_output = None; }
        }
    }

    pub fn disconnect_input(&mut self) {
        self.input_conn = None;
        self.active_input = None;
    }

    pub fn disconnect_output(&mut self) {
        self.output_conn = None;
        self.active_output = None;
    }

    /// Drain all queued MIDI events from the input port.
    pub fn poll_events(&mut self) -> Vec<MidiEvent> {
        let mut out = Vec::new();
        while let Ok(ev) = self.event_rx.try_recv() {
            out.push(ev);
        }
        out
    }

    /// Send a NoteOn to the connected MIDI output port.
    pub fn send_note_on(&mut self, channel: u8, note: u8, velocity: u8) {
        if let Some(conn) = &mut self.output_conn {
            let ch = channel & 0x0F;
            let _ = conn.send(&[0x90 | ch, note & 0x7F, velocity & 0x7F]);
        }
    }

    /// Send a NoteOff to the connected MIDI output port.
    pub fn send_note_off(&mut self, channel: u8, note: u8) {
        if let Some(conn) = &mut self.output_conn {
            let ch = channel & 0x0F;
            let _ = conn.send(&[0x80 | ch, note & 0x7F, 0]);
        }
    }

    pub fn has_input(&self) -> bool { self.input_conn.is_some() }
    pub fn has_output(&self) -> bool { self.output_conn.is_some() }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_midi(msg: &[u8]) -> Option<MidiEvent> {
    if msg.is_empty() { return None; }
    let status = msg[0] & 0xF0;
    match status {
        0x90 if msg.len() >= 3 => {
            let note = msg[1];
            let vel  = msg[2];
            // NoteOn with velocity 0 is treated as NoteOff.
            if vel == 0 {
                Some(MidiEvent::NoteOff { note })
            } else {
                Some(MidiEvent::NoteOn { note, velocity: vel })
            }
        }
        0x80 if msg.len() >= 3 => Some(MidiEvent::NoteOff { note: msg[1] }),
        0xB0 if msg.len() >= 3 => Some(MidiEvent::ControlChange { controller: msg[1], value: msg[2] }),
        _ => None,
    }
}

fn enumerate_input_ports() -> Vec<String> {
    let midi_in = match MidiInput::new(CLIENT_NAME) { Ok(m) => m, Err(_) => return Vec::new() };
    midi_in.ports().iter()
        .filter_map(|p| midi_in.port_name(p).ok())
        .collect()
}

fn enumerate_output_ports() -> Vec<String> {
    let midi_out = match MidiOutput::new(CLIENT_NAME) { Ok(m) => m, Err(_) => return Vec::new() };
    midi_out.ports().iter()
        .filter_map(|p| midi_out.port_name(p).ok())
        .collect()
}

// These return port objects (not names) — needed for connect().
fn enumerate_input_port_objects() -> Vec<midir::MidiInputPort> {
    let midi_in = match MidiInput::new(CLIENT_NAME) { Ok(m) => m, Err(_) => return Vec::new() };
    midi_in.ports().into_iter().collect()
}

fn enumerate_output_port_objects() -> Vec<midir::MidiOutputPort> {
    let midi_out = match MidiOutput::new(CLIENT_NAME) { Ok(m) => m, Err(_) => return Vec::new() };
    midi_out.ports().into_iter().collect()
}
