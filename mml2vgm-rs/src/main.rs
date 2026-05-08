//! mml2vgm-rs: Command-line utility for compiling MML files to VGM/XGM/ZGM formats
//!
//! This is the main entry point for the CLI application.

use clap::{Parser, ValueEnum};
use log::{debug, error, info, warn};
use std::path::{Path, PathBuf};
use std::process;

use mml2vgm::{
    CompileInfo, CompileOptions, CompileResult, MmlResult, OutputFormat, SoundChip,
    compiler::compiler::MmlCompiler,
    player::VgmPlayer,
    utils::wav::write_wav,
};

/// mml2vgm-rs: MML to VGM/XGM/ZGM compiler and player
#[derive(Parser, Debug)]
#[command(name = "mml2vgm-rs")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Compile MML files to VGM/XGM/ZGM formats and play them")]
#[command(long_about = None)]
#[command(disable_version_flag = true)]
struct Args {
    /// Input MML file (.gwi). If not specified, reads from stdin.
    #[arg(required = false, index = 1)]
    input: Option<PathBuf>,

    /// Output file. If not specified, uses input name with .vgm/.xgm/.zgm extension.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "vgm")]
    format: FormatArg,

    /// Play the output file after compilation (or play a .vgm file directly)
    #[arg(short, long)]
    play: bool,

    /// Export rendered audio to a WAV file (can be combined with --play)
    #[arg(short = 'w', long = "export-wav")]
    export_wav: Option<PathBuf>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Show debug output (very verbose, for development)
    #[arg(long)]
    debug: bool,

    /// Output trace information file
    #[arg(long)]
    trace: bool,

    /// Validate MML only, don't compile
    #[arg(long)]
    check: bool,

    /// List supported sound chips
    #[arg(long)]
    list_chips: bool,

    /// List supported output formats
    #[arg(long)]
    list_formats: bool,

    /// Target sound chip (can be specified multiple times)
    #[arg(short, long)]
    chip: Vec<String>,

    /// Clock count override (default: 192)
    #[arg(long)]
    clock_count: Option<u32>,

    /// Add include path (can be specified multiple times)
    #[arg(short = 'I', long = "include")]
    include: Vec<PathBuf>,

    /// Show version information
    #[arg(long)]
    version: bool,
}

/// Wrapper for OutputFormat that implements ValueEnum for clap
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum FormatArg {
    /// Standard VGM format
    Vgm,
    /// XGM format (Mega Drive)
    Xgm,
    /// XGM2 format (Mega Drive with extended features)
    Xgm2,
    /// ZGM format (Extended VGM with YM2609 and MIDI support)
    Zgm,
    /// MIDI format (Standard MIDI File)
    Mid,
}

impl std::fmt::Display for FormatArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatArg::Vgm => write!(f, "vgm"),
            FormatArg::Xgm => write!(f, "xgm"),
            FormatArg::Xgm2 => write!(f, "xgm2"),
            FormatArg::Zgm => write!(f, "zgm"),
            FormatArg::Mid => write!(f, "mid"),
        }
    }
}

impl From<FormatArg> for OutputFormat {
    fn from(arg: FormatArg) -> Self {
        match arg {
            FormatArg::Vgm => OutputFormat::VGM,
            FormatArg::Xgm => OutputFormat::XGM,
            FormatArg::Xgm2 => OutputFormat::XGM2,
            FormatArg::Zgm => OutputFormat::ZGM,
            FormatArg::Mid => OutputFormat::MID,
        }
    }
}

/// Initialize logging based on verbosity settings
fn init_logging(verbose: bool, debug: bool) {
    use log::LevelFilter;

    let level = if debug {
        LevelFilter::Debug
    } else if verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Warn
    };

    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp(None)
        .init();

    debug!("Logging initialized at level: {}", level);
}

/// List all supported sound chips
fn list_supported_chips() {
    println!("Supported Sound Chips:");
    println!();

    // Group by category - using Vec to allow different sized arrays
    let categories: Vec<(&str, Vec<SoundChip>)> = vec![
        ("Mega Drive / Genesis", vec![SoundChip::YM2612, SoundChip::YM2612X, SoundChip::YM2612X2]),
        ("PSG / SSG", vec![SoundChip::SN76489, SoundChip::SN76489X2, SoundChip::AY8910]),
        ("PC Engine / TurboGrafx-16", vec![SoundChip::YM2608]),
        ("Extended OPNA", vec![SoundChip::YM2609]),
        ("OPN Series", vec![SoundChip::YM2610B, SoundChip::YM2203]),
        ("OPM Series", vec![SoundChip::YM2151]),
        ("OPL Series", vec![SoundChip::YM3526, SoundChip::Y8950, SoundChip::YM3812, SoundChip::YMF262]),
        ("OPLL", vec![SoundChip::YM2413]),
        ("PCM Chips", vec![SoundChip::RF5C164, SoundChip::SegaPCM, SoundChip::C140, SoundChip::C352, SoundChip::QSound]),
        ("Other", vec![SoundChip::HuC6280, SoundChip::K051649, SoundChip::K053260, SoundChip::K054539]),
        ("Console Chips", vec![SoundChip::NES, SoundChip::DMG, SoundChip::VRC6]),
        ("Atari", vec![SoundChip::POKEY]),
        ("MIDI", vec![SoundChip::MIDI]),
        ("Special", vec![SoundChip::CONDUCTOR]),
    ];

    for (category, chips) in &categories {
        println!("  {}:", category);
        for chip in chips {
            println!(
                "    - {} [{}] (clock: {}Hz)",
                chip.name(),
                chip.support_tier(),
                chip.clock_rate()
            );
        }
        println!();
    }
}

/// List all supported output formats
fn list_supported_formats() {
    println!("Supported Output Formats:");
    println!();
    println!("  - vgm  [{}] - Standard VGM format", OutputFormat::VGM.support_tier());
    println!("  - xgm  [{}] - XGM format (Mega Drive)", OutputFormat::XGM.support_tier());
    println!("  - xgm2 [{}] - XGM2 format (Mega Drive with extended features)", OutputFormat::XGM2.support_tier());
    println!("  - zgm  [{}] - ZGM format (Extended VGM with YM2609 and MIDI support)", OutputFormat::ZGM.support_tier());
    println!("  - mid  [{}] - Standard MIDI File format", OutputFormat::MID.support_tier());
    println!();
    println!("Default: vgm");
}

/// Determine output path based on input and format
fn determine_output_path(input: Option<&PathBuf>, output: Option<&PathBuf>, format: OutputFormat) -> MmlResult<PathBuf> {
    if let Some(out) = output {
        return Ok(out.to_path_buf());
    }

    if let Some(in_path) = input {
        let stem = in_path.file_stem().unwrap_or_else(|| in_path.as_os_str());
        let extension = format.extension();
        let mut out_path = in_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
        out_path.push(format!("{}.{}", stem.to_string_lossy(), extension));
        return Ok(out_path);
    }

    // No input, no output specified - use stdout
    Ok(PathBuf::from("-"))
}

/// Render VGM data to samples, optionally export WAV and/or play via rodio.
fn render_and_play(vgm_data: &[u8], play: bool, export_wav: Option<&PathBuf>) {
    let sample_rate = 44100u32;
    let mut player = VgmPlayer::new();
    if let Err(e) = player.load(vgm_data) {
        error!("Failed to load VGM for playback: {}", e);
        process::exit(1);
    }
    player.init_chips_from_header();
    let samples = match player.render_to_pcm(sample_rate) {
        Ok(s) => s,
        Err(e) => { error!("Failed to render audio: {}", e); process::exit(1); }
    };
    if samples.is_empty() {
        warn!("No audio samples generated — check MML content and chip driver.");
        return;
    }
    if let Some(wav_path) = export_wav {
        match write_wav(wav_path, &samples, sample_rate) {
            Ok(()) => println!("WAV exported to: {}", wav_path.display()),
            Err(e) => { error!("WAV export failed: {}", e); process::exit(1); }
        }
    }
    if play {
        match rodio::OutputStream::try_default() {
            Ok((_stream, handle)) => {
                match rodio::Sink::try_new(&handle) {
                    Ok(sink) => {
                        sink.append(rodio::buffer::SamplesBuffer::new(2, sample_rate, samples));
                        println!("Playing... (Ctrl+C to stop)");
                        sink.sleep_until_end();
                        println!("Playback complete.");
                    }
                    Err(e) => { error!("Audio sink error: {}", e); process::exit(1); }
                }
            }
            Err(e) => { error!("No audio output device: {}", e); process::exit(1); }
        }
    }
}

/// Returns true if the path looks like a pre-compiled VGM/XGM/ZGM file.
fn is_compiled_audio_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()).map(str::to_lowercase).as_deref() {
        Some("vgm" | "xgm" | "xgm2" | "zgm" | "vgz") => true,
        _ => false,
    }
}

/// Print a compiler error with source-context (line + caret) when position info is available.
fn print_diagnostic(path: Option<&Path>, err: &mml2vgm::MmlError) {
    match err {
        mml2vgm::MmlError::Parse { line, column, message } => {
            eprintln!("error[E0001]: {}", message);
            if let Some(p) = path {
                eprintln!("  --> {}:{}:{}", p.display(), line, column);
                if let Ok(source) = std::fs::read_to_string(p) {
                    let lines: Vec<&str> = source.lines().collect();
                    if let Some(src_line) = lines.get(line.saturating_sub(1)) {
                        let ln_width = format!("{}", line).len();
                        eprintln!("  {:width$} |", "", width = ln_width);
                        eprintln!(" {:>width$} | {}", line, src_line, width = ln_width);
                        let caret_col = column.saturating_sub(1);
                        eprintln!("  {:width$} | {}^", "", " ".repeat(caret_col), width = ln_width);
                    }
                }
            }
            if let Some(hint) = parse_error_hint(message) {
                eprintln!("  = help: {}", hint);
            }
        }
        mml2vgm::MmlError::UnsupportedChip(name) => {
            eprintln!("error: unknown chip '{}'", name);
            eprintln!("  = help: valid chip names include YM2612, SN76489, YM2608, YM2151, YM3812,");
            eprintln!("           AY8910, HuC6280, YM2413, K051649, NES, DMG, POKEY, VRC6, QSound");
            eprintln!("           (use --list-chips for the full list with support tiers)");
        }
        mml2vgm::MmlError::FileNotFound(p) => {
            eprintln!("error: file not found: {}", p.display());
            eprintln!("  = help: check the path and that the file has a .gwi extension");
        }
        other => {
            eprintln!("error: {}", other);
        }
    }
}

/// Return a contextual hint for a parse error message, if one is known.
fn parse_error_hint(message: &str) -> Option<&'static str> {
    let m = message.to_ascii_lowercase();
    if m.contains("unexpected") || m.contains("expected") {
        Some("notes are A-G (or a-g), rests are 'r'; commands: t (tempo), v (volume), o (octave), l (default length), @ (instrument)")
    } else if m.contains("duration") || m.contains("invalid number") {
        Some("note durations must be 1, 2, 4, 8, 16, 32, or 64 — optionally followed by '.' for dotted")
    } else if m.contains("octave") {
        Some("octave range is o0 through o8; use '>' to step up and '<' to step down")
    } else if m.contains("instrument") || m.contains("@") {
        Some("FM instruments are defined with '@N' followed by TL/DR/AR/RR parameters on subsequent lines")
    } else {
        None
    }
}

/// Print compilation statistics
fn print_stats(info: &CompileInfo) {
    println!();
    println!("Compilation Statistics:");
    println!("  Parts: {}", info.part_count);
    println!("  Commands: {}", info.command_count);
    println!("  Duration: {:.2} seconds ({} samples)", info.duration_seconds, info.duration_samples);
    println!("  Format: {}", info.format_version);
    
    if !info.chips_used.is_empty() {
        println!("  Chips used:");
        for chip in &info.chips_used {
            println!("    - {}", chip.name());
        }
    }
}

/// Validate an MML file
fn run_validate(args: &Args) -> MmlResult<()> {
    let input = args.input.as_ref()
        .ok_or_else(|| mml2vgm::MmlError::UnsupportedCommand("No input file specified".to_string()))?;

    info!("Validating MML file: {}", input.display());

    // Build compile options
    let options = CompileOptions::new()
        .with_output_format(args.format.into())
        .verbose(args.verbose)
        .debug(args.debug);

    // Create compiler and validate
    let compiler = MmlCompiler::new(options);
    compiler.validate(input)?;

    println!("✓ Validation successful");
    Ok(())
}

/// Main compilation function
fn run_compile(args: &Args) -> MmlResult<CompileResult> {
    let input = args.input.as_ref()
        .ok_or_else(|| mml2vgm::MmlError::UnsupportedCommand("No input file specified".to_string()))?;

    let output_path = determine_output_path(args.input.as_ref(), args.output.as_ref(), args.format.into())?;

    // Build compile options
    let mut options = CompileOptions::new()
        .with_output_format(args.format.into())
        .verbose(args.verbose)
        .debug(args.debug)
        .output_trace(args.trace);

    if let Some(clock) = args.clock_count {
        options = options.clock_count(clock);
    }

    for path in &args.include {
        options = options.add_include_path(path.to_string_lossy().into_owned());
    }

    // Parse target chips if specified
    if !args.chip.is_empty() {
        let mut chips = Vec::new();
        for chip_str in &args.chip {
            let chip: SoundChip = chip_str.parse()?;
            chips.push(chip);
        }
        options = options.with_target_chips(chips);
    }

    info!("Compiling MML file: {}", input.display());
    info!("Output format: {}", args.format);

    // Create compiler and compile
    let compiler = MmlCompiler::new(options);
    let mut result = compiler.compile(input)?;

    // Write output file if path is not stdout
    if output_path.to_string_lossy() != "-" {
        std::fs::write(&output_path, &result.data)
            .map_err(|e| mml2vgm::MmlError::UnsupportedCommand(
                format!("Failed to write output file: {}", e)
            ))?;
        info!("Output written to: {}", output_path.display());
    } else {
        // Write to stdout
        use std::io::Write;
        std::io::stdout().write_all(&result.data)
            .map_err(|e| mml2vgm::MmlError::UnsupportedCommand(
                format!("Failed to write to stdout: {}", e)
            ))?;
    }

    // Update result with actual output path
    result.output_path = Some(output_path.to_string_lossy().into_owned());

    Ok(result)
}

fn main() {
    let args = Args::parse();

    // Handle version flag separately to avoid clap conflict
    if args.version {
        println!("mml2vgm-rs {}", env!("CARGO_PKG_VERSION"));
        println!("A command-line utility for compiling MML files to VGM/XGM/ZGM formats");
        process::exit(0);
    }

    // Initialize logging
    init_logging(args.verbose, args.debug);

    // Handle list commands
    if args.list_chips {
        list_supported_chips();
        process::exit(0);
    }

    if args.list_formats {
        list_supported_formats();
        process::exit(0);
    }

    // Check if we have an input file or need to read from stdin
    if let Some(ref input) = args.input {
        if !input.exists() {
            error!("Input file not found: {}", input.display());
            process::exit(1);
        }
        debug!("Input file exists: {}", input.display());

        // If the input is a pre-compiled VGM/XGM/ZGM file, skip compilation.
        if is_compiled_audio_file(input) {
            if !args.play && args.export_wav.is_none() {
                error!("Input is a compiled audio file. Use --play or --export-wav.");
                process::exit(1);
            }
            let vgm_data = match std::fs::read(input) {
                Ok(d) => d,
                Err(e) => { error!("Failed to read file: {}", e); process::exit(1); }
            };
            render_and_play(&vgm_data, args.play, args.export_wav.as_ref());
            process::exit(0);
        }
    }

    // Handle validation or compilation
    if args.check {
        match run_validate(&args) {
            Ok(()) => { info!("Validation successful"); process::exit(0); }
            Err(e) => {
                print_diagnostic(args.input.as_deref(), &e);
                process::exit(1);
            }
        }
    } else {
        match run_compile(&args) {
            Ok(result) => {
                info!("Compilation successful");

                if !result.warnings.is_empty() {
                    warn!("Compilation completed with {} warnings", result.warnings.len());
                    for warning in &result.warnings {
                        warn!("Warning at {}: {}", warning.position, warning.message);
                    }
                }

                print_stats(&result.info);

                if (args.play || args.export_wav.is_some()) && !result.data.is_empty() {
                    render_and_play(&result.data, args.play, args.export_wav.as_ref());
                }

                process::exit(0);
            }
            Err(e) => {
                print_diagnostic(args.input.as_deref(), &e);
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_arg_conversion() {
        assert_eq!(
            OutputFormat::from(FormatArg::Vgm),
            OutputFormat::VGM
        );
        assert_eq!(
            OutputFormat::from(FormatArg::Xgm),
            OutputFormat::XGM
        );
    }

    #[test]
    fn test_output_path_determination() {
        let input = PathBuf::from("test.gwi");
        
        let path = determine_output_path(
            Some(&input),
            None,
            OutputFormat::VGM
        ).unwrap();
        
        assert_eq!(path, PathBuf::from("test.vgm"));

        let path = determine_output_path(
            Some(&input),
            None,
            OutputFormat::XGM
        ).unwrap();
        
        assert_eq!(path, PathBuf::from("test.xgm"));

        let path = determine_output_path(
            Some(&input),
            None,
            OutputFormat::XGM2
        ).unwrap();

        assert_eq!(path, PathBuf::from("test.xgm2"));

        let custom = PathBuf::from("custom.vgm");
        let path = determine_output_path(
            Some(&input),
            Some(&custom),
            OutputFormat::VGM
        ).unwrap();
        
        assert_eq!(path, custom);
    }
}
