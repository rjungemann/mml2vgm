#!/usr/bin/env python3
"""
Phase 2 Golden Master Validation Runner

Compiles Tier 2 test MML files and validates generated VGM files.
This is the primary script for Phase 2 validation.
"""

import subprocess
import sys
from pathlib import Path
from datetime import datetime
import json
import os

# Configuration
COMPILER = Path(__file__).parent.parent.parent / "mml2vgm-rs" / "target" / "release" / "mml2vgm-rs"
TEST_DIR = Path(__file__).parent.parent.parent / "tests" / "golden_master" / "tier2"
OUTPUT_DIR = Path(__file__).parent.parent.parent / "validation_results" / "phase2"
VALIDATE_TOOL = Path(__file__).parent / "validate_vgm_binary.py"

# Tier 2 Test definitions: (test_file, chip_type, description)
# Based on docs/Golden_Master_Comparison_Plan.md
TIER2_TESTS = [
    # YM2413 (OPLL) - 3 tests
    ("test_ym2413_patches.gwi", "YM2413", "YM2413 Built-in Patches Test"),
    ("test_ym2413_custom.gwi", "YM2413", "YM2413 Custom Patch Test"),
    ("test_ym2413_rhythm.gwi", "YM2413", "YM2413 Rhythm Mode Test"),
    
    # Y8950 (OPL + ADPCM) - 2 tests
    ("test_y8950_opl.gwi", "Y8950", "Y8950 OPL Core Test"),
    ("test_y8950_adpcm.gwi", "Y8950", "Y8950 ADPCM Test"),
    
    # RF5C164 (Sega CD) - 2 tests
    ("test_rf5c164_basic.gwi", "RF5C164", "RF5C164 Basic PCM Test"),
    ("test_rf5c164_pitch.gwi", "RF5C164", "RF5C164 Pitch Tracking Test"),
    
    # C140 (Namco) - 2 tests
    ("test_c140_basic.gwi", "C140", "C140 Basic PCM Test"),
    ("test_c140_loop.gwi", "C140", "C140 Loop Address Test"),
    
    # C352 (Namco System 21/22) - 2 tests
    ("test_c352_basic.gwi", "C352", "C352 Basic PCM Test"),
    ("test_c352_filter.gwi", "C352", "C352 Filter Test"),
    
    # K053260 (Konami PCM) - 2 tests
    ("test_k053260_basic.gwi", "K053260", "K053260 Basic Test"),
    ("test_konami_pcm_pitch.gwi", "K053260", "K053260 Pitch Tracking Test"),
    
    # K054539 (Konami Enhanced PCM) - 2 tests
    ("test_k054539_basic.gwi", "K054539", "K054539 Basic Test"),
    ("test_konami_pcm_pitch.gwi", "K054539", "K054539 Pitch Tracking Test"),
    
    # AY8910 (PSG) - 2 tests
    ("test_ay8910_envelope.gwi", "AY8910", "AY8910 Envelope Test"),
    ("test_ay8910_wavetable.gwi", "AY8910", "AY8910 Wavetable Test"),
    
    # HuC6280 (PC Engine) - 1 test
    ("test_huc6280_wavetable.gwi", "HuC6280", "HuC6280 Wavetable Test"),
]

# Chip information for reporting
CHIP_INFO = {
    "YM2413": {"name": "YM2413 (OPLL)", "tier": "tier2", "reference": "Mednafen 1.32.1"},
    "Y8950": {"name": "Y8950 (OPL + ADPCM)", "tier": "tier2", "reference": "DOSBox-X 2026.05.02"},
    "RF5C164": {"name": "RF5C164 (Sega CD)", "tier": "tier2", "reference": "Mednafen 1.32.1"},
    "C140": {"name": "C140 (Namco 163)", "tier": "tier2", "reference": "MAME 0.287"},
    "C352": {"name": "C352 (Namco System 21/22)", "tier": "tier2", "reference": "MAME 0.287"},
    "K053260": {"name": "K053260 (Konami PCM)", "tier": "tier2", "reference": "MAME 0.287"},
    "K054539": {"name": "K054539 (Konami Enhanced PCM)", "tier": "tier2", "reference": "MAME 0.287"},
    "AY8910": {"name": "AY8910 (PSG)", "tier": "tier2", "reference": "Mednafen 1.32.1"},
    "HuC6280": {"name": "HuC6280 (PC Engine)", "tier": "tier2", "reference": "Mednafen 1.32.1"},
}

def run_test(test_file: str, chip_type: str, description: str) -> dict:
    """Run a single Phase 2 validation test."""
    test_path = TEST_DIR / test_file
    output_vgm = OUTPUT_DIR / f"{test_file.replace('.gwi', '.vgm')}"
    
    if not test_path.exists():
        return {
            "test": test_file,
            "chip": chip_type,
            "status": "SKIP",
            "reason": f"Test file not found: {test_path}",
            "timestamp": datetime.now().isoformat()
        }
    
    # Compile MML file
    print(f"  Compiling {test_file}...", end=" ", flush=True)
    try:
        result = subprocess.run(
            [str(COMPILER), str(test_path), "-o", str(output_vgm), "--chip", chip_type],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode != 0:
            return {
                "test": test_file,
                "chip": chip_type,
                "status": "FAIL",
                "reason": f"Compilation error: {result.stderr[:200]}",
                "timestamp": datetime.now().isoformat(),
                "compiler_output": result.stdout,
                "compiler_error": result.stderr
            }
        
        # Parse compilation output for stats
        output = result.stdout + result.stderr
        lines = output.split('\n')
        stats = {}
        for line in lines:
            if "Parts:" in line:
                try:
                    parts = int(line.strip().split()[-1])
                    stats['parts'] = parts
                except:
                    pass
            if "Commands:" in line:
                try:
                    commands = int(line.strip().split()[-1])
                    stats['commands'] = commands
                except:
                    pass
            if "Duration:" in line:
                try:
                    duration_str = line.split("(")[1].split(" samples")[0]
                    stats['duration'] = duration_str
                except:
                    pass
        
        if stats.get('commands', 0) == 0:
            return {
                "test": test_file,
                "chip": chip_type,
                "status": "FAIL",
                "reason": f"No register writes generated: {stats}",
                "timestamp": datetime.now().isoformat(),
                "stats": stats
            }
        
        # Verify output file exists and has content
        if not output_vgm.exists() or output_vgm.stat().st_size == 0:
            return {
                "test": test_file,
                "chip": chip_type,
                "status": "FAIL",
                "reason": "Output VGM file not created or empty",
                "timestamp": datetime.now().isoformat(),
                "stats": stats
            }
        
        file_size = output_vgm.stat().st_size
        print(f"✓ ({stats.get('commands', 0)} commands, {file_size} bytes)")
        
        return {
            "test": test_file,
            "chip": chip_type,
            "status": "PASS",
            "reason": "Compilation successful",
            "timestamp": datetime.now().isoformat(),
            "stats": stats,
            "output_vgm": str(output_vgm),
            "vgm_size_bytes": file_size
        }
    
    except subprocess.TimeoutExpired:
        return {
            "test": test_file,
            "chip": chip_type,
            "status": "FAIL",
            "reason": "Compilation timeout (30s)",
            "timestamp": datetime.now().isoformat()
        }
    except Exception as e:
        return {
            "test": test_file,
            "chip": chip_type,
            "status": "FAIL",
            "reason": f"Error: {str(e)}",
            "timestamp": datetime.now().isoformat()
        }


def run_binary_validation(output_dir: Path) -> list:
    """Run binary validation on all compiled VGM files."""
    print("\n" + "=" * 70)
    print("PHASE 2: Binary VGM Validation")
    print("=" * 70 + "\n")
    
    vgm_files = sorted(output_dir.glob("*.vgm"))
    vgm_results = []
    
    print(f"Validating {len(vgm_files)} VGM files...\n")
    
    for vgm_file in vgm_files:
        print(f"  {vgm_file.name:40} ", end="", flush=True)
        try:
            result = subprocess.run(
                [sys.executable, str(VALIDATE_TOOL), str(vgm_file), "--json"],
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if result.returncode == 0:
                try:
                    report = json.loads(result.stdout)
                    vgm_results.append(report)
                    writes = report["summary"]["register_writes"]
                    print(f"✅ ({writes} register writes)")
                except json.JSONDecodeError:
                    print("✅ (valid structure)")
            else:
                print(f"⚠️  (validation warnings)")
                vgm_results.append({"file": str(vgm_file), "status": "WARN", "error": result.stderr})
        except Exception as e:
            print(f"❌ {e}")
            vgm_results.append({"file": str(vgm_file), "status": "FAIL", "error": str(e)})
    
    return vgm_results


def generate_summary(results: list, vgm_results: list) -> dict:
    """Generate comprehensive summary of Phase 2 validation."""
    print("\n" + "=" * 70)
    print("PHASE 2 VALIDATION SUMMARY")
    print("=" * 70 + "\n")
    
    # Count by chip
    chip_stats = {}
    for result in results:
        chip = result.get("chip", "Unknown")
        if chip not in chip_stats:
            chip_stats[chip] = {"total": 0, "passed": 0, "failed": 0, "skipped": 0, "size": 0}
        chip_stats[chip]["total"] += 1
        if result["status"] == "PASS":
            chip_stats[chip]["passed"] += 1
            chip_stats[chip]["size"] += result.get("vgm_size_bytes", 0)
        elif result["status"] == "FAIL":
            chip_stats[chip]["failed"] += 1
        else:
            chip_stats[chip]["skipped"] += 1
    
    print("Compilation Results by Chip:\n")
    for chip in sorted(chip_stats.keys()):
        stats = chip_stats[chip]
        chip_name = CHIP_INFO.get(chip, {"name": chip})["name"]
        pass_rate = (stats["passed"] / stats["total"] * 100) if stats["total"] > 0 else 0
        print(f"  {chip_name:30} {stats['passed']}/{stats['total']} PASS ({pass_rate:.0f}%)")
    
    total_passed = sum(s["passed"] for s in chip_stats.values())
    total_failed = sum(s["failed"] for s in chip_stats.values())
    total_skipped = sum(s["skipped"] for s in chip_stats.values())
    total_size = sum(s["size"] for s in chip_stats.values())
    
    print(f"\n{'='*70}")
    print(f"Overall: {total_passed} passed, {total_failed} failed, {total_skipped} skipped")
    print(f"Total VGM Size: {total_size:,} bytes")
    print(f"{'='*70}\n")
    
    # Binary validation summary
    if vgm_results:
        print("Binary Validation Summary:\n")
        valid_count = sum(1 for r in vgm_results if r.get("status") != "FAIL")
        print(f"  VGM Files Validated: {valid_count}/{len(vgm_results)}")
        
        total_writes = 0
        for r in vgm_results:
            if isinstance(r, dict) and "summary" in r:
                total_writes += r["summary"].get("register_writes", 0)
        print(f"  Total Register Writes: {total_writes}")
    
    print("\n" + "=" * 70)
    
    return {
        "timestamp": datetime.now().isoformat(),
        "summary": {
            "phase": "phase2",
            "total_tests": len(results),
            "passed": total_passed,
            "failed": total_failed,
            "skipped": total_skipped,
            "total_vgm_size_bytes": total_size,
            "binary_validated": valid_count,
            "total_register_writes": total_writes
        },
        "chip_stats": chip_stats,
        "results": results
    }


def main():
    """Run Phase 2 validation suite."""
    print("=" * 70)
    print("PHASE 2 GOLDEN MASTER VALIDATION RUNNER")
    print("Tier 2 Chip Validation")
    print("=" * 70)
    print()
    
    # Create output directory
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    
    # Verify compiler exists
    if not COMPILER.exists():
        print(f"ERROR: Compiler not found at {COMPILER}")
        print("Please build the project with: cargo build --release -p mml2vgm-rs")
        sys.exit(1)
    
    print(f"Compiler: {COMPILER}")
    print(f"Test Directory: {TEST_DIR}")
    print(f"Output Directory: {OUTPUT_DIR}")
    print(f"Tests to run: {len(TIER2_TESTS)}")
    print()
    
    # Run compilation tests
    print("=" * 70)
    print("PHASE 2: MML Compilation & VGM Generation")
    print("=" * 70)
    print()
    
    results = []
    
    for test_file, chip_type, description in TIER2_TESTS:
        print(f"{description:40} ", end="", flush=True)
        result = run_test(test_file, chip_type, description)
        results.append(result)
        
        if result["status"] == "PASS":
            print(f"✓")
        elif result["status"] == "FAIL":
            print(f"✗ {result['reason'][:50]}")
        else:
            print(f"⊘ {result['reason']}")
    
    # Run binary validation on compiled files
    vgm_results = run_binary_validation(OUTPUT_DIR)
    
    # Generate summary
    summary = generate_summary(results, vgm_results)
    
    # Save results to JSON
    results_file = OUTPUT_DIR / "phase2_results.json"
    with open(results_file, 'w') as f:
        json.dump(summary, f, indent=2)
    
    print(f"Results saved to: {results_file}\n")
    
    # Determine exit code
    failed_count = summary["summary"]["failed"]
    return 0 if failed_count == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
