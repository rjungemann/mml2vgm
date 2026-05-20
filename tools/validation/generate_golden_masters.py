#!/usr/bin/env python3
"""
Generate Golden Master Audio Files

Renders compiled VGM files to WAV format for each Tier 2 chip using MAME vgmplay.
Creates reference audio files for spectral analysis validation.
"""

import subprocess
import sys
from pathlib import Path
from datetime import datetime
import json
import os

# Configuration
PROJECT_ROOT = Path(__file__).parent.parent.parent
VGM_SOURCE_DIR = PROJECT_ROOT / "validation_results" / "phase2"
GOLDEN_MASTER_DIR = PROJECT_ROOT / "tests" / "golden_master" / "references" / "tier2"
MAME_BIN = "mame"

# Tier 2 chip configurations
TIER2_CHIPS = {
    "YM2413": {
        "name": "YM2413 (OPLL)",
        "reference_emulator": "Mednafen",
        "sample_rate": 44100,
    },
    "Y8950": {
        "name": "Y8950 (OPL + ADPCM)",
        "reference_emulator": "DOSBox-X",
        "sample_rate": 44100,
    },
    "RF5C164": {
        "name": "RF5C164 (Sega CD)",
        "reference_emulator": "Mednafen",
        "sample_rate": 44100,
    },
    "C140": {
        "name": "C140 (Namco)",
        "reference_emulator": "MAME",
        "sample_rate": 44100,
    },
    "C352": {
        "name": "C352 (Namco System 21/22)",
        "reference_emulator": "MAME",
        "sample_rate": 44100,
    },
    "K053260": {
        "name": "K053260 (Konami PCM)",
        "reference_emulator": "MAME",
        "sample_rate": 44100,
    },
    "K054539": {
        "name": "K054539 (Konami Enhanced PCM)",
        "reference_emulator": "MAME",
        "sample_rate": 44100,
    },
    "AY8910": {
        "name": "AY8910 (PSG)",
        "reference_emulator": "Mednafen",
        "sample_rate": 44100,
    },
    "HuC6280": {
        "name": "HuC6280 (PC Engine)",
        "reference_emulator": "Mednafen",
        "sample_rate": 44100,
    },
}

# VGM file to chip mapping (for reference)
VGM_TO_CHIP = {
    "test_ym2413_patches.vgm": "YM2413",
    "test_ym2413_custom.vgm": "YM2413",
    "test_ym2413_rhythm.vgm": "YM2413",
    "test_y8950_opl.vgm": "Y8950",
    "test_y8950_adpcm.vgm": "Y8950",
    "test_rf5c164_basic.vgm": "RF5C164",
    "test_rf5c164_pitch.vgm": "RF5C164",
    "test_c140_basic.vgm": "C140",
    "test_c140_loop.vgm": "C140",
    "test_c352_basic.vgm": "C352",
    "test_c352_filter.vgm": "C352",
    "test_k053260_basic.vgm": "K053260",
    "test_konami_pcm_pitch.vgm": "K053260",  # Also used by K054539
    "test_k054539_basic.vgm": "K054539",
    "test_ay8910_envelope.vgm": "AY8910",
    "test_ay8910_wavetable.vgm": "AY8910",
    "test_huc6280_wavetable.vgm": "HuC6280",
}

def render_vgm_to_wav(vgm_path: Path, wav_path: Path) -> dict:
    """
    Render VGM to WAV using MAME vgmplay.
    
    Returns dict with status, file sizes, and metadata.
    """
    if not vgm_path.exists():
        return {
            "vgm": str(vgm_path),
            "status": "SKIP",
            "reason": f"VGM file not found"
        }
    
    print(f"  {vgm_path.name:40} ", end="", flush=True)
    
    try:
        # Use MAME vgmplay to render VGM to WAV
        result = subprocess.run(
            [MAME_BIN, "vgmplay", str(vgm_path), "-wavwrite", str(wav_path)],
            capture_output=True,
            text=True,
            timeout=120
        )
        
        if result.returncode != 0:
            # Check if it's just a warning
            if wav_path.exists() and wav_path.stat().st_size > 0:
                wav_size = wav_path.stat().st_size
                print(f"✓ ({wav_size/1024:.1f} KB, with warnings)")
                return {
                    "vgm": str(vgm_path),
                    "wav": str(wav_path),
                    "status": "PASS",
                    "reason": "Rendered with warnings",
                    "wav_size_bytes": wav_size,
                    "vgm_size_bytes": vgm_path.stat().st_size
                }
            else:
                print(f"✗ Rendering failed")
                return {
                    "vgm": str(vgm_path),
                    "wav": str(wav_path),
                    "status": "FAIL",
                    "reason": f"MAME error: {result.stderr[:200]}"
                }
        
        if not wav_path.exists() or wav_path.stat().st_size == 0:
            print(f"✗ No WAV output")
            return {
                "vgm": str(vgm_path),
                "wav": str(wav_path),
                "status": "FAIL",
                "reason": "WAV file was not created"
            }
        
        wav_size = wav_path.stat().st_size
        print(f"✓ ({wav_size/1024:.1f} KB)")
        
        return {
            "vgm": str(vgm_path),
            "wav": str(wav_path),
            "status": "PASS",
            "reason": "Successfully rendered",
            "wav_size_bytes": wav_size,
            "vgm_size_bytes": vgm_path.stat().st_size
        }
    
    except subprocess.TimeoutExpired:
        print(f"✗ Timeout")
        return {
            "vgm": str(vgm_path),
            "wav": str(wav_path),
            "status": "FAIL",
            "reason": "Rendering timeout (120s)"
        }
    except FileNotFoundError:
        print(f"✗ MAME not found")
        return {
            "vgm": str(vgm_path),
            "wav": str(wav_path),
            "status": "FAIL",
            "reason": f"MAME not found in PATH"
        }
    except Exception as e:
        print(f"✗ {str(e)[:40]}")
        return {
            "vgm": str(vgm_path),
            "wav": str(wav_path),
            "status": "FAIL",
            "reason": f"Error: {str(e)}"
        }


def main():
    """Generate golden master audio files for Tier 2 chips."""
    print("=" * 70)
    print("TIER 2 GOLDEN MASTER GENERATION")
    print("VGM to WAV Rendering (MAME vgmplay)")
    print("=" * 70)
    print()
    
    # Verify directories exist
    if not VGM_SOURCE_DIR.exists():
        print(f"ERROR: VGM source directory not found: {VGM_SOURCE_DIR}")
        return 1
    
    GOLDEN_MASTER_DIR.mkdir(parents=True, exist_ok=True)
    
    # Verify MAME is available
    try:
        result = subprocess.run([MAME_BIN, "-version"], capture_output=True, text=True, timeout=5)
        if result.returncode != 0:
            print(f"ERROR: MAME not working properly")
            return 1
        print(f"MAME found: {result.stdout.split()[0]}")
    except FileNotFoundError:
        print(f"ERROR: MAME not found in PATH. Install with: brew install mame")
        return 1
    
    print(f"VGM Source: {VGM_SOURCE_DIR}")
    print(f"Golden Master Directory: {GOLDEN_MASTER_DIR}")
    print()
    
    # Find all VGM files
    vgm_files = sorted(VGM_SOURCE_DIR.glob("*.vgm"))
    
    if not vgm_files:
        print("ERROR: No VGM files found in source directory")
        return 1
    
    print(f"Found {len(vgm_files)} VGM files to render")
    print()
    
    # Render all VGM files to WAV
    results = []
    passed = 0
    failed = 0
    
    for vgm_file in vgm_files:
        wav_name = vgm_file.stem + ".wav"
        wav_path = GOLDEN_MASTER_DIR / wav_name
        
        result = render_vgm_to_wav(vgm_file, wav_path)
        results.append(result)
        
        if result["status"] == "PASS":
            passed += 1
        else:
            failed += 1
    
    # Generate summary
    print()
    print("=" * 70)
    print("GOLDEN MASTER GENERATION SUMMARY")
    print("=" * 70)
    print()
    
    print(f"Total VGM files: {len(vgm_files)}")
    print(f"Successfully rendered: {passed}/{len(vgm_files)}")
    print(f"Failed: {failed}/{len(vgm_files)}")
    print()
    
    # Count by chip
    chip_stats = {}
    for result in results:
        vgm_name = Path(result["vgm"]).name
        chip = VGM_TO_CHIP.get(vgm_name, "Unknown")
        
        if chip not in chip_stats:
            chip_stats[chip] = {"total": 0, "passed": 0}
        chip_stats[chip]["total"] += 1
        if result["status"] == "PASS":
            chip_stats[chip]["passed"] += 1
    
    print("Results by Chip:")
    for chip in sorted(chip_stats.keys()):
        stats = chip_stats[chip]
        chip_info = TIER2_CHIPS.get(chip, {"name": chip})
        status = "✓" if stats["passed"] == stats["total"] else "✗"
        print(f"  {status} {chip_info['name']:40} {stats['passed']}/{stats['total']}")
    
    print()
    
    # Save results
    results_file = GOLDEN_MASTER_DIR / "generation_results.json"
    with open(results_file, 'w') as f:
        json.dump({
            "timestamp": datetime.now().isoformat(),
            "source_dir": str(VGM_SOURCE_DIR),
            "output_dir": str(GOLDEN_MASTER_DIR),
            "mame_version": "vgmplay",
            "total_files": len(vgm_files),
            "passed": passed,
            "failed": failed,
            "results": results
        }, f, indent=2)
    
    print(f"Results saved to: {results_file}")
    print()
    
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
