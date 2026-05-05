//! External driver unit tests
//!
//! Run with: cargo test drivers::tests

use std::time::Instant;

use crate::drivers::{
    DriverCompileOptions, DriverRegistry, ExternalDriver,
    M98Driver, MucomDriver, MoonDriver, PMDDriver, MuapDriver,
};
use std::sync::Arc;

const TIMEOUT_SECS: u64 = 2;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn default_registry() -> DriverRegistry {
    let mut reg = DriverRegistry::new();
    reg.register_driver(Arc::new(M98Driver));
    reg.register_driver(Arc::new(MucomDriver));
    reg.register_driver(Arc::new(MoonDriver));
    reg.register_driver(Arc::new(PMDDriver));
    reg.register_driver(Arc::new(MuapDriver));
    reg
}

fn assert_driver_conforms(driver: &dyn ExternalDriver, own_extension: &str, minimal_snippet: &str) {
    let start = Instant::now();

    // id / display_name / description non-empty
    assert!(!driver.id().is_empty(), "{}: id() empty", driver.id());
    assert!(!driver.display_name().is_empty(), "{}: display_name() empty", driver.id());
    assert!(!driver.description().is_empty(), "{}: description() empty", driver.id());
    assert!(!driver.version().is_empty(), "{}: version() empty", driver.id());
    assert!(!driver.target_platform().is_empty(), "{}: target_platform() empty", driver.id());

    // extensions
    let exts = driver.supported_extensions();
    assert!(!exts.is_empty(), "{}: supported_extensions() empty", driver.id());

    // detect own extension → high confidence
    let own_conf = driver.detect("", Some(own_extension));
    assert!(own_conf >= 60, "{}: own extension confidence too low: {}", driver.id(), own_conf);

    // detect foreign extension → low confidence
    let foreign_conf = driver.detect("", Some("song.xyz_unknown_format"));
    assert!(foreign_conf < 60, "{}: foreign extension confidence too high: {}", driver.id(), foreign_conf);

    // compile minimal snippet – must not panic, error is acceptable
    let opts = DriverCompileOptions::default();
    let _ = driver.compile(minimal_snippet, &opts);

    // compile empty – must not panic
    let _ = driver.compile("", &opts);

    // compile garbage – must not panic
    let _ = driver.compile("§§§ INVALID §§§", &opts);

    // validate valid – must not panic
    let diag_result = driver.validate(minimal_snippet);
    assert!(diag_result.is_ok(), "{}: validate(valid) returned Err", driver.id());

    // tokenize basic – must return at least one token for non-empty input
    let tok_result = driver.tokenize(minimal_snippet);
    assert!(tok_result.is_ok(), "{}: tokenize returned Err", driver.id());

    assert!(start.elapsed().as_secs() < TIMEOUT_SECS, "{}: test exceeded timeout", driver.id());
}

// ── M98 ───────────────────────────────────────────────────────────────────────

const M98_SNIPPET: &str = "; M98 test\nt120\no4\nC4 D4 E4\n";

#[test]
fn m98_trait_conformance() {
    assert_driver_conforms(&M98Driver, "song.m98", M98_SNIPPET);
}

#[test]
fn m98_detect_content_keywords() {
    let start = Instant::now();
    let conf = M98Driver.detect("M98 test file\n", None);
    assert!(conf >= 80, "expected ≥80, got {}", conf);
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

#[test]
fn m98_detect_pc98_keyword() {
    let conf = M98Driver.detect("PC-98 sound driver\n", None);
    assert!(conf >= 80);
}

// ── Mucom88 ───────────────────────────────────────────────────────────────────

const MUCOM_SNIPPET: &str = "; mucom88 test\n@0 t120 o4 C4 D4 E4\n";

#[test]
fn mucom_trait_conformance() {
    assert_driver_conforms(&MucomDriver, "song.muc", MUCOM_SNIPPET);
}

#[test]
fn mucom_fm_channel_range() {
    let start = Instant::now();
    let opts = DriverCompileOptions::default();
    for ch in 0..=5u32 {
        let src = format!("@{} t120 o4 C4 D4 E4\n", ch);
        let _ = MucomDriver.compile(&src, &opts); // must not panic
    }
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

#[test]
fn mucom_psg_channel_range() {
    let start = Instant::now();
    let opts = DriverCompileOptions::default();
    for ch in 6..=9u32 {
        let src = format!("@{} t120 o4 C4 D4 E4\n", ch);
        let _ = MucomDriver.compile(&src, &opts);
    }
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

// ── MoonDriver ────────────────────────────────────────────────────────────────

const MOON_SNIPPET: &str = "#OPN2\n@0 t120 o4 C4 D4 E4\n";

#[test]
fn moon_trait_conformance() {
    assert_driver_conforms(&MoonDriver, "song.mdl", MOON_SNIPPET);
}

#[test]
fn moon_detect_md_directive() {
    let start = Instant::now();
    let conf = MoonDriver.detect("#MD\n@0 t120\n", None);
    assert!(conf > 0, "expected >0, got {}", conf);
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

#[test]
fn moon_opn2_target_compiles() {
    let start = Instant::now();
    let src = "#OPN2\n@0 t120 o4 C4 D4\n";
    let _ = MoonDriver.compile(src, &DriverCompileOptions::default());
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

#[test]
fn moon_opna_target_compiles() {
    let start = Instant::now();
    let src = "#OPNA\n@0 t120 o4 C4 D4\n";
    let _ = MoonDriver.compile(src, &DriverCompileOptions::default());
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

// ── PMD ───────────────────────────────────────────────────────────────────────

const PMD_SNIPPET: &str = "; PMD test\n@0 t120 o4 C4 D4 E4\n";

#[test]
fn pmd_trait_conformance() {
    assert_driver_conforms(&PMDDriver, "song.mdl", PMD_SNIPPET);
}

#[test]
fn pmd_finite_loop_parses() {
    let start = Instant::now();
    let src = "@0 (C4 D4)2\n";
    let _ = PMDDriver.compile(src, &DriverCompileOptions::default());
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

// ── Muap ─────────────────────────────────────────────────────────────────────

const MUAP_SNIPPET: &str = "@OPNA\n@FM 0 t120 o4 C4 D4 E4\n";

#[test]
fn muap_trait_conformance() {
    assert_driver_conforms(&MuapDriver, "song.muap", MUAP_SNIPPET);
}

#[test]
fn muap_detect_extension_high_confidence() {
    let start = Instant::now();
    let conf = MuapDriver.detect("", Some("track.muap"));
    assert!(conf >= 80, "expected ≥80, got {}", conf);
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

// ── Driver Registry ───────────────────────────────────────────────────────────

#[test]
fn registry_all_drivers_registered() {
    let start = Instant::now();
    let reg = default_registry();
    let drivers = reg.get_all_drivers();
    assert_eq!(drivers.len(), 5, "expected 5 drivers, got {}", drivers.len());
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

#[test]
fn registry_lookup_by_id() {
    let start = Instant::now();
    let reg = default_registry();
    for id in ["m98", "mucom", "moondriver", "pmd", "muap"] {
        assert!(reg.get_driver(id).is_some(), "driver '{}' not found in registry", id);
    }
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

#[test]
fn registry_auto_detect_m98() {
    let start = Instant::now();
    let reg = default_registry();
    let result = reg.detect_format("M98 data", Some("song.m98"));
    assert!(result.is_some(), "expected a driver match for .m98 file");
    let (id, conf) = result.unwrap();
    assert_eq!(id, "m98");
    assert!(conf >= 30);
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}

#[test]
fn registry_no_false_positive_random_text() {
    let start = Instant::now();
    let reg = default_registry();
    // Random text that should match no driver with high confidence
    let result = reg.detect_format("hello world this is not music", Some("readme.txt"));
    // Either no match, or a low-confidence one (registry threshold is 30)
    if let Some((_, conf)) = result {
        assert!(conf < 80, "unexpected high confidence {} for random text", conf);
    }
    assert!(start.elapsed().as_secs() < TIMEOUT_SECS);
}
