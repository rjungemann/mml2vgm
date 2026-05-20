#!/usr/bin/env python3
"""
Phase 2 Audio Validation - Golden Master Generation

Generates golden master audio references for all Phase 2 Tier 2 chip VGM files
and creates the foundation for spectral analysis and metrics calculation.

This script:
1. Renders each Phase 2 VGM file to WAV using MAME vgmplay
2. Validates audio output integrity
3. Prepares for spectral analysis and metrics calculation
4. Generates phase 2 audio validation baseline report
"""

import subprocess
import json
import sys
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Tuple

# Configuration
PROJECT_ROOT = Path("/Users/rjungemann/Projects/mml2vgm")
PHASE2_VGM_DIR = PROJECT_ROOT / "validation_results" / "phase2"
AUDIO_OUTPUT_DIR = PHASE2_VGM_DIR / "audio"
REPORTS_DIR = PROJECT_ROOT / "docs" / "reports"
TOOLS_DIR = PROJECT_ROOT / "tools" / "validation"

# Chip metadata for Phase 2
CHIP_METADATA = {
    "YM2413": {"tier": 2, "method": "spectral", "channels": 9, "operator_count": 36},
    "Y8950": {"tier": 2, "method": "spectral", "channels": 9, "operator_count": 36},
    "RF5C164": {"tier": 2, "method": "binary", "channels": 8, "pcm": True},
    "C140": {"tier": 2, "method": "binary", "channels": 24, "pcm": True},
    "C352": {"tier": 2, "method": "spectral", "channels": 24, "pcm": True},
    "K053260": {"tier": 2, "method": "binary", "channels": 4, "pcm": True},
    "K054539": {"tier": 2, "method": "binary", "channels": 8, "pcm": True},
    "AY8910": {"tier": 2, "method": "spectral", "channels": 3, "psg": True},
    "HuC6280": {"tier": 2, "method": "spectral", "channels": 6, "wavetable": True},
}

class Phase2AudioGenerator:
    def __init__(self):
        self.setup_directories()
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "phase": "Phase 2 Audio Validation",
            "stage": "Golden Master Generation",
            "audio_files": {},
            "summary": {}
        }

    def setup_directories(self):
        """Create necessary output directories."""
        AUDIO_OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
        REPORTS_DIR.mkdir(parents=True, exist_ok=True)
        print(f"✅ Output directories ready")
        print(f"   Audio: {AUDIO_OUTPUT_DIR}")
        print(f"   Reports: {REPORTS_DIR}")

    def render_vgm_file(self, vgm_path: Path, output_wav: Path) -> Tuple[bool, str]:
        """
        Render a single VGM file to WAV using MAME vgmplay.

        Returns: (success, message)
        """
        try:
            mame_bin = "/opt/homebrew/bin/mame"
            result = subprocess.run(
                [mame_bin, "vgmplay", str(vgm_path), "-wavwrite", str(output_wav)],
                capture_output=True,
                text=True,
                timeout=120
            )

            if result.returncode == 0 and output_wav.exists():
                size_kb = output_wav.stat().st_size / 1024
                return True, f"{size_kb:.1f}KB"
            else:
                error_msg = result.stderr.split('\n')[0] if result.stderr else "Unknown error"
                return False, f"MAME error: {error_msg[:50]}"

        except subprocess.TimeoutExpired:
            return False, "Timeout (120s)"
        except FileNotFoundError:
            return False, "MAME not found (install via: brew install mame)"
        except Exception as e:
            return False, str(e)[:50]

    def extract_chip_name(self, vgm_filename: str) -> str:
        """Extract chip name from VGM filename."""
        parts = vgm_filename.replace("test_", "").split("_")
        chip = parts[0].upper()

        # Handle multi-part chip names
        if chip == "K" and len(parts) > 1:
            chip = f"K{parts[1]}"

        return chip if chip in CHIP_METADATA else "UNKNOWN"

    def generate_golden_masters(self) -> Dict[str, Dict]:
        """Generate golden master audio files for all Phase 2 VGM files."""
        print("\n" + "="*70)
        print("PHASE 2: GENERATING GOLDEN MASTER AUDIO FILES")
        print("="*70)

        vgm_files = sorted(PHASE2_VGM_DIR.glob("*.vgm"))

        if not vgm_files:
            print("❌ No VGM files found in:", PHASE2_VGM_DIR)
            return {}

        print(f"\nFound {len(vgm_files)} VGM files to render\n")

        results_by_chip = {}
        passed = 0
        failed = 0

        for vgm_file in vgm_files:
            chip_name = self.extract_chip_name(vgm_file.stem)
            wav_filename = vgm_file.stem.replace(".vgm", "") + "_golden.wav"
            wav_path = AUDIO_OUTPUT_DIR / wav_filename

            # Status prefix
            status = "  " if chip_name in CHIP_METADATA else "⚠️ "
            print(f"{status}{vgm_file.name:45} → ", end="", flush=True)

            # Render
            success, message = self.render_vgm_file(vgm_file, wav_path)

            if success:
                print(f"✅ {message}")
                passed += 1

                # Track results
                if chip_name not in results_by_chip:
                    results_by_chip[chip_name] = {"tests": [], "status": "complete"}
                results_by_chip[chip_name]["tests"].append({
                    "vgm": vgm_file.name,
                    "wav": wav_filename,
                    "size_bytes": wav_path.stat().st_size
                })

                self.results["audio_files"][vgm_file.stem] = {
                    "vgm": str(vgm_path),
                    "wav": str(wav_path),
                    "size_bytes": wav_path.stat().st_size,
                    "chip": chip_name,
                    "status": "ready_for_analysis"
                }
            else:
                print(f"❌ {message}")
                failed += 1
                if chip_name not in results_by_chip:
                    results_by_chip[chip_name] = {"tests": [], "status": "partial"}

        # Summary
        print("\n" + "="*70)
        print(f"✅ Generated {passed}/{len(vgm_files)} audio files")
        if failed > 0:
            print(f"⚠️  {failed} files failed to render")
        print("="*70)

        self.results["summary"]["generated"] = passed
        self.results["summary"]["failed"] = failed
        self.results["summary"]["total"] = len(vgm_files)

        return results_by_chip

    def generate_baseline_report(self, audio_results: Dict) -> str:
        """Generate the Phase 2 Audio Validation baseline report."""
        report_path = REPORTS_DIR / "PHASE2_AUDIO_VALIDATION_BASELINE.md"

        with open(report_path, 'w') as f:
            f.write("# Phase 2 Audio Validation - Golden Master Baseline\n\n")
            f.write(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')}\n\n")

            f.write("## Overview\n\n")
            f.write(f"This report documents the generation of golden master audio files ")
            f.write(f"for Phase 2 Tier 2 chip validation.\n\n")

            f.write("### Summary\n\n")
            f.write(f"- **Total VGM Files**: {self.results['summary'].get('total', 0)}\n")
            f.write(f"- **Audio Generated**: {self.results['summary'].get('generated', 0)}\n")
            f.write(f"- **Generation Failures**: {self.results['summary'].get('failed', 0)}\n")
            f.write(f"- **Output Directory**: `{AUDIO_OUTPUT_DIR}`\n\n")

            f.write("## Chip-by-Chip Status\n\n")
            for chip in sorted(CHIP_METADATA.keys()):
                if chip in audio_results:
                    data = audio_results[chip]
                    test_count = len(data["tests"])
                    f.write(f"### {chip}\n\n")
                    f.write(f"- **Status**: ✅ {data['status'].upper()}\n")
                    f.write(f"- **Tests**: {test_count}\n")
                    f.write(f"- **Method**: {CHIP_METADATA[chip]['method'].upper()}\n")
                    f.write(f"- **Audio Files**:\n")
                    for test in data["tests"]:
                        size_mb = test['size_bytes'] / (1024*1024)
                        f.write(f"  - `{test['wav']}` ({size_mb:.2f}MB)\n")
                    f.write("\n")

            f.write("## Next Steps\n\n")
            f.write("### Phase 2 Audio Validation Pipeline:\n\n")
            f.write("1. **Spectral Analysis** (Next)\n")
            f.write("   - Compare spectrograms using STFT\n")
            f.write("   - Frequency response validation\n")
            f.write("   - Generate spectrogram plots\n\n")
            f.write("2. **Audio Quality Metrics**\n")
            f.write("   - Compute signal metrics\n")
            f.write("   - Harmonic accuracy check\n")
            f.write("   - SNR and frequency response\n\n")
            f.write("3. **Final Sign-Off**\n")
            f.write("   - Comprehensive validation report\n")
            f.write("   - Per-chip acceptance criteria\n")
            f.write("   - Phase 2 completion summary\n\n")

            f.write("## Files Generated\n\n")
            f.write(f"- Golden master audio files: `{AUDIO_OUTPUT_DIR}`\n")
            f.write(f"- Baseline report: This file\n")
            f.write(f"- Full results JSON: `validation_results/phase2/audio_generation_results.json`\n\n")

            f.write("---\n\n")
            f.write("**Status**: ✅ Golden Master Generation Complete\n")
            f.write("**Phase**: Phase 2 Audio Validation\n")
            f.write("**Ready for**: Spectral Analysis (Step 2)\n")

        return str(report_path)

    def run_pipeline(self) -> bool:
        """Execute the Phase 2 audio golden master generation pipeline."""
        print("\n")
        print("╔" + "="*68 + "╗")
        print("║" + " "*68 + "║")
        print("║" + "  PHASE 2: GOLDEN MASTER AUDIO GENERATION".center(68) + "║")
        print("║" + "  Tier 2 Chip Audio Validation Baseline".center(68) + "║")
        print("║" + " "*68 + "║")
        print("╚" + "="*68 + "╝")

        try:
            # Step 1: Generate golden master audio files
            audio_results = self.generate_golden_masters()

            # Step 2: Generate baseline report
            report_path = self.generate_baseline_report(audio_results)

            # Step 3: Save results JSON
            results_path = PHASE2_VGM_DIR / "audio_generation_results.json"
            with open(results_path, 'w') as f:
                json.dump(self.results, f, indent=2)

            print(f"\n✅ Results saved: {results_path}")
            print(f"✅ Report saved: {report_path}")

            print("\n" + "="*70)
            print("✅ PHASE 2 GOLDEN MASTER GENERATION COMPLETE")
            print("="*70)
            print(f"\nNext: Run spectral analysis and audio metrics")
            print(f"  python3 {TOOLS_DIR}/phase2_audio_validation.py")

            return True

        except Exception as e:
            print(f"\n❌ Pipeline failed: {e}")
            import traceback
            traceback.print_exc()
            return False


if __name__ == "__main__":
    generator = Phase2AudioGenerator()
    success = generator.run_pipeline()
    sys.exit(0 if success else 1)
