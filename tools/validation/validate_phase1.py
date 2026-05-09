#!/usr/bin/env python3
"""
Phase 1 Golden Master Validation Runner

Compiles test MML files and compares generated VGM against golden masters.
"""

import subprocess
import sys
from pathlib import Path
from datetime import datetime
import json

# Configuration
COMPILER = Path(__file__).parent.parent.parent / "mml2vgm-rs" / "target" / "release" / "mml2vgm-rs"
TEST_DIR = Path(__file__).parent.parent.parent / "tests" / "golden_master" / "tier1"
OUTPUT_DIR = Path(__file__).parent.parent.parent / "validation_results"
COMPARE_TOOL = Path(__file__).parent / "vgm_compare.py"

# Test definitions: (test_file, chip_type, description)
TIER1_TESTS = [
    ("test_ym2151_envelope.gwi", "YM2151", "YM2151 Envelope Test"),
    ("test_nes_pulse.gwi", "NES", "NES Pulse Channel Test"),
    ("test_nes_triangle.gwi", "NES", "NES Triangle Channel Test"),
    ("test_nes_noise.gwi", "NES", "NES Noise Channel Test"),
    ("test_ym2203_fm.gwi", "YM2203", "YM2203 FM Test"),
    ("test_ym2203_ssg.gwi", "YM2203", "YM2203 SSG Test"),
    ("test_ym2203_mixed.gwi", "YM2203", "YM2203 Mixed FM+SSG Test"),
    ("test_ym2608_fm.gwi", "YM2608", "YM2608 FM Test"),
    ("test_ym2608_ssg.gwi", "YM2608", "YM2608 SSG Test"),
    ("test_opl2_basic.gwi", "YM3812", "OPL2 Basic Test"),
    ("test_opl_envelope.gwi", "YM3812", "OPL Envelope Test"),
    ("test_opl3_4op.gwi", "YMF262", "OPL3 4-Operator Test"),
]

def run_test(test_file: str, chip_type: str, description: str) -> dict:
    """Run a single validation test."""
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
                "reason": f"Compilation error: {result.stderr}",
                "timestamp": datetime.now().isoformat()
            }
        
        # Parse compilation output for stats (output goes to stdout)
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
        
        print(f"✓ ({stats.get('commands', 0)} commands)")
        
        return {
            "test": test_file,
            "chip": chip_type,
            "status": "PASS",
            "reason": "Compilation successful",
            "timestamp": datetime.now().isoformat(),
            "stats": stats,
            "output_vgm": str(output_vgm)
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

def main():
    """Run Phase 1 validation suite."""
    print("=" * 70)
    print("PHASE 1 GOLDEN MASTER VALIDATION RUNNER")
    print("=" * 70)
    print()
    
    # Create output directory
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    
    # Verify compiler exists
    if not COMPILER.exists():
        print(f"ERROR: Compiler not found at {COMPILER}")
        print("Please build the project with: cargo build --release -p mml2vgm-rs")
        sys.exit(1)
    
    results = []
    passed = 0
    failed = 0
    skipped = 0
    
    print("Running Tier 1 tests...")
    print()
    
    for test_file, chip_type, description in TIER1_TESTS:
        print(f"{description:40} ", end="", flush=True)
        result = run_test(test_file, chip_type, description)
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
    
    # Save results to JSON
    results_file = OUTPUT_DIR / "phase1_results.json"
    with open(results_file, 'w') as f:
        json.dump({
            "timestamp": datetime.now().isoformat(),
            "summary": {
                "passed": passed,
                "failed": failed,
                "skipped": skipped,
                "total": len(TIER1_TESTS)
            },
            "results": results
        }, f, indent=2)
    
    print(f"\nResults saved to: {results_file}")
    
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
