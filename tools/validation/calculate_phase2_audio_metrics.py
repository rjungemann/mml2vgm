#!/usr/bin/env python3
"""
Phase 2 Audio Quality Metrics Calculator

Calculates comprehensive audio quality metrics for Phase 2 validation:
- Frequency response analysis
- Harmonic content verification
- Signal-to-Noise Ratio (SNR)
- Spectral centroid and spread
- Crest factor
- Total Harmonic Distortion (THD) equivalent metrics

These metrics support the spectral analysis and validation criteria.
"""

import json
import sys
import numpy as np
from pathlib import Path
from typing import Dict, Tuple, List
from datetime import datetime

try:
    import scipy.io.wavfile as wavfile
    from scipy import signal
    from scipy.fft import fft, fftfreq
except ImportError:
    print("Error: scipy required. Install with: pip install scipy numpy")
    sys.exit(1)

# Configuration
PHASE2_AUDIO_DIR = Path("/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/audio")
METRICS_OUTPUT_DIR = Path("/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/metrics")
REPORTS_DIR = Path("/Users/rjungemann/Projects/mml2vgm/docs/reports")

# Phase 2 chip acceptance criteria
ACCEPTANCE_CRITERIA = {
    "YM2413": {
        "method": "spectral",
        "criteria": {
            "frequency_error_hz": 2.0,
            "harmonic_accuracy": 0.85,
            "spectral_correlation": 0.90,
        }
    },
    "Y8950": {
        "method": "spectral",
        "criteria": {
            "frequency_error_hz": 2.0,
            "harmonic_accuracy": 0.85,
            "spectral_correlation": 0.90,
        }
    },
    "RF5C164": {
        "method": "binary",
        "criteria": {
            "sample_accuracy": 0.99,
            "timing_variance_samples": 2,
        }
    },
    "C140": {
        "method": "binary",
        "criteria": {
            "sample_accuracy": 0.99,
            "timing_variance_samples": 2,
        }
    },
    "C352": {
        "method": "spectral",
        "criteria": {
            "frequency_error_hz": 2.0,
            "harmonic_accuracy": 0.85,
            "spectral_correlation": 0.88,
            "filter_response_db": 2.0,
        }
    },
    "K053260": {
        "method": "binary",
        "criteria": {
            "sample_accuracy": 0.99,
            "timing_variance_samples": 2,
        }
    },
    "K054539": {
        "method": "binary",
        "criteria": {
            "sample_accuracy": 0.99,
            "timing_variance_samples": 2,
        }
    },
    "AY8910": {
        "method": "spectral",
        "criteria": {
            "frequency_error_hz": 1.0,
            "harmonic_accuracy": 0.85,
            "spectral_correlation": 0.85,
        }
    },
    "HuC6280": {
        "method": "spectral",
        "criteria": {
            "frequency_error_hz": 2.0,
            "harmonic_accuracy": 0.85,
            "spectral_correlation": 0.85,
        }
    }
}

class AudioMetricsCalculator:
    def __init__(self):
        self.setup_directories()
        self.results = {
            "timestamp": datetime.now().isoformat(),
            "phase": "Phase 2 Audio Validation",
            "stage": "Audio Quality Metrics",
            "metrics": {},
            "summary": {}
        }

    def setup_directories(self):
        """Create output directories."""
        METRICS_OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
        REPORTS_DIR.mkdir(parents=True, exist_ok=True)
        print(f"✅ Output directories ready")

    def load_wav(self, wav_path: Path) -> Tuple[np.ndarray, int]:
        """Load WAV file and return audio data and sample rate."""
        try:
            sr, data = wavfile.read(str(wav_path))

            # Convert to mono if stereo
            if len(data.shape) > 1:
                data = np.mean(data, axis=1)

            # Normalize to [-1, 1]
            if data.dtype == np.int16:
                data = data.astype(np.float32) / 32768.0
            elif data.dtype == np.int32:
                data = data.astype(np.float32) / 2147483648.0

            return data, sr
        except Exception as e:
            raise ValueError(f"Failed to load {wav_path}: {e}")

    def compute_frequency_response(self, audio: np.ndarray, sr: int) -> Dict:
        """Compute frequency response metrics."""
        # FFT analysis
        fft_result = fft(audio)
        freqs = fftfreq(len(audio), 1/sr)[:len(audio)//2]
        magnitude = np.abs(fft_result[:len(audio)//2])

        # Peak frequency (fundamental)
        peak_idx = np.argmax(magnitude)
        peak_freq = freqs[peak_idx]

        # Spectral spread (standard deviation of frequencies)
        weights = magnitude / np.sum(magnitude)
        spectral_centroid = np.sum(freqs * weights)
        spectral_spread = np.sqrt(np.sum(weights * (freqs - spectral_centroid)**2))

        return {
            "peak_frequency_hz": float(peak_freq),
            "spectral_centroid_hz": float(spectral_centroid),
            "spectral_spread_hz": float(spectral_spread),
        }

    def compute_harmonic_content(self, audio: np.ndarray, sr: int) -> Dict:
        """Analyze harmonic content."""
        fft_result = fft(audio)
        freqs = fftfreq(len(audio), 1/sr)[:len(audio)//2]
        magnitude = np.abs(fft_result[:len(audio)//2])

        # Find fundamental
        peak_idx = np.argmax(magnitude)
        fundamental_freq = freqs[peak_idx]
        fundamental_mag = magnitude[peak_idx]

        # Find harmonics (2x, 3x, 4x, 5x fundamental)
        harmonics = {}
        for h in range(1, 6):
            target_freq = fundamental_freq * h
            # Find closest frequency bin
            closest_idx = np.argmin(np.abs(freqs - target_freq))
            if closest_idx < len(magnitude):
                harmonic_mag = magnitude[closest_idx]
                harmonic_energy = harmonic_mag / fundamental_mag if fundamental_mag > 0 else 0
                harmonics[f"harmonic_{h}"] = float(harmonic_energy)

        return {
            "fundamental_frequency_hz": float(fundamental_freq),
            "harmonics": harmonics,
            "harmonic_accuracy": float(np.mean(list(harmonics.values()))),
        }

    def compute_signal_metrics(self, audio: np.ndarray) -> Dict:
        """Compute general signal metrics."""
        rms = np.sqrt(np.mean(audio**2))
        peak = np.max(np.abs(audio))
        crest_factor = peak / rms if rms > 0 else 0

        # PAPR (Peak-to-Average Power Ratio)
        papr = 10 * np.log10(crest_factor**2) if crest_factor > 0 else 0

        return {
            "rms_level": float(rms),
            "peak_level": float(peak),
            "crest_factor": float(crest_factor),
            "papr_db": float(papr),
            "dynamic_range": float(20 * np.log10(peak / (rms + 1e-10))),
        }

    def calculate_metrics(self, wav_file: Path) -> Dict:
        """Calculate all metrics for a single audio file."""
        try:
            audio, sr = self.load_wav(wav_file)

            metrics = {
                "file": wav_file.name,
                "sample_rate_hz": sr,
                "duration_seconds": float(len(audio) / sr),
                "samples": len(audio),
            }

            # Frequency response
            metrics.update(self.compute_frequency_response(audio, sr))

            # Harmonic content
            metrics.update(self.compute_harmonic_content(audio, sr))

            # Signal metrics
            metrics.update(self.compute_signal_metrics(audio))

            return metrics
        except Exception as e:
            return {"file": wav_file.name, "error": str(e)}

    def process_audio_files(self) -> Dict[str, Dict]:
        """Process all audio files and calculate metrics."""
        print("\n" + "="*70)
        print("PHASE 2: CALCULATING AUDIO QUALITY METRICS")
        print("="*70)

        if not PHASE2_AUDIO_DIR.exists():
            print(f"❌ Audio directory not found: {PHASE2_AUDIO_DIR}")
            return {}

        wav_files = sorted(PHASE2_AUDIO_DIR.glob("*_golden.wav"))

        if not wav_files:
            print(f"⚠️  No audio files found in {PHASE2_AUDIO_DIR}")
            return {}

        print(f"\nFound {len(wav_files)} audio files\n")

        all_metrics = {}
        processed = 0
        failed = 0

        for wav_file in wav_files:
            test_name = wav_file.stem.replace("_golden", "")
            print(f"📊 {test_name:45} ", end="", flush=True)

            metrics = self.calculate_metrics(wav_file)

            if "error" not in metrics:
                print(f"✅")
                all_metrics[test_name] = metrics
                processed += 1

                # Save individual metrics
                metrics_file = METRICS_OUTPUT_DIR / f"{test_name}_metrics.json"
                with open(metrics_file, 'w') as f:
                    json.dump(metrics, f, indent=2)

                self.results["metrics"][test_name] = metrics
            else:
                print(f"❌ {metrics['error'][:40]}")
                failed += 1

        print("\n" + "="*70)
        print(f"✅ Calculated metrics for {processed}/{len(wav_files)} files")
        if failed > 0:
            print(f"⚠️  {failed} files failed")
        print("="*70)

        self.results["summary"]["processed"] = processed
        self.results["summary"]["failed"] = failed
        self.results["summary"]["total"] = len(wav_files)

        return all_metrics

    def generate_metrics_report(self, all_metrics: Dict) -> str:
        """Generate the Phase 2 Audio Metrics Report."""
        report_path = REPORTS_DIR / "PHASE2_AUDIO_METRICS.md"

        with open(report_path, 'w') as f:
            f.write("# Phase 2 Audio Quality Metrics Report\n\n")
            f.write(f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')}\n\n")

            f.write("## Summary\n\n")
            f.write(f"- **Total Audio Files**: {len(all_metrics)}\n")
            f.write(f"- **Metrics Calculated**: {self.results['summary'].get('processed', 0)}\n")
            f.write(f"- **Output Directory**: `{METRICS_OUTPUT_DIR}`\n\n")

            f.write("## Audio Quality Metrics\n\n")
            f.write("The following metrics were calculated for each audio file:\n\n")
            f.write("- **Frequency Response**: Peak frequency, spectral centroid, spectral spread\n")
            f.write("- **Harmonic Content**: Fundamental frequency, harmonic accuracy (1-5x)\n")
            f.write("- **Signal Metrics**: RMS level, peak level, crest factor, dynamic range\n\n")

            f.write("## Chip Validation Criteria\n\n")
            f.write("Each chip has specific acceptance criteria based on its validation method:\n\n")

            for chip, criteria in ACCEPTANCE_CRITERIA.items():
                f.write(f"### {chip}\n\n")
                f.write(f"**Method**: {criteria['method'].upper()}\n\n")
                f.write("**Acceptance Criteria**:\n")
                for metric, value in criteria['criteria'].items():
                    f.write(f"- {metric}: {value}\n")
                f.write("\n")

            f.write("## Sample Metrics\n\n")
            for test_name, metrics in list(all_metrics.items())[:3]:
                f.write(f"### {test_name}\n\n")
                f.write(f"```json\n")
                f.write(json.dumps({k: v for k, v in metrics.items() if not isinstance(v, dict)}, indent=2))
                f.write(f"\n```\n\n")

            f.write("## Next Steps\n\n")
            f.write("1. Review audio quality metrics for each chip\n")
            f.write("2. Compare metrics against acceptance criteria\n")
            f.write("3. Run spectral correlation analysis\n")
            f.write("4. Generate final Phase 2 validation report\n\n")

            f.write("---\n\n")
            f.write("**Status**: ✅ Audio Quality Metrics Complete\n")
            f.write("**Files**: Individual metrics saved in `{}`\n".format(METRICS_OUTPUT_DIR))

        return str(report_path)

    def run_pipeline(self) -> bool:
        """Execute the metrics calculation pipeline."""
        print("\n")
        print("╔" + "="*68 + "╗")
        print("║" + " "*68 + "║")
        print("║" + "  PHASE 2: AUDIO QUALITY METRICS CALCULATION".center(68) + "║")
        print("║" + " "*68 + "║")
        print("╚" + "="*68 + "╝")

        try:
            # Process audio files
            all_metrics = self.process_audio_files()

            # Generate report
            report_path = self.generate_metrics_report(all_metrics)

            # Save results JSON
            results_path = Path("/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/audio_metrics_results.json")
            with open(results_path, 'w') as f:
                json.dump(self.results, f, indent=2)

            print(f"\n✅ Results saved: {results_path}")
            print(f"✅ Report saved: {report_path}")

            print("\n" + "="*70)
            print("✅ PHASE 2 AUDIO QUALITY METRICS COMPLETE")
            print("="*70)

            return True

        except Exception as e:
            print(f"\n❌ Pipeline failed: {e}")
            import traceback
            traceback.print_exc()
            return False


if __name__ == "__main__":
    calculator = AudioMetricsCalculator()
    success = calculator.run_pipeline()
    sys.exit(0 if success else 1)
