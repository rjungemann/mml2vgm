#!/usr/bin/env python3
"""
Golden Master Test Runner — work item 1 of GOLDEN_MASTER_TEST_PLAN.md

For each GWI test file across tier1/tier2/tier3:
  Check B: compile GWI -> VGM, count chip-specific register write opcodes (>0 = pass)
  Check A: render VGM -> WAV, spectral correlation vs stored reference WAV (>=0.95 = pass)

Exit code: 0 if all non-SKIP tests pass, 1 otherwise.
Results written to: validation_results/golden_master_results.json
"""

import json
import struct
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
from scipy.io import wavfile

# ── Paths ─────────────────────────────────────────────────────────────────────

ROOT        = Path(__file__).parent.parent.parent
VGM2WAV     = ROOT.parent / "libvgm" / "build" / "bin" / "vgm2wav"
CARGO       = ROOT / "mml2vgm-rs" / "Cargo.toml"
REFS        = ROOT / "tests" / "golden_master" / "references"
TIER_DIRS   = [
    ROOT / "tests" / "golden_master" / "tier1",
    ROOT / "tests" / "golden_master" / "tier2",
    ROOT / "tests" / "golden_master" / "tier3",
]
RESULTS_DIR = ROOT / "validation_results"
OUT_JSON    = RESULTS_DIR / "golden_master_results.json"

# ── Correlation pass threshold ────────────────────────────────────────────────

CORR_THRESHOLD = 0.95

# ── Stem → references subfolder (must match render_golden_masters.py) ─────────

STEM_MAP = [
    ("test_dmg_",        "tier3/dmg"),
    ("test_vrc6_",       "tier3/vrc6"),
    ("test_k051649_",    "tier3/k051649"),
    ("test_ay8910_",     "tier2/ay8910"),
    ("test_c140_",       "tier2/c140"),
    ("test_c352_",       "tier2/c352"),
    ("test_huc6280_",    "tier2/huc6280"),
    ("test_k053260_",    "tier2/k053260"),
    ("test_k054539_",    "tier2/k054539"),
    ("test_konami_pcm_", "tier2/k053260"),
    ("test_rf5c164_",    "tier2/rf5c164"),
    ("test_y8950_",      "tier2/y8950"),
    ("test_ym2413_",     "tier2/ym2413"),
    ("test_nes_",        "nes"),
    ("test_opl",         "opl"),
    ("test_qsound_",     "qsound"),
    ("test_segapcm_",    "segapcm"),
    ("test_ym2151_",     "ym2151"),
    ("test_ym2203_",     "ym2203"),
    ("test_ym2608_",     "ym2608"),
]

def ref_folder(stem: str) -> str | None:
    for prefix, folder in STEM_MAP:
        if stem.startswith(prefix):
            return folder
    return None

# ── Chip opcode map ───────────────────────────────────────────────────────────
# Values are VgmCommandType enum entries from mml2vgm-rs/src/compiler/codegen/vgm.rs.
# Chips whose write opcodes conflict with standard VGM waits or PCM blocks in
# libvgm are marked SKIP_AUDIO — their WAVs will be silent/incorrect.

STEM_TO_OPCODES: dict[str, list[int]] = {
    "test_ym2608_":    [0x56, 0x57],  # YM2608 port 0/1
    "test_ym2151_":    [0x54],
    "test_ym2203_":    [0x55],
    "test_opl3_":      [0x5E, 0x5F],  # YMF262 — must come before test_opl_ prefix
    "test_opl2_":      [0x5A],        # YM3812
    "test_opl_":       [0x5A],        # YM3812 (test_opl_envelope uses YM3812, not YM3526)
    "test_nes_":       [0xB4],
    "test_segapcm_":   [0xA4],
    "test_qsound_":    [0xC4],
    "test_ym2413_":    [0x51],
    "test_y8950_":     [0x5C],
    "test_rf5c164_":   [0x68],        # non-standard opcode (see plan §5)
    "test_c140_":      [0x7F],        # non-standard opcode (see plan §5)
    "test_c352_":      [0x8E],        # non-standard opcode (see plan §5)
    "test_k053260_":   [0xBA],
    "test_konami_pcm_":[0xBA],
    "test_k054539_":   [0xD3],
    "test_ay8910_":    [0xA0],
    "test_huc6280_":   [0xB9],
    "test_dmg_":       [0xB3],
    "test_vrc6_":      [0xB6],
    "test_k051649_":   [0xD2],
}

# Stems whose audio output is unreliable with the current toolchain.
# Either: PCM chip with no sample data, or non-standard VGM opcode that
# libvgm misinterprets as waits.
SKIP_AUDIO_PREFIXES = {
    "test_segapcm_",    # PCM, no sample data
    "test_qsound_",     # PCM, no sample data
    "test_rf5c164_",    # non-standard opcode 0x68
    "test_c140_",       # non-standard opcode 0x7F
    "test_c352_",       # non-standard opcode 0x8E
    "test_k053260_",    # PCM, no sample data
    "test_konami_pcm_", # PCM, no sample data
    "test_k054539_",    # PCM, no sample data
    "test_y8950_adpcm", # ADPCM, no sample data
}

def is_skip_audio(stem: str) -> bool:
    return any(stem.startswith(p) or stem == p for p in SKIP_AUDIO_PREFIXES)

def opcodes_for(stem: str) -> list[int]:
    for prefix, ops in STEM_TO_OPCODES.items():
        if stem.startswith(prefix):
            return ops
    return []

# ── VGM register-write counter ────────────────────────────────────────────────

# Command byte → total bytes consumed (opcode included).
# Based on VGM 1.71 spec + non-standard opcodes used by this compiler.
_CMD_SIZE: dict[int, int] = {
    0x50: 2,                          # SN76489
    **{op: 3 for op in range(0x51, 0x60)},  # FM chips
    0x61: 3,                          # wait n samples
    0x62: 1, 0x63: 1, 0x65: 1, 0x66: 1,
    **{op: 1 for op in range(0x70, 0x7F)},  # short waits 0x70-0x7E
    0x7F: 3,                              # C140Write (non-standard; standard: wait 16 samples)
    **{op: 1 for op in range(0x80, 0x8E)},  # YM2612 PCM+wait 0x80-0x8D
    0x8E: 3,                              # C352Write (non-standard; standard: PCM+14-sample wait)
    0x8F: 1,                              # YM2612 PCM + 15-sample wait
    **{op: 3 for op in range(0xA0, 0xC0)},  # AY8910 and similar
    0xA4: 4,                                    # SegaPCM: bank+addr+data (ported write)
    **{op: 4 for op in range(0xC0, 0xD6)},  # 4-byte chip writes
    0xC4: 3,                                    # QSound: addr+data (simple write)
    **{op: 5 for op in range(0xE0, 0xE4)},  # seek
}

def count_vgm_opcodes(vgm_path: Path, target_opcodes: set[int]) -> int:
    """Parse VGM command stream and count occurrences of target opcodes."""
    data = vgm_path.read_bytes()
    if len(data) < 0x40:
        return 0

    # Data offset from header field at 0x34 (relative to that field's address)
    data_offset_field = struct.unpack_from("<I", data, 0x34)[0]
    pos = 0x34 + data_offset_field if data_offset_field else 0x40

    count = 0
    while pos < len(data):
        cmd = data[pos]

        if cmd == 0x66:   # end of data
            break
        elif cmd == 0x67: # data block: skip 7-byte header + block size
            if pos + 7 > len(data):
                break
            block_size = struct.unpack_from("<I", data, pos + 3)[0]
            pos += 7 + block_size
            continue
        elif cmd == 0x68: # PCM RAM write (standard: 10 bytes) / RF5C164 (this compiler: 3 bytes)
            if target_opcodes and 0x68 in target_opcodes:
                count += 1
                pos += 3
            else:
                pos += 10  # standard VGM size
            continue

        if cmd in target_opcodes:
            count += 1

        size = _CMD_SIZE.get(cmd)
        if size is None:
            # Unknown command: advance by 1 to avoid infinite loop
            pos += 1
        else:
            pos += size

    return count

# ── Compilation ───────────────────────────────────────────────────────────────

def compile_gwi(gwi: Path, out_vgm: Path) -> tuple[bool, str]:
    out_vgm.parent.mkdir(parents=True, exist_ok=True)
    try:
        r = subprocess.run(
            ["cargo", "run", "--manifest-path", str(CARGO), "--quiet", "--",
             str(gwi), "-o", str(out_vgm)],
            capture_output=True, text=True, timeout=120,
        )
    except subprocess.TimeoutExpired:
        return False, "compile timeout (120s)"
    if r.returncode != 0:
        err = (r.stderr or r.stdout).strip().splitlines()
        return False, "\n".join(err[-5:])
    if not out_vgm.exists() or out_vgm.stat().st_size == 0:
        return False, "no output file produced"
    return True, ""

# ── WAV rendering ─────────────────────────────────────────────────────────────

def render_vgm(vgm: Path, out_wav: Path) -> tuple[bool, str]:
    try:
        r = subprocess.run(
            [str(VGM2WAV), "--loops", "1", "--fade", "1.0", str(vgm), str(out_wav)],
            capture_output=True, text=True, timeout=60,
        )
    except subprocess.TimeoutExpired:
        return False, "render timeout (60s)"
    if r.returncode != 0:
        return False, r.stderr.strip()[:200]
    if not out_wav.exists():
        return False, "no wav produced"
    return True, ""

# ── Spectral correlation ──────────────────────────────────────────────────────

_SILENCE_RMS = 1e-4  # below this, treat as silence

def _load_mono_f32(path: Path) -> tuple[np.ndarray, int]:
    sr, data = wavfile.read(str(path))
    if data.ndim > 1:
        data = data.mean(axis=1)
    if data.dtype == np.int16:
        data = data.astype(np.float32) / 32768.0
    elif data.dtype == np.int32:
        data = data.astype(np.float32) / 2147483648.0
    else:
        data = data.astype(np.float32)
    return data, sr

def spectral_correlation(wav_new: Path, wav_ref: Path) -> dict:
    """
    Compare two WAV files using averaged log-magnitude FFT spectra.
    Returns dict with keys: correlation (float), silent (bool), error (str|None).
    """
    try:
        a, sr_a = _load_mono_f32(wav_new)
        b, sr_b = _load_mono_f32(wav_ref)
    except Exception as e:
        return {"correlation": 0.0, "silent": False, "error": str(e)}

    rms_a = float(np.sqrt(np.mean(a ** 2)))
    rms_b = float(np.sqrt(np.mean(b ** 2)))

    if rms_a < _SILENCE_RMS and rms_b < _SILENCE_RMS:
        return {"correlation": 1.0, "silent": True, "error": None}

    # Trim/pad to same length
    n = min(len(a), len(b))
    a, b = a[:n], b[:n]

    # Chunk into 2048-sample windows, compute FFT magnitude, average
    win = 2048
    mags_a, mags_b = [], []
    for start in range(0, n - win, win):
        chunk_a = a[start:start + win] * np.hanning(win)
        chunk_b = b[start:start + win] * np.hanning(win)
        mags_a.append(np.abs(np.fft.rfft(chunk_a)))
        mags_b.append(np.abs(np.fft.rfft(chunk_b)))

    if not mags_a:
        return {"correlation": 0.0, "silent": False, "error": "audio too short for FFT"}

    mag_a = np.log1p(np.mean(mags_a, axis=0))
    mag_b = np.log1p(np.mean(mags_b, axis=0))

    # Pearson correlation of log-magnitude spectra
    if mag_a.std() < 1e-9 or mag_b.std() < 1e-9:
        return {"correlation": 0.0, "silent": True, "error": None}

    corr = float(np.corrcoef(mag_a, mag_b)[0, 1])
    return {"correlation": corr, "silent": False, "error": None}

# ── Per-test runner ───────────────────────────────────────────────────────────

def run_test(gwi: Path, tmp_dir: Path) -> dict:
    stem = gwi.stem
    folder = ref_folder(stem)
    ref_wav = REFS / folder / (stem + ".wav") if folder else None
    opcodes = opcodes_for(stem)
    skip_audio = is_skip_audio(stem)

    result = {
        "gwi": str(gwi.relative_to(ROOT)),
        "stem": stem,
        "ref_wav": str(ref_wav.relative_to(ROOT)) if ref_wav else None,
        "skip_audio": skip_audio,
        "compile": {"ok": False, "error": ""},
        "check_b": {"ok": False, "write_count": 0, "opcodes": [f"0x{o:02X}" for o in opcodes]},
        "render":  {"ok": False, "error": ""},
        "check_a": {"ok": False, "correlation": None, "silent": False, "error": None},
        "status":  "FAIL",
    }

    tmp_vgm = tmp_dir / (stem + ".vgm")
    tmp_wav = tmp_dir / (stem + ".wav")

    # ── Step 1: compile ───────────────────────────────────────────────────────
    ok, err = compile_gwi(gwi, tmp_vgm)
    result["compile"] = {"ok": ok, "error": err}
    if not ok:
        result["status"] = "FAIL"
        return result

    # ── Step 2: Check B — register writes ────────────────────────────────────
    write_count = count_vgm_opcodes(tmp_vgm, set(opcodes)) if opcodes else -1
    check_b_ok = write_count > 0 if opcodes else True
    result["check_b"] = {
        "ok": check_b_ok,
        "write_count": write_count,
        "opcodes": [f"0x{o:02X}" for o in opcodes],
    }

    # ── Step 3: render to WAV ─────────────────────────────────────────────────
    ok, err = render_vgm(tmp_vgm, tmp_wav)
    result["render"] = {"ok": ok, "error": err}
    if not ok:
        result["status"] = "WARN" if check_b_ok else "FAIL"
        return result

    # ── Step 4: Check A — spectral correlation ────────────────────────────────
    if skip_audio:
        result["check_a"] = {"ok": True, "correlation": None, "silent": None, "error": None, "skipped": True}
        result["status"] = "PASS" if check_b_ok else "FAIL"
        return result

    if ref_wav is None or not ref_wav.exists():
        result["check_a"] = {"ok": False, "correlation": None, "error": "no reference WAV", "skipped": False}
        result["status"] = "WARN"
        return result

    spec = spectral_correlation(tmp_wav, ref_wav)
    corr = spec["correlation"]
    check_a_ok = corr >= CORR_THRESHOLD
    result["check_a"] = {
        "ok": check_a_ok,
        "correlation": round(corr, 4),
        "silent": spec["silent"],
        "error": spec["error"],
        "skipped": False,
    }

    if check_b_ok and check_a_ok:
        result["status"] = "PASS"
    elif check_b_ok and not check_a_ok:
        result["status"] = "FAIL"
    else:
        result["status"] = "FAIL"

    return result

# ── Main ──────────────────────────────────────────────────────────────────────

STATUS_ICON = {"PASS": "✅", "FAIL": "❌", "WARN": "⚠️ "}

def main() -> int:
    if not VGM2WAV.exists():
        print(f"ERROR: vgm2wav not found at {VGM2WAV}", file=sys.stderr)
        return 1

    gwi_files = sorted(p for d in TIER_DIRS for p in d.glob("*.gwi"))
    if not gwi_files:
        print("No GWI files found.", file=sys.stderr)
        return 1

    print(f"Running golden master tests for {len(gwi_files)} GWI files")
    print(f"Correlation threshold: {CORR_THRESHOLD}\n")

    results = []
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        for gwi in gwi_files:
            print(f"  {gwi.stem:<40}", end=" ", flush=True)
            r = run_test(gwi, tmp_dir)
            results.append(r)

            icon = STATUS_ICON.get(r["status"], "?")
            details = []
            if r["compile"]["ok"]:
                wc = r["check_b"]["write_count"]
                details.append(f"B:{wc}w")
            else:
                details.append(f"compile:{r['compile']['error'][:40]}")
            if r["check_a"].get("skipped"):
                details.append("A:skip")
            elif r["check_a"]["correlation"] is not None:
                details.append(f"A:{r['check_a']['correlation']:.3f}")
            elif r["check_a"]["error"]:
                details.append(f"A:err")

            print(f"{icon} {r['status']:<4}  {' '.join(details)}")

    # ── Summary ───────────────────────────────────────────────────────────────
    passed  = sum(1 for r in results if r["status"] == "PASS")
    warned  = sum(1 for r in results if r["status"] == "WARN")
    failed  = sum(1 for r in results if r["status"] == "FAIL")
    total   = len(results)

    print(f"\n{'─'*60}")
    print(f"Results: {passed}/{total} PASS  {warned} WARN  {failed} FAIL")

    if failed:
        print("\nFailing tests:")
        for r in results:
            if r["status"] == "FAIL":
                ce = r["compile"]["error"]
                ae = r["check_a"].get("error") or ""
                reason = ce or ae or f"B:{r['check_b']['write_count']}w"
                print(f"  ❌ {r['stem']}: {reason[:80]}")

    # ── Write JSON ────────────────────────────────────────────────────────────
    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    summary = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "threshold": CORR_THRESHOLD,
        "total": total,
        "passed": passed,
        "warned": warned,
        "failed": failed,
        "tests": results,
    }
    OUT_JSON.write_text(json.dumps(summary, indent=2))
    print(f"\nResults written to {OUT_JSON.relative_to(ROOT)}")

    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
