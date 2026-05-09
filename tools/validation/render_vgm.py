#!/usr/bin/env python3
"""
VGM-to-WAV Rendering Script

Uses MAME vgmplay to render compiled VGM files to WAV format for spectral analysis.
"""

import subprocess
import sys
from pathlib import Path
import json
from datetime import datetime

# Configuration
VALIDATION_DIR = Path(__file__).parent.parent.parent / "validation_results"
MAME_BIN = "mame"  # Assumes MAME is in PATH

def render_vgm(vgm_path: str, output_wav: str) -> dict:
    """
    Render a VGM file to WAV using MAME vgmplay.

    Args:
        vgm_path: Path to input VGM file
        output_wav: Path to output WAV file

    Returns:
        Result dict with status, duration, size, etc.
    """
    vgm_file = Path(vgm_path)
    wav_file = Path(output_wav)

    if not vgm_file.exists():
        return {
            "vgm": str(vgm_path),
            "status": "SKIP",
            "reason": f"VGM file not found: {vgm_path}",
            "timestamp": datetime.now().isoformat()
        }

    print(f"  Rendering {vgm_file.name}...", end=" ", flush=True)

    try:
        # MAME vgmplay: mame vgmplay <file> -wavwrite <output> [-out auto]
        result = subprocess.run(
            [MAME_BIN, "vgmplay", str(vgm_file), "-wavwrite", str(wav_file)],
            capture_output=True,
            text=True,
            timeout=120
        )

        if result.returncode != 0:
            return {
                "vgm": str(vgm_path),
                "wav": str(output_wav),
                "status": "FAIL",
                "reason": f"MAME rendering failed: {result.stderr}",
                "timestamp": datetime.now().isoformat()
            }

        if not wav_file.exists():
            return {
                "vgm": str(vgm_path),
                "wav": str(output_wav),
                "status": "FAIL",
                "reason": "WAV file was not created by MAME",
                "timestamp": datetime.now().isoformat()
            }

        # Get WAV file info
        wav_size = wav_file.stat().st_size

        # Try to parse duration from MAME output
        duration_str = "unknown"
        for line in (result.stdout + result.stderr).split('\n'):
            if "samples" in line.lower() or "time" in line.lower():
                duration_str = line.strip()
                break

        print(f"✓ ({wav_size/1024:.1f} KB)")

        return {
            "vgm": str(vgm_path),
            "wav": str(output_wav),
            "status": "PASS",
            "reason": "Rendering successful",
            "wav_size": wav_size,
            "duration_info": duration_str,
            "timestamp": datetime.now().isoformat()
        }

    except subprocess.TimeoutExpired:
        return {
            "vgm": str(vgm_path),
            "wav": str(output_wav),
            "status": "FAIL",
            "reason": "Rendering timeout (120s)",
            "timestamp": datetime.now().isoformat()
        }
    except FileNotFoundError:
        return {
            "vgm": str(vgm_path),
            "wav": str(output_wav),
            "status": "FAIL",
            "reason": f"MAME not found. Is it installed and in PATH? ({MAME_BIN})",
            "timestamp": datetime.now().isoformat()
        }
    except Exception as e:
        return {
            "vgm": str(vgm_path),
            "wav": str(output_wav),
            "status": "FAIL",
            "reason": f"Error: {str(e)}",
            "timestamp": datetime.now().isoformat()
        }

def main():
    """Render all Phase 1 VGM files to WAV."""
    print("=" * 70)
    print("VGM-TO-WAV RENDERING (MAME vgmplay)")
    print("=" * 70)
    print()

    # Create WAV output directory
    wav_dir = VALIDATION_DIR / "rendered_audio"
    wav_dir.mkdir(parents=True, exist_ok=True)

    # Find all VGM files in validation_results
    vgm_files = sorted(VALIDATION_DIR.glob("test_*.vgm"))

    if not vgm_files:
        print("No VGM files found in validation_results/")
        return 1

    results = []
    passed = 0
    failed = 0
    skipped = 0

    print(f"Found {len(vgm_files)} VGM files to render")
    print()

    for vgm_file in vgm_files:
        wav_name = vgm_file.stem + ".wav"
        wav_output = wav_dir / wav_name

        print(f"{vgm_file.name:40} ", end="", flush=True)
        result = render_vgm(str(vgm_file), str(wav_output))
        results.append(result)

        if result["status"] == "PASS":
            passed += 1
        elif result["status"] == "FAIL":
            failed += 1
            print(f"✗ {result['reason']}")
        else:
            skipped += 1
            print(f"⊘ {result['reason']}")

    print()
    print("=" * 70)
    print(f"Results: {passed} passed, {failed} failed, {skipped} skipped")
    print("=" * 70)

    # Save results
    results_file = VALIDATION_DIR / "render_results.json"
    with open(results_file, 'w') as f:
        json.dump({
            "timestamp": datetime.now().isoformat(),
            "summary": {
                "passed": passed,
                "failed": failed,
                "skipped": skipped,
                "total": len(vgm_files)
            },
            "results": results,
            "wav_directory": str(wav_dir)
        }, f, indent=2)

    print(f"\nResults saved to: {results_file}")
    print(f"WAV files saved to: {wav_dir}")

    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
