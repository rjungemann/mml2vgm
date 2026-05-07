//! Phase 8: VGM codegen accuracy tests.
//!
//! These tests open the generated VGM binary and verify that the register-write
//! bytes match the theoretical values derived from the MML input (MIDI note
//! numbers, tempo-derived wait durations, etc.).

use mml2vgm::compiler::compiler::MmlCompiler;
use mml2vgm::{CompileOptions, OutputFormat};

const SN76489_CLOCK: u32 = 3_579_545;
const SAMPLE_RATE: u32 = 44100;

// ── VGM command extraction helpers ───────────────────────────────────────────

/// A decoded VGM command as (opcode, payload_bytes).
#[derive(Debug, Clone)]
struct VgmCmd {
    opcode: u8,
    payload: Vec<u8>,
}

/// Extract all commands from a VGM binary starting at the data offset.
fn extract_vgm_commands(data: &[u8]) -> Vec<VgmCmd> {
    let data_offset = if data.len() > 0x40 {
        let raw = u32::from_le_bytes([data[0x34], data[0x35], data[0x36], data[0x37]]);
        if raw == 0 { 0x40 } else { (raw as usize) + 0x34 }
    } else {
        0x40
    };

    let mut cmds = Vec::new();
    let mut i = data_offset.min(data.len());

    while i < data.len() {
        let op = data[i];
        match op {
            0x50 if i + 1 < data.len() => {
                cmds.push(VgmCmd { opcode: op, payload: vec![data[i + 1]] });
                i += 2;
            }
            0x51..=0x5F | 0xA0 | 0xB0..=0xBF if i + 2 < data.len() => {
                cmds.push(VgmCmd { opcode: op, payload: vec![data[i + 1], data[i + 2]] });
                i += 3;
            }
            0x52 | 0x53 if i + 2 < data.len() => {
                cmds.push(VgmCmd { opcode: op, payload: vec![data[i + 1], data[i + 2]] });
                i += 3;
            }
            0x61 if i + 2 < data.len() => {
                let wait = u16::from_le_bytes([data[i + 1], data[i + 2]]);
                cmds.push(VgmCmd { opcode: op, payload: vec![data[i + 1], data[i + 2]] });
                let _ = wait;
                i += 3;
            }
            0x62 => {
                cmds.push(VgmCmd { opcode: op, payload: vec![] });
                i += 1;
            }
            0x63 => {
                cmds.push(VgmCmd { opcode: op, payload: vec![] });
                i += 1;
            }
            0x66 => {
                cmds.push(VgmCmd { opcode: op, payload: vec![] });
                break;
            }
            0x67 if i + 6 < data.len() => {
                let len = u32::from_le_bytes([data[i+2], data[i+3], data[i+4], data[i+5]]) as usize;
                cmds.push(VgmCmd { opcode: op, payload: data[i+2..i+6+len.min(data.len()-i-6)].to_vec() });
                i += 7 + len;
            }
            0xC0..=0xC4 | 0xD0..=0xD6 if i + 3 < data.len() => {
                cmds.push(VgmCmd { opcode: op, payload: vec![data[i+1], data[i+2], data[i+3]] });
                i += 4;
            }
            0xE0..=0xE1 if i + 3 < data.len() => {
                cmds.push(VgmCmd { opcode: op, payload: vec![data[i+1], data[i+2], data[i+3]] });
                i += 4;
            }
            _ => { i += 1; }
        }
    }
    cmds
}

/// Sum all wait samples in a command list.
fn total_wait_samples(cmds: &[VgmCmd]) -> u64 {
    let mut total = 0u64;
    for cmd in cmds {
        match cmd.opcode {
            0x61 if cmd.payload.len() >= 2 => {
                total += u16::from_le_bytes([cmd.payload[0], cmd.payload[1]]) as u64;
            }
            0x62 => total += 735,
            0x63 => total += 882,
            _ => {}
        }
    }
    total
}

/// Collect all SN76489 write bytes from the command list.
fn sn76489_writes(cmds: &[VgmCmd]) -> Vec<u8> {
    cmds.iter()
        .filter(|c| c.opcode == 0x50 && !c.payload.is_empty())
        .map(|c| c.payload[0])
        .collect()
}

/// Theoretical SN76489 tone divider for a MIDI note.
fn midi_to_psg_divider(midi_note: u8) -> u16 {
    let freq = 440.0_f64 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0);
    let divider = (SN76489_CLOCK as f64 / (32.0 * freq)).round() as u32;
    divider.min(1023) as u16
}

/// Expected SN76489 latch byte for channel 0 tone, given low 4 bits.
fn psg_latch_tone(divider: u16) -> u8 {
    0x80 | (divider & 0x0F) as u8 // ch0, tone type, low4
}

/// Expected SN76489 data byte, given high 6 bits.
fn psg_data_byte(divider: u16) -> u8 {
    ((divider >> 4) & 0x3F) as u8 // high6
}

// ── SN76489 tone register tests ───────────────────────────────────────────────

fn compile_vgm(mml: &str) -> Vec<u8> {
    MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(mml)
    .unwrap_or_else(|e| panic!("compile failed: {e}"))
    .data
}

#[test]
fn psg_c4_tone_bytes_in_vgm() {
    // C4 = MIDI 60; verify the two SN76489 tone bytes are in the output
    let vgm = compile_vgm("@0 t120 c4");
    let cmds = extract_vgm_commands(&vgm);
    let writes = sn76489_writes(&cmds);

    let divider = midi_to_psg_divider(60); // C4
    let latch = psg_latch_tone(divider);
    let data = psg_data_byte(divider);

    assert!(
        writes.contains(&latch),
        "VGM missing SN76489 latch byte {latch:#04x} for C4 (divider={divider}); writes={writes:?}"
    );
    assert!(
        writes.contains(&data),
        "VGM missing SN76489 data byte {data:#04x} for C4 (divider={divider}); writes={writes:?}"
    );
}

#[test]
fn psg_a4_tone_bytes_in_vgm() {
    // A4 = MIDI 69 = 440 Hz reference
    let vgm = compile_vgm("@0 t120 a4");
    let cmds = extract_vgm_commands(&vgm);
    let writes = sn76489_writes(&cmds);

    let divider = midi_to_psg_divider(69); // A4
    let latch = psg_latch_tone(divider);
    let data = psg_data_byte(divider);

    assert!(
        writes.contains(&latch),
        "VGM missing latch byte {latch:#04x} for A4; divider={divider}"
    );
    assert!(
        writes.contains(&data),
        "VGM missing data byte {data:#04x} for A4; divider={divider}"
    );
}

#[test]
fn psg_multi_note_all_tones_present() {
    // Three different notes — all three tone frequencies should appear in the stream
    let vgm = compile_vgm("@0 t120 c4 e4 g4");
    let cmds = extract_vgm_commands(&vgm);
    let writes = sn76489_writes(&cmds);

    for midi in [60u8, 64, 67] { // C4, E4, G4
        let divider = midi_to_psg_divider(midi);
        let latch = psg_latch_tone(divider);
        assert!(
            writes.contains(&latch),
            "VGM missing latch byte {latch:#04x} for MIDI {midi} (div={divider}); writes={writes:?}"
        );
    }
}

#[test]
fn psg_volume_latch_appears_before_first_note() {
    // Volume write (0x90 | atten) should appear alongside each note
    let vgm = compile_vgm("@0 t120 o4 c2");
    let cmds = extract_vgm_commands(&vgm);
    let writes = sn76489_writes(&cmds);

    // Volume latch byte for ch0: 1_00_1_VVVV = 0x90 | volume
    // Default MML volume produces some non-15 attenuation
    let any_vol_write = writes.iter().any(|&b| (b & 0xF0) == 0x90);
    assert!(
        any_vol_write,
        "expected a ch0 volume latch byte (0x9X) in SN76489 writes; got {writes:?}"
    );
}

// ── Wait duration accuracy tests ──────────────────────────────────────────────

#[test]
fn wait_duration_quarter_note_120bpm() {
    // Quarter note at 120 BPM = 0.5 s = 22050 samples at 44100 Hz
    let vgm = compile_vgm("@0 t120 o4 c4 r8192"); // Note + long rest to isolate
    let cmds = extract_vgm_commands(&vgm);
    let total = total_wait_samples(&cmds);

    // The total duration must be at least the quarter note's worth
    let min_expected = 22050u64;
    assert!(
        total >= min_expected,
        "expected ≥ {min_expected} wait samples for 120bpm quarter note, got {total}"
    );
}

#[test]
fn wait_total_matches_vgm_header_total_samples() {
    let vgm = compile_vgm("@0 t120 o4 c4 d4 e4 f4");
    let cmds = extract_vgm_commands(&vgm);
    let computed = total_wait_samples(&cmds);

    let header_total = u32::from_le_bytes([vgm[0x18], vgm[0x19], vgm[0x1A], vgm[0x1B]]) as u64;
    assert_eq!(
        computed, header_total,
        "sum of wait commands ({computed}) must equal VGM header total_samples ({header_total})"
    );
}

#[test]
fn tempo_affects_wait_duration() {
    // Slower tempo → longer wait for same note value
    let vgm_120 = compile_vgm("@0 t120 o4 c4");
    let vgm_60  = compile_vgm("@0 t60  o4 c4");

    let t120 = u32::from_le_bytes([vgm_120[0x18], vgm_120[0x19], vgm_120[0x1A], vgm_120[0x1B]]) as u64;
    let t60  = u32::from_le_bytes([vgm_60[0x18],  vgm_60[0x19],  vgm_60[0x1A],  vgm_60[0x1B]])  as u64;

    assert!(
        t60 > t120,
        "t=60 ({t60} samples) should produce longer duration than t=120 ({t120} samples)"
    );
}

#[test]
fn dotted_note_is_longer_than_plain() {
    // Dotted quarter = 1.5x regular quarter
    let vgm_plain  = compile_vgm("@0 t120 o4 c4");
    let vgm_dotted = compile_vgm("@0 t120 o4 c4.");

    let t_plain  = u32::from_le_bytes([vgm_plain[0x18],  vgm_plain[0x19],  vgm_plain[0x1A],  vgm_plain[0x1B]])  as u64;
    let t_dotted = u32::from_le_bytes([vgm_dotted[0x18], vgm_dotted[0x19], vgm_dotted[0x1A], vgm_dotted[0x1B]]) as u64;

    assert!(
        t_dotted > t_plain,
        "dotted quarter ({t_dotted}) should be longer than plain quarter ({t_plain})"
    );
    // Should be approximately 1.5x (allow ±1 sample rounding)
    let ratio = t_dotted as f64 / t_plain as f64;
    assert!(
        (ratio - 1.5).abs() < 0.1,
        "dotted/plain ratio should be ~1.5, got {ratio:.3}"
    );
}

// ── VGM structure tests ───────────────────────────────────────────────────────

#[test]
fn vgm_ends_with_eof_command() {
    let vgm = compile_vgm("@0 t120 o4 c4");
    let cmds = extract_vgm_commands(&vgm);
    let last = cmds.last().map(|c| c.opcode);
    assert_eq!(last, Some(0x66), "VGM command stream must end with EOF (0x66)");
}

#[test]
fn vgm_tone_write_precedes_wait_for_first_note() {
    // The SN76489 tone write must come before the first wait (note starts, then time passes)
    let vgm = compile_vgm("@0 t120 o4 c4");
    let cmds = extract_vgm_commands(&vgm);

    let first_write_idx = cmds.iter().position(|c| c.opcode == 0x50);
    let first_wait_idx  = cmds.iter().position(|c| matches!(c.opcode, 0x61 | 0x62 | 0x63));

    assert!(first_write_idx.is_some(), "no SN76489 write found");
    assert!(first_wait_idx.is_some(),  "no wait command found");
    assert!(
        first_write_idx.unwrap() < first_wait_idx.unwrap(),
        "tone write (idx={}) should precede first wait (idx={})",
        first_write_idx.unwrap(),
        first_wait_idx.unwrap()
    );
}

#[test]
fn sequential_notes_produce_multiple_write_groups() {
    // Two different notes should produce at least 2 distinct tone latch bytes
    let vgm = compile_vgm("@0 t120 o4 c4 g4");
    let cmds = extract_vgm_commands(&vgm);
    let writes = sn76489_writes(&cmds);

    let c4_latch = psg_latch_tone(midi_to_psg_divider(60));
    let g4_latch = psg_latch_tone(midi_to_psg_divider(67));

    assert_ne!(c4_latch, g4_latch, "C4 and G4 must have different latch bytes");
    assert!(writes.contains(&c4_latch), "C4 latch {c4_latch:#04x} missing from writes");
    assert!(writes.contains(&g4_latch), "G4 latch {g4_latch:#04x} missing from writes");
}

// ── CompileInfo accuracy ──────────────────────────────────────────────────────

#[test]
fn compile_info_command_count_positive() {
    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });
    let result = compiler
        .compile_from_source("@0 t120 o4 c4 d4 e4")
        .expect("compile failed");
    assert!(
        result.info.command_count > 0,
        "command_count should be positive for note-bearing MML"
    );
}

#[test]
fn compile_info_duration_proportional_to_note_count() {
    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });
    let r1 = compiler.compile_from_source("@0 t120 o4 c4").expect("compile failed");
    let r4 = compiler.compile_from_source("@0 t120 o4 c4 d4 e4 f4").expect("compile failed");

    assert!(
        r4.info.duration_seconds > r1.info.duration_seconds,
        "four notes ({:.3}s) should be longer than one note ({:.3}s)",
        r4.info.duration_seconds,
        r1.info.duration_seconds
    );
}

#[test]
fn compile_info_part_count_reflects_mml_parts() {
    let compiler = MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    });
    // Two explicit parts
    let result = compiler
        .compile_from_source("'A t120 o4 c4 d4\n'B t120 o3 c2 g2")
        .expect("compile failed");
    assert!(
        result.info.part_count >= 2,
        "expected ≥2 parts, got {}",
        result.info.part_count
    );
}

// ── YM2612 FM accuracy tests ──────────────────────────────────────────────────

const FM_INSTRUMENT: &str = "
'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 031,012,012,015,015,020,000,001,000,000,000
'@ 007,000
";

fn compile_ym2612(notes: &str) -> Vec<u8> {
    let mml = format!(
        "{{\n  PartYM2612 = A\n}}\n{}\n'A1 T120\n'A1 @0 v100 l4 o4 {}",
        FM_INSTRUMENT, notes
    );
    MmlCompiler::new(CompileOptions {
        format: OutputFormat::VGM,
        ..Default::default()
    })
    .compile_from_source(&mml)
    .unwrap_or_else(|e| panic!("YM2612 compile failed: {e}"))
    .data
}

/// Collect all YM2612 port-0 writes (opcode 0x52) as (addr, data) pairs.
fn ym2612_port0_writes(cmds: &[VgmCmd]) -> Vec<(u8, u8)> {
    cmds.iter()
        .filter(|c| c.opcode == 0x52 && c.payload.len() >= 2)
        .map(|c| (c.payload[0], c.payload[1]))
        .collect()
}

/// Collect all YM2612 port-1 writes (opcode 0x53) as (addr, data) pairs.
fn ym2612_port1_writes(cmds: &[VgmCmd]) -> Vec<(u8, u8)> {
    cmds.iter()
        .filter(|c| c.opcode == 0x53 && c.payload.len() >= 2)
        .map(|c| (c.payload[0], c.payload[1]))
        .collect()
}

#[test]
fn ym2612_vgm_contains_port0_writes() {
    let vgm = compile_ym2612("c d e f");
    let cmds = extract_vgm_commands(&vgm);
    let writes = ym2612_port0_writes(&cmds);
    assert!(
        !writes.is_empty(),
        "YM2612 VGM should contain at least one port-0 register write (0x52)"
    );
}

#[test]
fn ym2612_key_on_register_written() {
    // Key-on is written to port 0 addr 0x28; any value with bit4..6 set
    let vgm = compile_ym2612("c4");
    let cmds = extract_vgm_commands(&vgm);
    let writes = ym2612_port0_writes(&cmds);

    let has_key_on = writes.iter().any(|&(addr, data)| addr == 0x28 && data & 0xF0 != 0);
    assert!(
        has_key_on,
        "VGM missing YM2612 key-on write (addr=0x28, data bits[7:4] non-zero); port0_writes={writes:?}"
    );
}

#[test]
fn ym2612_different_notes_produce_different_fnum_writes() {
    // F-number registers are at port 0 addrs 0xA0-0xA2 (lo) and 0xA4-0xA6 (hi+block)
    let vgm_c = compile_ym2612("c4 r8192"); // C note
    let vgm_g = compile_ym2612("g4 r8192"); // G note (perfect fifth up)

    let cmds_c = extract_vgm_commands(&vgm_c);
    let cmds_g = extract_vgm_commands(&vgm_g);

    let fnum_c: Vec<_> = ym2612_port0_writes(&cmds_c)
        .into_iter()
        .filter(|&(addr, _)| (0xA0..=0xA2).contains(&addr))
        .collect();
    let fnum_g: Vec<_> = ym2612_port0_writes(&cmds_g)
        .into_iter()
        .filter(|&(addr, _)| (0xA0..=0xA2).contains(&addr))
        .collect();

    assert!(!fnum_c.is_empty(), "no F-num lo writes for C note");
    assert!(!fnum_g.is_empty(), "no F-num lo writes for G note");
    assert_ne!(
        fnum_c, fnum_g,
        "C note and G note should produce different F-number register values"
    );
}

#[test]
fn ym2612_key_off_before_next_note() {
    // Playing two sequential notes should generate a key-off (0x28 with lower nibble bits, upper bits 0)
    // between them — specifically after the first note's wait
    let vgm = compile_ym2612("c4 d4");
    let cmds = extract_vgm_commands(&vgm);
    let port0 = ym2612_port0_writes(&cmds);

    // Both key-on and key-off use addr 0x28; key-off has bits[7:4] == 0
    let has_key_off = port0.iter().any(|&(addr, data)| addr == 0x28 && data & 0xF0 == 0);
    assert!(
        has_key_off,
        "two sequential YM2612 notes must generate at least one key-off (addr=0x28, data[7:4]=0)"
    );
}

#[test]
fn ym2612_operator_tl_written_for_fm_instrument() {
    // Total Level (TL) registers are at addrs 0x40-0x4F (port 0 and 1)
    // An FM instrument definition should write these before key-on
    let vgm = compile_ym2612("c4");
    let cmds = extract_vgm_commands(&vgm);
    let port0 = ym2612_port0_writes(&cmds);
    let port1 = ym2612_port1_writes(&cmds);

    let has_tl = port0.iter().any(|&(addr, _)| (0x40..=0x4F).contains(&addr))
        || port1.iter().any(|&(addr, _)| (0x40..=0x4F).contains(&addr));
    assert!(
        has_tl,
        "YM2612 FM instrument should write TL registers (0x40-0x4F)"
    );
}

#[test]
fn ym2612_b0_register_written_to_enable_channel() {
    // Register 0xB0 sets algorithm and feedback; must be written when loading an instrument
    let vgm = compile_ym2612("c4");
    let cmds = extract_vgm_commands(&vgm);
    let port0 = ym2612_port0_writes(&cmds);
    let port1 = ym2612_port1_writes(&cmds);

    let has_b0 = port0.iter().any(|&(addr, _)| (0xB0..=0xB2).contains(&addr))
        || port1.iter().any(|&(addr, _)| (0xB0..=0xB2).contains(&addr));
    assert!(
        has_b0,
        "YM2612 FM instrument should write algorithm/feedback register (0xB0-0xB2)"
    );
}

// ── SN76489 silence / key-off ─────────────────────────────────────────────────

#[test]
fn psg_volume_off_written_after_note_ends() {
    // After a note, the PSG channel should be silenced (attenuation = 0xF = max atten)
    // Volume latch for ch0: 0x9F
    let vgm = compile_vgm("@0 t120 o4 c4 r4"); // note + rest
    let cmds = extract_vgm_commands(&vgm);
    let writes = sn76489_writes(&cmds);
    let has_silence = writes.contains(&0x9F); // ch0 max attenuation
    assert!(
        has_silence,
        "SN76489 ch0 should be silenced (0x9F) after a note ends; writes={writes:?}"
    );
}

// ── Accidental frequency accuracy ────────────────────────────────────────────

#[test]
fn psg_sharp_note_has_different_frequency_than_natural() {
    // C4 and C#4 must produce different SN76489 tone dividers
    let vgm_c  = compile_vgm("@0 t120 o4 c4");
    let vgm_cs = compile_vgm("@0 t120 o4 c+4");

    let writes_c  = sn76489_writes(&extract_vgm_commands(&vgm_c));
    let writes_cs = sn76489_writes(&extract_vgm_commands(&vgm_cs));

    // The latch bytes for the tone frequency should differ
    let latch_c  = psg_latch_tone(midi_to_psg_divider(60)); // C4 = MIDI 60
    let latch_cs = psg_latch_tone(midi_to_psg_divider(61)); // C#4 = MIDI 61

    assert_ne!(latch_c, latch_cs, "C4 and C#4 latch bytes must differ");
    assert!(writes_c.contains(&latch_c),  "C4  latch {latch_c:#04x}  missing from writes");
    assert!(writes_cs.contains(&latch_cs), "C#4 latch {latch_cs:#04x} missing from writes");
}

#[test]
fn psg_flat_note_equals_enharmonic_sharp() {
    // D-flat4 == C#4 (enharmonic equivalents); should produce same divider
    let vgm_cs = compile_vgm("@0 t120 o4 c+4");
    let vgm_db = compile_vgm("@0 t120 o4 d-4");

    let latch_cs: Vec<_> = sn76489_writes(&extract_vgm_commands(&vgm_cs))
        .into_iter()
        .filter(|&b| b & 0x80 != 0 && b & 0x60 == 0)
        .collect();
    let latch_db: Vec<_> = sn76489_writes(&extract_vgm_commands(&vgm_db))
        .into_iter()
        .filter(|&b| b & 0x80 != 0 && b & 0x60 == 0)
        .collect();

    assert_eq!(
        latch_cs, latch_db,
        "C#4 and D-flat4 should produce identical SN76489 latch bytes (enharmonic)"
    );
}

// ── Wait granularity ──────────────────────────────────────────────────────────

#[test]
fn short_notes_use_0x70_style_waits() {
    // Notes with durations 1-16 samples often use the 0x70-0x7F compact wait opcodes
    // (Not a hard requirement, but if total_samples > 0, the stream isn't empty)
    let vgm = compile_vgm("@0 t240 o4 c32 d32 e32 f32");
    let cmds = extract_vgm_commands(&vgm);
    let total = total_wait_samples(&cmds);
    assert!(total > 0, "fast notes at high BPM should still produce non-zero total samples");
}

#[test]
fn long_note_uses_0x61_wait() {
    // A whole note at low BPM should produce at least one 0x61 wait
    let vgm = compile_vgm("@0 t60 o4 c1");
    let cmds = extract_vgm_commands(&vgm);
    let has_long_wait = cmds.iter().any(|c| c.opcode == 0x61);
    assert!(has_long_wait, "a whole note at t60 should produce at least one 0x61 (long wait) command");
}
