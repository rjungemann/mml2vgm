/// Instrument serialization utilities for parsing and generating MML instrument definitions.
///
/// Provides round-trip serialization (parse MML → data structure → regenerate MML) for:
/// - FM Tone: `'@ M/F NNN` (4-operator instruments)
/// - PCM Sample: `'@ P NNN` (WAV-based samples)
/// - Envelope: `'@ E NNN` (volume envelope sequences)
/// - Arpeggio: `'@ A NNN` (note sequences)

use crate::MmlResult;
use std::collections::HashMap;

// ============================================================================
// FM Instrument Types
// ============================================================================

/// One FM operator row: 11 parameters in order [AR, DR, SR, RR, SL, TL, KS, ML, DT, AM, SSG].
pub type FmOperator = [u32; 11];

/// FM instrument definition with 4 operators.
#[derive(Debug, Clone)]
pub struct FmInstrumentDef {
    /// 0-based instrument number
    pub number: u32,
    /// M-type (auto-TL) or F-type (explicit carrier TL)
    pub ty: char,
    /// Optional patch name from header line
    pub name: String,
    /// 4 operator rows in MML order (OP1..OP4)
    pub ops: [FmOperator; 4],
    /// Algorithm 0-7
    pub alg: u32,
    /// Feedback 0-7
    pub fb: u32,
    /// Line index of the opening "'@ M/F NNN" line
    pub start_line: i32,
    /// Line index of the final ALG/FB row
    pub end_line: i32,
}

/// Parameter names in order for FM operators
pub const FM_PARAM_NAMES: &[&str] = &["AR", "DR", "SR", "RR", "SL", "TL", "KS", "ML", "DT", "AM", "SSG"];

/// Max values for each FM parameter
pub const FM_PARAM_MAX: &[u32] = &[31, 31, 31, 15, 15, 127, 3, 15, 7, 1, 15];

/// Returns which OP indices (0-based) are carriers for a given algorithm.
pub fn get_carrier_ops(alg: u32) -> Vec<usize> {
    match alg {
        0 => vec![3],
        1 => vec![3],
        2 => vec![3],
        3 => vec![3],
        4 => vec![2, 3],
        5 => vec![1, 2, 3],
        6 => vec![1, 2, 3],
        7 => vec![0, 1, 2, 3],
        _ => vec![3],
    }
}

/// Default FM operator (all zeros except ML=1).
pub const DEFAULT_OP: FmOperator = [31, 0, 0, 7, 0, 0, 0, 1, 0, 0, 0];

/// Parse all FM instrument definitions from a source string.
pub fn parse_fm_instruments(source: &str) -> Vec<FmInstrumentDef> {
    let lines: Vec<&str> = source.lines().collect();
    let mut results = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Match: '@ M 000 or '@ F 001 "optional name"
        if line.starts_with("'@") {
            let rest = &line[2..].trim();
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 && (parts[0] == "M" || parts[0] == "F") {
                if let Ok(number) = parts[1].parse::<u32>() {
                    let ty = parts[0].chars().next().unwrap_or('M');
                    let name = if rest.contains('"') {
                        let start = rest.find('"').unwrap_or(0) + 1;
                        let end = rest[start..].find('"').map(|p| start + p).unwrap_or(start);
                        rest[start..end].to_string()
                    } else {
                        String::new()
                    };
                    let start_line = i as i32;

                    // Skip comment/header line if present
                    let mut j = i + 1;
                    if j < lines.len() {
                        let next = lines[j].trim();
                        if next.starts_with("AR") || next.starts_with(';') {
                            j += 1;
                        }
                    }

                    // Read 4 operator rows
                    let mut ops: Vec<FmOperator> = Vec::new();
                    while j < lines.len() && ops.len() < 4 {
                        let op_line = lines[j].trim();
                        if op_line.starts_with("'@") && !op_line[2..].trim().is_empty() {
                            let vals_str = &op_line[2..].trim();
                            let mut vals: Vec<u32> = vals_str
                                .split(',')
                                .filter_map(|s| s.trim().parse().ok())
                                .collect();
                            if vals.len() >= 4 {
                                while vals.len() < 11 {
                                    vals.push(0);
                                }
                                ops.push([vals[0], vals[1], vals[2], vals[3], vals[4], vals[5], vals[6], vals[7], vals[8], vals[9], vals[10]]);
                                j += 1;
                                continue;
                            }
                        }
                        let empty_or_comment = op_line.is_empty() || op_line.starts_with(';');
                        if empty_or_comment {
                            j += 1;
                        } else {
                            break;
                        }
                    }

                    if ops.len() != 4 {
                        i += 1;
                        continue;
                    }

                    // Read ALG/FB row
                    let mut alg = 7u32;
                    let mut fb = 0u32;
                    let mut end_line = j as i32;
                    while j < lines.len() {
                        let alg_line = lines[j].trim();
                        if alg_line.starts_with("'@") {
                            let vals_str = &alg_line[2..].trim();
                            let parts: Vec<&str> = vals_str.split(',').collect();
                            if parts.len() >= 2 {
                                if let (Ok(a), Ok(f)) = (parts[0].trim().parse::<u32>(), parts[1].trim().parse::<u32>()) {
                                    alg = a;
                                    fb = f;
                                    end_line = j as i32;
                                    j += 1;
                                    break;
                                }
                            }
                        }
                        let empty_or_comment = alg_line.is_empty() || alg_line.starts_with(';');
                        if empty_or_comment {
                            j += 1;
                        } else {
                            break;
                        }
                    }

                    results.push(FmInstrumentDef {
                        number,
                        ty,
                        name,
                        ops: [ops[0], ops[1], ops[2], ops[3]],
                        alg,
                        fb,
                        start_line,
                        end_line,
                    });

                    i = j;
                    continue;
                }
            }
        }

        i += 1;
    }

    results
}

/// Serialize a single FM instrument definition back to MML text.
pub fn serialize_fm_instrument(inst: &FmInstrumentDef) -> String {
    let num = format!("{:03}", inst.number);
    let name_part = if inst.name.is_empty() {
        String::new()
    } else {
        format!(" \"{}\"", inst.name)
    };
    let header = format!("'@ {} {}{}", inst.ty, num, name_part);
    let col_header = "   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG";
    let mut op_lines = Vec::new();
    for op in &inst.ops {
        let vals = op.iter().map(|v| format!("{:03}", v)).collect::<Vec<_>>().join(",");
        op_lines.push(format!("'@ {}", vals));
    }
    let alg_line = "   ALG FB";
    let alg_fb_line = format!("'@ {:03},{:03}", inst.alg, inst.fb);

    let mut result = vec![header, col_header.to_string()];
    result.extend(op_lines);
    result.push(alg_line.to_string());
    result.push(alg_fb_line);
    result.join("\n")
}

// ============================================================================
// PCM Instrument Types
// ============================================================================

/// PCM/Sample instrument definition.
#[derive(Debug, Clone)]
pub struct PcmInstrumentDef {
    pub number: u32,
    pub filename: String,
    pub frequency: u32,
    pub volume: u32,
    pub chip: String,
    pub option: String,
    pub start_line: i32,
    pub end_line: i32,
}

/// Parse all PCM instrument definitions from a source string.
pub fn parse_pcm_instruments(source: &str) -> Vec<PcmInstrumentDef> {
    let mut results = Vec::new();

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // '@ P 001,"kick.wav",8000,100,YM2612[,option]
        if trimmed.starts_with("'@ P") {
            if let Some(csv_part) = trimmed.strip_prefix("'@ P").map(|s| s.trim()) {
                let mut parts = Vec::new();
                let mut current = String::new();
                let mut in_quotes = false;

                for ch in csv_part.chars() {
                    match ch {
                        '"' => in_quotes = !in_quotes,
                        ',' if !in_quotes => {
                            parts.push(current.trim().to_string());
                            current.clear();
                        }
                        _ => current.push(ch),
                    }
                }
                if !current.is_empty() {
                    parts.push(current.trim().to_string());
                }

                if parts.len() >= 5 {
                    if let Ok(number) = parts[0].parse::<u32>() {
                        let filename = parts[1].trim_matches('"').to_string();
                        let frequency = parts[2].parse::<u32>().unwrap_or(8000);
                        let volume = parts[3].parse::<u32>().unwrap_or(100);
                        let chip = parts[4].clone();
                        let option = parts.get(5).cloned().unwrap_or_default();

                        results.push(PcmInstrumentDef {
                            number,
                            filename,
                            frequency,
                            volume,
                            chip,
                            option,
                            start_line: i as i32,
                            end_line: i as i32,
                        });
                    }
                }
            }
        }
    }

    results
}

/// Serialize a single PCM instrument definition back to MML text.
pub fn serialize_pcm_instrument(inst: &PcmInstrumentDef) -> String {
    let num = format!("{:03}", inst.number);
    let opt = if inst.option.is_empty() {
        String::new()
    } else {
        format!(",{}", inst.option)
    };
    format!("'@ P {},{}\"{}\",{},{},{}{}",
        num, "\"", inst.filename, inst.frequency, inst.volume, inst.chip, opt)
}

// ============================================================================
// Envelope Types
// ============================================================================

/// Envelope definition (volume sequence).
#[derive(Debug, Clone)]
pub struct EnvelopeDef {
    pub number: u32,
    pub steps: Vec<u32>,
    pub start_line: i32,
    pub end_line: i32,
}

/// Parse all envelope definitions from a source string.
pub fn parse_envelopes(source: &str) -> Vec<EnvelopeDef> {
    let mut results = Vec::new();

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // '@ E 001, 0,1,2,3,4,5,6,7
        if trimmed.starts_with("'@ E") {
            if let Some(rest) = trimmed.strip_prefix("'@ E").map(|s| s.trim()) {
                let parts: Vec<&str> = rest.split(',').collect();
                if let Ok(number) = parts[0].parse::<u32>() {
                    let steps: Vec<u32> = parts[1..]
                        .iter()
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();

                    results.push(EnvelopeDef {
                        number,
                        steps,
                        start_line: i as i32,
                        end_line: i as i32,
                    });
                }
            }
        }
    }

    results
}

/// Serialize a single envelope definition back to MML text.
pub fn serialize_envelope(env: &EnvelopeDef) -> String {
    let num = format!("{:03}", env.number);
    let steps = env.steps.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
    format!("'@ E {}, {}", num, steps)
}

// ============================================================================
// Arpeggio Types
// ============================================================================

/// Arpeggio definition (note sequence).
#[derive(Debug, Clone)]
pub struct ArpeggioDef {
    pub number: u32,
    pub notes: Vec<String>,
    pub start_line: i32,
    pub end_line: i32,
}

/// Parse all arpeggio definitions from a source string.
pub fn parse_arpeggios(source: &str) -> Vec<ArpeggioDef> {
    let mut results = Vec::new();

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // '@ A 001, c4,e4,g4,c5
        if trimmed.starts_with("'@ A") {
            if let Some(rest) = trimmed.strip_prefix("'@ A").map(|s| s.trim()) {
                let parts: Vec<&str> = rest.split(',').collect();
                if let Ok(number) = parts[0].parse::<u32>() {
                    let notes: Vec<String> = parts[1..]
                        .iter()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    results.push(ArpeggioDef {
                        number,
                        notes,
                        start_line: i as i32,
                        end_line: i as i32,
                    });
                }
            }
        }
    }

    results
}

/// Serialize a single arpeggio definition back to MML text.
pub fn serialize_arpeggio(arp: &ArpeggioDef) -> String {
    let num = format!("{:03}", arp.number);
    let notes = arp.notes.join(",");
    format!("'@ A {}, {}", num, notes)
}

// ============================================================================
// Block Replacement
// ============================================================================

/// Replace an existing instrument definition block in source text with a new one.
/// If start_line == -1 (new instrument), append to end of source.
pub fn replace_definition_block(source: &str, start_line: i32, end_line: i32, new_block: &str) -> String {
    if start_line == -1 {
        let sep = if source.ends_with('\n') { "\n" } else { "\n\n" };
        return format!("{}{}{}\n", source, sep, new_block);
    }

    let lines: Vec<&str> = source.lines().collect();
    let start_idx = start_line as usize;
    let end_idx = end_line as usize;

    if start_idx >= lines.len() {
        return source.to_string();
    }

    let before = &lines[..start_idx];
    let after = if end_idx + 1 < lines.len() {
        &lines[end_idx + 1..]
    } else {
        &[]
    };

    let mut result = Vec::new();
    result.extend_from_slice(before);
    result.push(new_block);
    result.extend_from_slice(after);
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carrier_ops() {
        assert_eq!(get_carrier_ops(0), vec![3]);
        assert_eq!(get_carrier_ops(4), vec![2, 3]);
        assert_eq!(get_carrier_ops(5), vec![1, 2, 3]);
        assert_eq!(get_carrier_ops(7), vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_fm_roundtrip() {
        let source = r#"'@ M 000
   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
'@ 031,012,000,007,002,000,000,001,000,000,000
'@ 031,012,000,007,002,000,000,001,000,000,000
'@ 031,012,000,007,002,000,000,001,000,000,000
'@ 031,012,000,007,002,000,000,001,000,000,000
   ALG FB
'@ 007,000"#;

        let instruments = parse_fm_instruments(source);
        assert_eq!(instruments.len(), 1);
        assert_eq!(instruments[0].number, 0);
        assert_eq!(instruments[0].ty, 'M');
        assert_eq!(instruments[0].alg, 7);
        assert_eq!(instruments[0].fb, 0);

        let serialized = serialize_fm_instrument(&instruments[0]);
        let re_parsed = parse_fm_instruments(&serialized);
        assert_eq!(re_parsed.len(), 1);
        assert_eq!(re_parsed[0].number, instruments[0].number);
        assert_eq!(re_parsed[0].alg, instruments[0].alg);
    }

    #[test]
    fn test_envelope_roundtrip() {
        let source = "'@ E 001, 0,16,48,96,127,96,64,32,16,8,4,0";
        let envelopes = parse_envelopes(source);
        assert_eq!(envelopes.len(), 1);
        assert_eq!(envelopes[0].number, 1);
        assert_eq!(envelopes[0].steps.len(), 12);

        let serialized = serialize_envelope(&envelopes[0]);
        let re_parsed = parse_envelopes(&serialized);
        assert_eq!(re_parsed.len(), 1);
        assert_eq!(re_parsed[0].steps, envelopes[0].steps);
    }

    #[test]
    fn test_arpeggio_roundtrip() {
        let source = "'@ A 001, c4,e4,g4,c5";
        let arpeggios = parse_arpeggios(source);
        assert_eq!(arpeggios.len(), 1);
        assert_eq!(arpeggios[0].number, 1);
        assert_eq!(arpeggios[0].notes.len(), 4);

        let serialized = serialize_arpeggio(&arpeggios[0]);
        let re_parsed = parse_arpeggios(&serialized);
        assert_eq!(re_parsed.len(), 1);
        assert_eq!(re_parsed[0].notes, arpeggios[0].notes);
    }
}
