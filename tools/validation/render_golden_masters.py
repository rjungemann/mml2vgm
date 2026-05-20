#!/usr/bin/env python3
"""
Batch render all compiled VGM files to WAV golden master references using vgm2wav (libvgm).

Output layout:
  tests/golden_master/references/
    nes/          <- tier 1
    opl/
    qsound/
    segapcm/
    ym2151/
    ym2203/
    ym2608/
    tier2/
      ay8910/
      c140/
      c352/
      huc6280/
      k053260/
      k054539/
      rf5c164/
      y8950/
      ym2413/
"""

import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

VGM2WAV = Path(__file__).parent.parent.parent.parent / "libvgm" / "build" / "bin" / "vgm2wav"
PROJECT_ROOT = Path(__file__).parent.parent.parent
REFS = PROJECT_ROOT / "tests" / "golden_master" / "references"
RESULTS = PROJECT_ROOT / "validation_results"

SAMPLE_RATE = 44100
LOOPS = 1
FADE = 1.0

# Map from stem prefix -> references subfolder (relative to REFS)
# Order matters: more-specific prefixes first.
STEM_MAP = [
    # Tier 3 chips
    ("test_dmg_",           "tier3/dmg"),
    ("test_vrc6_",          "tier3/vrc6"),
    ("test_k051649_",       "tier3/k051649"),
    # Tier 2 chips
    ("test_ay8910_",        "tier2/ay8910"),
    ("test_c140_",          "tier2/c140"),
    ("test_c352_",          "tier2/c352"),
    ("test_huc6280_",       "tier2/huc6280"),
    ("test_k053260_",       "tier2/k053260"),
    ("test_k054539_",       "tier2/k054539"),
    ("test_konami_pcm_",    "tier2/k053260"),   # shared pitch test
    ("test_rf5c164_",       "tier2/rf5c164"),
    ("test_y8950_",         "tier2/y8950"),
    ("test_ym2413_",        "tier2/ym2413"),
    # Tier 1 chips
    ("test_nes_",           "nes"),
    ("test_opl",            "opl"),             # opl, opl2, opl3
    ("test_qsound_",        "qsound"),
    ("test_segapcm_",       "segapcm"),
    ("test_ym2151_",        "ym2151"),
    ("test_ym2203_",        "ym2203"),
    ("test_ym2608_",        "ym2608"),
]

def classify(stem: str) -> str | None:
    for prefix, folder in STEM_MAP:
        if stem.startswith(prefix):
            return folder
    return None

def render(vgm: Path, out_wav: Path) -> dict:
    out_wav.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        str(VGM2WAV),
        f"--samplerate", str(SAMPLE_RATE),
        f"--loops", str(LOOPS),
        f"--fade", str(FADE),
        str(vgm),
        str(out_wav),
    ]
    result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
    return {
        "vgm": str(vgm.relative_to(PROJECT_ROOT)),
        "wav": str(out_wav.relative_to(PROJECT_ROOT)),
        "ok": result.returncode == 0,
        "size_bytes": out_wav.stat().st_size if out_wav.exists() else 0,
        "stderr": result.stderr.strip(),
    }

def collect_vgms() -> list[Path]:
    vgms = []
    # Tier 1: flat in validation_results/
    for p in sorted(RESULTS.glob("test_*.vgm")):
        vgms.append(p)
    # Tier 2: in validation_results/phase2/
    for p in sorted((RESULTS / "phase2").glob("test_*.vgm")):
        vgms.append(p)
    # Tier 3: in validation_results/tier3/
    for p in sorted((RESULTS / "tier3").glob("test_*.vgm")):
        vgms.append(p)
    return vgms

def main():
    if not VGM2WAV.exists():
        print(f"ERROR: vgm2wav not found at {VGM2WAV}", file=sys.stderr)
        sys.exit(1)

    vgms = collect_vgms()
    print(f"Found {len(vgms)} VGM files to render")

    results = []
    skipped = []

    for vgm in vgms:
        folder = classify(vgm.stem)
        if folder is None:
            skipped.append(str(vgm.name))
            continue

        out_wav = REFS / folder / (vgm.stem + ".wav")
        print(f"  {vgm.name} -> references/{folder}/", end=" ", flush=True)
        r = render(vgm, out_wav)
        results.append(r)
        if r["ok"]:
            kb = r["size_bytes"] // 1024
            print(f"OK ({kb} KB)")
        else:
            print(f"FAILED")
            if r["stderr"]:
                print(f"    {r['stderr'][:200]}")

    passed = sum(1 for r in results if r["ok"])
    failed = sum(1 for r in results if not r["ok"])

    print(f"\nResults: {passed} rendered, {failed} failed, {len(skipped)} skipped")
    if skipped:
        print(f"Skipped (no mapping): {', '.join(skipped)}")

    summary = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "vgm2wav": str(VGM2WAV),
        "sample_rate": SAMPLE_RATE,
        "loops": LOOPS,
        "fade_seconds": FADE,
        "passed": passed,
        "failed": failed,
        "skipped": len(skipped),
        "files": results,
    }
    out_json = REFS / "generation_results.json"
    out_json.write_text(json.dumps(summary, indent=2))
    print(f"Summary written to {out_json.relative_to(PROJECT_ROOT)}")

    sys.exit(0 if failed == 0 else 1)

if __name__ == "__main__":
    main()
