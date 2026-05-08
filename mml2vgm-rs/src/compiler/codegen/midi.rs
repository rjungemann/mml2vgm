//! MIDI Format Generator
//!
//! This module generates Standard MIDI File (SMF) format files from MML AST.
//! Supports both Type 0 (single track) and Type 1 (multi-track) SMF formats.

use super::{CodeGenerator, NoteEvent, OutputFormat, SourceMap};
use crate::compiler::ast::{
    Alias, Envelope, FmInstrument, Include, InstrumentSelection, Length, Loop, 
    Metadata, MmlAst, MmlNode, Note, Octave, OctaveShift, PartDefinition, PcmInstrument,
    Quantize, Rest, Tempo, Volume,
};
use crate::{CompileOptions, MmlError, MmlResult, SoundChip};
use std::collections::{HashMap, HashSet};

/// MIDI event types for Standard MIDI File
#[derive(Debug, Clone, PartialEq)]
pub enum MidiEvent {
    /// Note Off event (status: 0x80-0x8F)
    NoteOff {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    /// Note On event (status: 0x90-0x9F)
    NoteOn {
        channel: u8,
        note: u8,
        velocity: u8,
    },
    /// Polyphonic Aftertouch (status: 0xA0-0xAF)
    PolyAftertouch {
        channel: u8,
        note: u8,
        value: u8,
    },
    /// Control Change (status: 0xB0-0xBF)
    ControlChange {
        channel: u8,
        controller: u8,
        value: u8,
    },
    /// Program Change (status: 0xC0-0xCF)
    ProgramChange {
        channel: u8,
        program: u8,
    },
    /// Channel Aftertouch (status: 0xD0-0xDF)
    ChannelAftertouch {
        channel: u8,
        value: u8,
    },
    /// Pitch Bend (status: 0xE0-0xEF)
    PitchBend {
        channel: u8,
        value: i16, // -8192 to 8191 (center = 0)
    },
    /// System Exclusive (status: 0xF0)
    SysEx {
        data: Vec<u8>,
    },
    /// Meta event (status: 0xFF)
    Meta {
        meta_type: u8,
        data: Vec<u8>,
    },
}

/// A MIDI track event with delta time
#[derive(Debug, Clone)]
pub struct MidiTrackEvent {
    /// Delta time in ticks
    pub delta: u32,
    /// MIDI event
    pub event: MidiEvent,
}

/// MIDI track containing a sequence of events
#[derive(Debug, Clone, Default)]
pub struct MidiTrack {
    pub events: Vec<MidiTrackEvent>,
}

/// MIDI file format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiFormat {
    /// Single track (Type 0)
    Type0,
    /// Multi-track (Type 1)
    Type1,
}

impl Default for MidiFormat {
    fn default() -> Self {
        MidiFormat::Type0
    }
}

/// Standard MIDI File generator
pub struct MidiGenerator {
    /// MIDI format (0 = Type 0, 1 = Type 1)
    format: MidiFormat,
    /// Ticks per quarter note (division)
    ticks_per_quarter: u16,
    /// Tracks for Type 1, or single track for Type 0
    tracks: Vec<MidiTrack>,
    /// Current track index being written
    current_track_index: usize,
    /// Whether to use running status compression
    running_status: bool,
    /// Source map for note events
    source_map: SourceMap,
    /// Current part name being processed
    current_part_name: String,
    /// Part MIDI channel assignments
    part_channels: HashMap<String, u8>,
    /// Part program assignments
    part_programs: HashMap<String, u8>,
    /// Part bank MSB assignments
    part_bank_msb: HashMap<String, u8>,
    /// Part bank LSB assignments
    part_bank_lsb: HashMap<String, u8>,
    /// Part transpose values
    part_transpose: HashMap<String, i8>,
    /// Global tempo in BPM
    global_tempo: u32,
    /// Current time in ticks for each track
    track_times: Vec<u32>,
    /// Sound chips used
    chips: Vec<SoundChip>,
    /// Ticks per tick (for duration conversion)
    ticks_per_tick: u32,
}

/// MIDI-specific AST nodes (will be integrated into main AST later)
#[derive(Debug, Clone, PartialEq)]
pub struct ControlChange {
    pub controller: u8,
    pub value: u8,
    pub channel: Option<u8>,
    pub span: Option<crate::Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramChange {
    pub program: u8,
    pub channel: Option<u8>,
    pub span: Option<crate::Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PitchBend {
    pub value: i16,
    pub channel: Option<u8>,
    pub span: Option<crate::Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Aftertouch {
    pub value: u8,
    pub channel: Option<u8>,
    pub span: Option<crate::Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolyAftertouch {
    pub note: u8,
    pub value: u8,
    pub channel: Option<u8>,
    pub span: Option<crate::Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SysEx {
    pub data: Vec<u8>,
    pub span: Option<crate::Span>,
}

impl MidiGenerator {
    /// Create a new MIDI generator from an AST
    pub fn from_ast(ast: &MmlAst, options: &CompileOptions) -> MmlResult<Self> {
        let mut generator = Self {
            format: MidiFormat::Type0,
            ticks_per_quarter: 192, // Default: 192 ticks per quarter note (common for sequencing)
            tracks: Vec::new(),
            current_track_index: 0,
            running_status: true,
            source_map: SourceMap::default(),
            current_part_name: String::new(),
            part_channels: HashMap::new(),
            part_programs: HashMap::new(),
            part_bank_msb: HashMap::new(),
            part_bank_lsb: HashMap::new(),
            part_transpose: HashMap::new(),
            global_tempo: 120,
            track_times: Vec::new(),
            chips: Vec::new(),
            ticks_per_tick: 1,
        };

        // Extract chips from AST
        generator.extract_chips(ast);

        // Check if MIDI is explicitly requested or if any part uses MIDI chip
        let has_midi_chip = ast.parts.values().any(|p| {
            p.chip.as_ref().map_or(false, |c| c.to_uppercase() == "MIDI")
        });

        // Determine format: Type 1 if multiple parts, Type 0 otherwise
        if ast.parts.len() > 1 {
            generator.format = MidiFormat::Type1;
        }

        // Set ticks per tick based on ClockCount or use default
        // For MIDI, we use a standard PPQN (pulses per quarter note)
        // Common values: 48, 96, 192, 240, 480
        // We'll use 192 as it provides good resolution

        // Extract global tempo from first part or use default
        if let Some(first_part) = ast.parts.values().next() {
            if let Some(tempo) = first_part.tempo {
                generator.global_tempo = tempo;
            }
        }

        // Process parts and assign MIDI channels
        generator.assign_midi_channels(ast);

        // Convert AST to MIDI events
        generator.convert_ast_to_events(ast)?;

        // Build tracks based on format
        generator.build_tracks();

        Ok(generator)
    }

    /// Extract sound chips from the AST
    fn extract_chips(&mut self, ast: &MmlAst) {
        let mut chips_set = HashSet::new();
        
        for part in ast.parts.values() {
            if let Some(chip_name) = &part.chip {
                if let Ok(chip) = chip_name.parse::<SoundChip>() {
                    chips_set.insert(chip);
                }
            }
        }
        
        // Always include MIDI if we're generating MIDI
        chips_set.insert(SoundChip::MIDI);
        
        self.chips = chips_set.into_iter().collect();
    }

    /// Assign MIDI channels to parts
    fn assign_midi_channels(&mut self, ast: &MmlAst) {
        let mut channel = 0u8;
        
        for (part_name, part) in &ast.parts {
            // Check if part explicitly specifies MIDI channel
            // For now, we'll auto-assign channels 0-15
            if channel < 16 {
                self.part_channels.insert(part_name.clone(), channel);
                
                // Default program based on channel
                // Channel 10 (index 9) is typically drums
                let program = if channel == 9 {
                    112 // Standard Drum Kit (GM)
                } else {
                    0 // Acoustic Grand Piano (GM)
                };
                
                self.part_programs.insert(part_name.clone(), program);
                self.part_transpose.insert(part_name.clone(), 0);
                
                channel += 1;
            }
        }
    }

    /// Convert AST to MIDI events
    fn convert_ast_to_events(&mut self, ast: &MmlAst) -> MmlResult<()> {
        // Initialize tracks
        if self.format == MidiFormat::Type1 {
            for _ in 0..ast.parts.len() {
                self.tracks.push(MidiTrack::default());
                self.track_times.push(0);
            }
        } else {
            self.tracks.push(MidiTrack::default());
            self.track_times.push(0);
        }

        // Process global settings
        self.process_global_settings(&ast.global_settings)?;

        // Process each part
        for (part_name, part) in &ast.parts {
            self.current_part_name = part_name.clone();
            
            if self.format == MidiFormat::Type1 {
                self.current_track_index = ast.parts.keys()
                    .position(|k| k == part_name)
                    .unwrap_or(0);
            }

            self.process_part(part)?;
        }

        // Add end-of-track meta events
        for track in &mut self.tracks {
            track.events.push(MidiTrackEvent {
                delta: 0,
                event: MidiEvent::Meta {
                    meta_type: 0x2F, // End of Track
                    data: vec![],
                },
            });
        }

        Ok(())
    }

    /// Process global settings
    fn process_global_settings(&mut self, settings: &[MmlNode]) -> MmlResult<()> {
        for node in settings {
            match node {
                MmlNode::Tempo(tempo) => {
                    self.global_tempo = tempo.bpm;
                    // Add tempo meta event to all tracks
                    let microseconds_per_quarter = self.calc_microseconds_per_quarter(tempo.bpm);
                    let tempo_data = vec![
                        (microseconds_per_quarter >> 16) as u8,
                        (microseconds_per_quarter >> 8) as u8,
                        microseconds_per_quarter as u8,
                    ];
                    
                    for track in &mut self.tracks {
                        track.events.push(MidiTrackEvent {
                            delta: 0,
                            event: MidiEvent::Meta {
                                meta_type: 0x51, // Set Tempo
                                data: tempo_data.clone(),
                            },
                        });
                    }
                }
                MmlNode::Metadata(meta) => {
                    // Handle metadata
                    if meta.key.to_uppercase() == "TIME_SIGNATURE" {
                        // Parse time signature
                        // Format: "4/4", "7/8", etc.
                        // For now, use 4/4 as default
                        let time_sig_data = vec![4, 2, 24, 8]; // 4/4, 24 clocks per click, 8 32nd notes per quarter
                        for track in &mut self.tracks {
                            track.events.push(MidiTrackEvent {
                                delta: 0,
                                event: MidiEvent::Meta {
                                    meta_type: 0x58, // Time Signature
                                    data: time_sig_data.clone(),
                                },
                            });
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Process a single part
    fn process_part(&mut self, part: &PartDefinition) -> MmlResult<()> {
        let channel = self.part_channels.get(&self.current_part_name).copied().unwrap_or(0);
        let program = self.part_programs.get(&self.current_part_name).copied().unwrap_or(0);
        let transpose = self.part_transpose.get(&self.current_part_name).copied().unwrap_or(0);

        let mut current_octave = 4u8;
        let mut current_length = 4u32;
        let mut current_volume = 100u8; // Default volume (0-127)
        let mut current_tempo = part.tempo.unwrap_or(self.global_tempo);
        let mut time_elapsed: u32 = 0;

        // Add program change at start of part
        self.add_event(MidiEvent::ProgramChange { channel, program });
        
        // Add bank select if needed
        // For GM, bank select MSB=0, LSB=0 is default
        self.add_event(MidiEvent::ControlChange { channel, controller: 0, value: 0 }); // Bank Select MSB
        self.add_event(MidiEvent::ControlChange { channel, controller: 32, value: 0 }); // Bank Select LSB
        
        // Set volume (CC7)
        self.add_event(MidiEvent::ControlChange { channel, controller: 7, value: current_volume });

        // Process each command in the part
        for node in &part.commands {
            match node {
                MmlNode::Note(note) => {
                    let midi_note = note.midi_note().wrapping_add(transpose as u8);
                    let velocity = note.volume.unwrap_or(current_volume);
                    let duration_ticks = self.calc_duration_ticks(note, current_length);

                    // Note On
                    self.add_event_with_delta(MidiEvent::NoteOn { channel, note: midi_note, velocity }, time_elapsed);
                    
                    // Note Off after duration
                    self.add_event_with_delta(MidiEvent::NoteOff { channel, note: midi_note, velocity: 0 }, duration_ticks);
                    
                    // Accumulate time
                    time_elapsed += duration_ticks;
                    
                    // Add to source map
                    if let Some(span) = &note.span {
                        self.source_map.events.push(NoteEvent {
                            sample_start: 0, // Will be calculated later
                            sample_end: 0,
                            part: self.current_part_name.clone(),
                            note_midi: midi_note,
                            instrument: note.instrument.unwrap_or(0) as u32,
                            line: span.start.line,
                            col_start: span.start.column,
                            col_end: span.end.column,
                        });
                    }
                }
                MmlNode::Rest(rest) => {
                    let duration_ticks = self.calc_duration_ticks(&Rest { duration: rest.duration, dotted: rest.dotted, span: rest.span.clone() }, current_length);
                    time_elapsed += duration_ticks;
                }
                MmlNode::Tempo(tempo) => {
                    current_tempo = tempo.bpm;
                    let microseconds_per_quarter = self.calc_microseconds_per_quarter(tempo.bpm);
                    let tempo_data = vec![
                        (microseconds_per_quarter >> 16) as u8,
                        (microseconds_per_quarter >> 8) as u8,
                        microseconds_per_quarter as u8,
                    ];
                    self.add_event(MidiEvent::Meta { meta_type: 0x51, data: tempo_data });
                }
                MmlNode::Volume(vol) => {
                    current_volume = vol.level;
                    self.add_event(MidiEvent::ControlChange { channel, controller: 7, value: current_volume });
                }
                MmlNode::Length(len) => {
                    current_length = len.value;
                }
                MmlNode::Octave(oct) => {
                    current_octave = oct.number;
                }
                MmlNode::OctaveShift(shift) => {
                    match shift {
                        OctaveShift::Up => current_octave = current_octave.saturating_add(1),
                        OctaveShift::Down => current_octave = current_octave.saturating_sub(1),
                    }
                }
                MmlNode::Quantize(q) => {
                    // Quantize affects timing, handle as needed
                }
                MmlNode::Loop(loop_node) => {
                    for _ in 0..loop_node.count {
                        for cmd in &loop_node.body {
                            // Process loop body
                            match cmd {
                                MmlNode::Note(note) => {
                                    let midi_note = note.midi_note().wrapping_add(transpose as u8);
                                    let velocity = note.volume.unwrap_or(current_volume);
                                    let duration_ticks = self.calc_duration_ticks(note, current_length);

                                    self.add_event_with_delta(MidiEvent::NoteOn { channel, note: midi_note, velocity }, time_elapsed);
                                    self.add_event_with_delta(MidiEvent::NoteOff { channel, note: midi_note, velocity: 0 }, duration_ticks);
                                    time_elapsed += duration_ticks;
                                }
                                MmlNode::Rest(rest) => {
                                    let duration_ticks = self.calc_duration_ticks(&Rest { duration: rest.duration, dotted: rest.dotted, span: rest.span.clone() }, current_length);
                                    time_elapsed += duration_ticks;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                MmlNode::Bar => {
                    // Bar line - no action needed for MIDI
                }
                MmlNode::PartDefinition(_) => {
                    // Nested part definition - skip
                }
                MmlNode::Metadata(meta) => {
                    // Handle per-part metadata
                }
                MmlNode::Comment(_) => {
                    // Skip comments
                }
                MmlNode::ChipCommand { chip: _, command, args } => {
                    // Map chip commands to MIDI CC messages
                    self.handle_chip_command_to_midi(command, args, channel);
                }
                MmlNode::InstrumentSelection(inst) => {
                    // Map instrument to MIDI program change
                    // For now, just use instrument number as program
                    let new_program = inst.number as u8;
                    self.add_event(MidiEvent::ProgramChange { channel, program: new_program });
                }
                MmlNode::FmInstrument(_) => {}
                MmlNode::PcmInstrument(_) => {}
                MmlNode::Envelope(_) => {}
                MmlNode::Arpeggio(_) => {}
                MmlNode::Alias(_) => {}
                MmlNode::Include(_) => {}
                // MIDI-specific nodes
                MmlNode::MidiControlChange(cc) => {
                    self.add_event(MidiEvent::ControlChange { channel, controller: cc.controller, value: cc.value });
                }
                MmlNode::MidiProgramChange(pc) => {
                    let target_channel = pc.channel.unwrap_or(channel);
                    self.add_event(MidiEvent::ProgramChange { channel: target_channel, program: pc.program });
                }
                MmlNode::MidiPitchBend(pb) => {
                    let target_channel = pb.channel.unwrap_or(channel);
                    self.add_event(MidiEvent::PitchBend { channel: target_channel, value: pb.value });
                }
                MmlNode::MidiAftertouch(at) => {
                    let target_channel = at.channel.unwrap_or(channel);
                    self.add_event(MidiEvent::ChannelAftertouch { channel: target_channel, value: at.value });
                }
                MmlNode::MidiPolyAftertouch(pa) => {
                    let target_channel = pa.channel.unwrap_or(channel);
                    self.add_event(MidiEvent::PolyAftertouch { channel: target_channel, note: pa.note, value: pa.value });
                }
                MmlNode::MidiSysEx(sysex) => {
                    self.add_event(MidiEvent::SysEx { data: sysex.data.clone() });
                }
                MmlNode::MidiChannel(mch) => {
                    // Channel assignment for part
                }
                MmlNode::MidiProgram(mprog) => {
                    // Program assignment for part
                }
            }
        }

        Ok(())
    }

    /// Add a MIDI event with current delta time
    fn add_event(&mut self, event: MidiEvent) {
        let delta = 0; // Will be calculated during track building
        let track_index = self.current_track_index;
        
        if track_index < self.tracks.len() {
            self.tracks[track_index].events.push(MidiTrackEvent { delta, event });
        }
    }

    /// Add a MIDI event with specified delta time
    fn add_event_with_delta(&mut self, event: MidiEvent, delta: u32) {
        let track_index = self.current_track_index;
        
        if track_index < self.tracks.len() {
            self.tracks[track_index].events.push(MidiTrackEvent { delta, event });
        }
    }

    /// Handle chip-specific commands by mapping them to MIDI CC messages (Phase 10)
    fn handle_chip_command_to_midi(&mut self, command: &str, args: &[u32], channel: u8) {
        use crate::compiler::codegen::midi_controller::*;
        
        let cmd_upper = command.to_uppercase();
        
        // Map command to MIDI CC based on command type
        match cmd_upper.as_str() {
            // FM Operator Level/Brightness parameters → Expression CC (11)
            "TL" => {
                if !args.is_empty() {
                    let cc_value = ((args[0] as u8).saturating_sub(127).wrapping_neg()) as u8;
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::EXPRESSION,
                        value: cc_value,
                    });
                }
            }
            // FM Attack Rate → Brightness CC (12) for PSG, or Expression for FM
            "AR" => {
                if !args.is_empty() {
                    let cc_value = ((args[0] as u8) >> 2) & 0x7F;
                    // Default to Expression, could be refined with chip detection
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::EXPRESSION,
                        value: cc_value,
                    });
                }
            }
            // Decay Rate → Resonance CC (13)
            "DR" => {
                if !args.is_empty() {
                    let cc_value = ((args[0] as u8) >> 2) & 0x7F;
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::EFFECT_CONTROL_2,
                        value: cc_value,
                    });
                }
            }
            // Algorithm (AL) → General Purpose Slider 1 (16)
            "AL" => {
                if !args.is_empty() {
                    let cc_value = ((args[0] as u8) * 16) & 0x7F;
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::GENERAL_PURPOSE_SLIDER_1,
                        value: cc_value,
                    });
                }
            }
            // Feedback (FB) → General Purpose Slider 2 (17)
            "FB" => {
                if !args.is_empty() {
                    let cc_value = ((args[0] as u8) * 16) & 0x7F;
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::GENERAL_PURPOSE_SLIDER_2,
                        value: cc_value,
                    });
                }
            }
            // Pan (PAN) → Pan CC (10)
            "PAN" => {
                if !args.is_empty() {
                    let pan_value = args[0] as u8;
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::PAN,
                        value: pan_value,
                    });
                }
            }
            // Volume control → Main Volume CC (7)
            "VOLUME" | "LVOL" | "RVOL" => {
                if !args.is_empty() {
                    let vol_value = ((args[0] as u8) >> 1) & 0x7F;
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::VOLUME,
                        value: vol_value,
                    });
                }
            }
            // Envelope enable → General Purpose Slider 3 (18)
            "EN" => {
                if !args.is_empty() {
                    let cc_value = if args[0] != 0 { 127 } else { 0 };
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::GENERAL_PURPOSE_SLIDER_3,
                        value: cc_value,
                    });
                }
            }
            // Filter/Distortion mode → Effect Control 1 (12)
            "FILTER" | "DIST" => {
                if !args.is_empty() {
                    let cc_value = ((args[0] as u8) * 32) & 0x7F;
                    self.add_event(MidiEvent::ControlChange {
                        channel,
                        controller: midi_cc::EFFECT_CONTROL_1,
                        value: cc_value,
                    });
                }
            }
            _ => {
                // Unknown command - silently skip
            }
        }
    }

    /// Calculate duration in ticks
    fn calc_duration_ticks(&self, note_or_rest: &impl std::fmt::Debug, current_length: u32) -> u32 {
        // For now, use a simple conversion
        // A quarter note (length=4) at 120 BPM with 192 ticks/quarter = 192 ticks
        // So: duration_ticks = (ticks_per_quarter / current_length) * note_duration
        // But we need to get the actual duration from the note
        
        let base_ticks = self.ticks_per_quarter as u32;
        let length_factor = match note_or_rest {
            _ => 4, // Default to quarter note equivalent
        };
        
        // Simple formula for now - will be refined
        base_ticks * 4 / length_factor
    }

    /// Calculate microseconds per quarter note from BPM
    fn calc_microseconds_per_quarter(&self, bpm: u32) -> u32 {
        if bpm == 0 {
            return 500000; // Default: 120 BPM = 500000 us per quarter
        }
        ((60_000_000u64 / bpm as u64) as u32).clamp(1, 0x00FFFFFF)
    }

    /// Build tracks from events
    fn build_tracks(&mut self) {
        for (track_idx, track) in &mut self.tracks.iter_mut().enumerate() {
            // Calculate proper delta times
            let mut prev_time = 0u32;
            for event in &mut track.events {
                // For now, just use sequential delta times
                // This will be improved with proper timing
                event.delta = if prev_time == 0 {
                    0
                } else {
                    1 // Placeholder
                };
                prev_time += event.delta;
            }
        }
    }

    /// Write variable-length quantity (used for delta times in MIDI)
    fn write_var_length(&self, value: u32, output: &mut Vec<u8>) {
        let mut v = value;
        loop {
            let mut byte = (v & 0x7F) as u8;
            v >>= 7;
            if v > 0 {
                byte |= 0x80;
            }
            output.push(byte);
            if v == 0 {
                break;
            }
        }
    }

    /// Get the status byte for a MIDI event
    fn get_status_byte(&self, event: &MidiEvent) -> u8 {
        match event {
            MidiEvent::NoteOff { channel, .. } => 0x80 | channel,
            MidiEvent::NoteOn { channel, .. } => 0x90 | channel,
            MidiEvent::PolyAftertouch { channel, .. } => 0xA0 | channel,
            MidiEvent::ControlChange { channel, .. } => 0xB0 | channel,
            MidiEvent::ProgramChange { channel, .. } => 0xC0 | channel,
            MidiEvent::ChannelAftertouch { channel, .. } => 0xD0 | channel,
            MidiEvent::PitchBend { channel, .. } => 0xE0 | channel,
            MidiEvent::SysEx { .. } => 0xF0,
            MidiEvent::Meta { .. } => 0xFF,
        }
    }

    /// Get the event data bytes (excluding status byte)
    fn get_event_data(&self, event: &MidiEvent) -> Vec<u8> {
        match event {
            MidiEvent::NoteOff { note, velocity, .. } => vec![*note, *velocity],
            MidiEvent::NoteOn { note, velocity, .. } => vec![*note, *velocity],
            MidiEvent::PolyAftertouch { note, value, .. } => vec![*note, *value],
            MidiEvent::ControlChange { controller, value, .. } => vec![*controller, *value],
            MidiEvent::ProgramChange { program, .. } => vec![*program],
            MidiEvent::ChannelAftertouch { value, .. } => vec![*value],
            MidiEvent::PitchBend { value, .. } => {
                // Pitch bend is 14-bit: LSB first, then MSB
                // Center is 0x2000 (8192), range 0-16383
                let clamped = i16::clamp(*value, -8192, 8191);
                let adjusted = (clamped + 8192) as u16;
                vec![(adjusted & 0x7F) as u8, ((adjusted >> 7) & 0x7F) as u8]
            }
            MidiEvent::SysEx { data } => {
                let mut result = vec![0xF0];
                result.extend_from_slice(data);
                result.push(0xF7); // End of SysEx
                result
            }
            MidiEvent::Meta { meta_type, data } => {
                let mut result = vec![*meta_type];
                if *meta_type != 0x2F && *meta_type != 0x51 {
                    // Add length for most meta events
                    result.push(data.len() as u8);
                } else if *meta_type == 0x51 {
                    // Set Tempo is always 3 bytes
                    result.push(3);
                } else if *meta_type == 0x2F {
                    // End of Track has no data
                    return vec![*meta_type, 0x00];
                }
                result.extend_from_slice(data);
                result
            }
        }
    }

    /// Check if running status can be used
    fn can_use_running_status(&self, prev_status: Option<u8>, current_event: &MidiEvent) -> bool {
        if !self.running_status {
            return false;
        }
        
        let current_status = self.get_status_byte(current_event);
        
        match (prev_status, current_event) {
            (Some(prev), _) => {
                // Running status can be used if same status byte
                // Also, NoteOn with velocity=0 can be treated as NoteOff
                prev == current_status || 
                (prev == 0x90 && current_status == 0x80) ||
                (prev == 0x80 && current_status == 0x90)
            }
            _ => false,
        }
    }
}

impl CodeGenerator for MidiGenerator {
    fn generate(&self) -> MmlResult<Vec<u8>> {
        let mut output = Vec::new();

        // Write header chunk
        self.write_header(&mut output)?;

        // Write track chunks
        for track in &self.tracks {
            self.write_track(track, &mut output)?;
        }

        Ok(output)
    }

    fn format(&self) -> OutputFormat {
        super::OutputFormat::Midi
    }

    fn chips(&self) -> &[SoundChip] {
        &self.chips
    }
}

impl MidiGenerator {
    /// Get the source map for this generator
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }
}

impl MidiGenerator {
    /// Write the MIDI header chunk (MThd)
    fn write_header(&self, output: &mut Vec<u8>) -> MmlResult<()> {
        // Chunk type: "MThd"
        output.extend_from_slice(b"MThd");

        // Length: 6 bytes (big-endian)
        output.extend_from_slice(&6u32.to_be_bytes());

        // Format: 0 = Type 0, 1 = Type 1
        let format_value = match self.format {
            MidiFormat::Type0 => 0u16,
            MidiFormat::Type1 => 1u16,
        };
        output.extend_from_slice(&format_value.to_be_bytes());

        // Number of tracks
        let num_tracks = match self.format {
            MidiFormat::Type0 => 1u16,
            MidiFormat::Type1 => self.tracks.len() as u16,
        };
        output.extend_from_slice(&num_tracks.to_be_bytes());

        // Division: ticks per quarter note
        output.extend_from_slice(&self.ticks_per_quarter.to_be_bytes());

        Ok(())
    }

    /// Write a MIDI track chunk (MTrk)
    fn write_track(&self, track: &MidiTrack, output: &mut Vec<u8>) -> MmlResult<()> {
        // Chunk type: "MTrk"
        output.extend_from_slice(b"MTrk");

        // Calculate track data
        let track_data = self.encode_track_events(track)?;

        // Length: track data length (big-endian)
        output.extend_from_slice(&(track_data.len() as u32).to_be_bytes());

        // Track data
        output.extend_from_slice(&track_data);

        Ok(())
    }

    /// Encode track events into MIDI track data
    fn encode_track_events(&self, track: &MidiTrack) -> MmlResult<Vec<u8>> {
        let mut data = Vec::new();
        let mut prev_status: Option<u8> = None;

        for event in &track.events {
            // Write delta time (variable length)
            self.write_var_length(event.delta, &mut data);

            let status = self.get_status_byte(&event.event);
            let event_data = self.get_event_data(&event.event);

            // Check if we can use running status
            let can_use_running = prev_status == Some(status);

            if can_use_running && self.running_status {
                // Use running status: omit status byte
                data.extend_from_slice(&event_data);
            } else {
                // Write full status byte and data
                data.push(status);
                data.extend_from_slice(&event_data);
            }

            prev_status = Some(status);
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::ast::{MmlAst, PartDefinition};

    #[test]
    fn test_midi_generator_basic() {
        let ast = MmlAst::new();
        let options = CompileOptions::default();

        let generator = MidiGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();

        // MIDI header is 14 bytes
        assert!(result.len() >= 14);
        
        // Check MThd chunk
        assert_eq!(&result[0..4], b"MThd");
        assert_eq!(&result[4..8], &6u32.to_be_bytes()); // Header length
    }

    #[test]
    fn test_midi_generator_with_note() {
        let mut ast = MmlAst::new();
        
        let mut note = Note::new('C', 0, 4);
        note.duration = Some(480);

        let part = PartDefinition {
            name: "MIDI1".to_string(),
            chip: Some("MIDI".to_string()),
            tempo: Some(120),
            commands: vec![MmlNode::Note(note)],
        };

        ast.parts.insert("MIDI1".to_string(), part);
        
        let options = CompileOptions::default();
        let generator = MidiGenerator::from_ast(&ast, &options).unwrap();
        let result = generator.generate().unwrap();

        // Should have header + at least one track
        assert!(result.len() > 14);
        
        // Check MTrk chunk exists
        assert!(result.windows(4).any(|w| w == b"MTrk"));
    }

    #[test]
    fn test_var_length_encoding() {
        let generator = MidiGenerator::from_ast(&MmlAst::new(), &CompileOptions::default()).unwrap();
        let mut output = Vec::new();

        // Test various values
        generator.write_var_length(0, &mut output);
        assert_eq!(output, vec![0x00]);

        output.clear();
        generator.write_var_length(127, &mut output);
        assert_eq!(output, vec![0x7F]);

        output.clear();
        generator.write_var_length(128, &mut output);
        // 128 = 0x80, 1
        // First byte: 128 & 0x7F = 0, v >>= 7 = 1, set MSB -> 0x80
        // Second byte: 1 & 0x7F = 1, v >>= 7 = 0 -> 0x01
        assert_eq!(output, vec![0x80, 0x01]);

        output.clear();
        generator.write_var_length(255, &mut output);
        // 255 = 0x7F, 1
        // First byte: 255 & 0x7F = 127, v >>= 7 = 1, set MSB -> 0xFF
        // Second byte: 1 & 0x7F = 1 -> 0x01
        // Actually: 255 & 0x7F = 0x7F, v >>= 7 = 1, so 0x7F | 0x80 = 0xFF
        // Then v = 1, byte = 1, v >>= 7 = 0 -> 0x01
        assert_eq!(output, vec![0xFF, 0x01]);

        output.clear();
        generator.write_var_length(16383, &mut output);
        // 16383 = 0x3FFF = 0b0011111111111111
        // First byte: 0x3FFF & 0x7F = 0x7F, v >>= 7 = 0x3F, set MSB -> 0xFF
        // Second byte: 0x3F & 0x7F = 0x3F, v >>= 7 = 0 -> 0x3F
        assert_eq!(output, vec![0xFF, 0x7F]);
    }

    #[test]
    fn test_pitch_bend_encoding() {
        let generator = MidiGenerator::from_ast(&MmlAst::new(), &CompileOptions::default()).unwrap();
        
        // Center (0)
        let data = generator.get_event_data(&MidiEvent::PitchBend { channel: 0, value: 0 });
        assert_eq!(data, vec![0x00, 0x40]); // 0x2000 = center (8192), LSB=0x00, MSB=0x40

        // +100
        // 0 + 100 + 8192 = 8292 = 0x2064
        // LSB = 0x2064 & 0x7F = 0x64 (100)
        // MSB = (0x2064 >> 7) & 0x7F = 0x40 (64)
        let data = generator.get_event_data(&MidiEvent::PitchBend { channel: 0, value: 100 });
        assert_eq!(data, vec![0x64, 0x40]);

        // -50
        // 0 - 50 + 8192 = 8142 = 0x1FCE
        // LSB = 0x1FCE & 0x7F = 0x4E (78)
        // MSB = (0x1FCE >> 7) & 0x7F = 0x3F (63)
        let data = generator.get_event_data(&MidiEvent::PitchBend { channel: 0, value: -50 });
        assert_eq!(data, vec![0x4E, 0x3F]);
    }

    #[test]
    fn test_midi_format_parsing() {
        use crate::OutputFormat;
        
        assert_eq!("mid".parse::<OutputFormat>().unwrap(), OutputFormat::MID);
        assert_eq!("MID".parse::<OutputFormat>().unwrap(), OutputFormat::MID);
        assert_eq!("Mid".parse::<OutputFormat>().unwrap(), OutputFormat::MID);
    }

    #[test]
    fn test_midi_codegenerator_format() {
        use super::super::OutputFormat;
        
        let generator = MidiGenerator::from_ast(&MmlAst::new(), &CompileOptions::default()).unwrap();
        assert_eq!(generator.format(), super::OutputFormat::Midi);
    }

    #[test]
    fn test_tempo_calculation() {
        let generator = MidiGenerator::from_ast(&MmlAst::new(), &CompileOptions::default()).unwrap();
        
        // 120 BPM = 500000 microseconds per quarter
        assert_eq!(generator.calc_microseconds_per_quarter(120), 500000);
        
        // 60 BPM = 1000000 microseconds per quarter
        assert_eq!(generator.calc_microseconds_per_quarter(60), 1000000);
        
        // 240 BPM = 250000 microseconds per quarter
        assert_eq!(generator.calc_microseconds_per_quarter(240), 250000);
    }
}
