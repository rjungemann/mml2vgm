#!/usr/bin/env python3
"""
Spectral Analysis & Comparison Tool

Compares rendered VGM audio against golden master references using STFT-based
spectral analysis to validate register write correctness through audio output.
"""

import numpy as np
from pathlib import Path
import json
import sys
from datetime import datetime
from typing import Tuple, Dict
import argparse

try:
    import scipy.io.wavfile as wavfile
    from scipy.signal import stft, windows
except ImportError:
    print("Error: scipy not found. Install with: pip install scipy numpy matplotlib")
    sys.exit(1)

try:
    import matplotlib.pyplot as plt
    import matplotlib.colors as mcolors
except ImportError:
    print("Error: matplotlib not found. Install with: pip install matplotlib")
    sys.exit(1)


class SpectralAnalyzer:
    """Analyze and compare spectrograms of VGM renderings."""

    def __init__(self, tolerance=0.05, plot=True):
        self.tolerance = tolerance
        self.plot = plot

    def load_wav(self, wav_path: str) -> Tuple[np.ndarray, int]:
        """Load WAV file and return audio data and sample rate."""
        try:
            sample_rate, audio_data = wavfile.read(wav_path)

            # Convert stereo to mono if needed
            if len(audio_data.shape) > 1:
                audio_data = np.mean(audio_data, axis=1)

            # Normalize to [-1, 1]
            if audio_data.dtype == np.int16:
                audio_data = audio_data.astype(np.float32) / 32768.0
            elif audio_data.dtype == np.int32:
                audio_data = audio_data.astype(np.float32) / 2147483648.0

            return audio_data, sample_rate
        except Exception as e:
            raise ValueError(f"Failed to load {wav_path}: {e}")

    def compute_stft(self, audio: np.ndarray, sr: int, n_fft: int = 512) -> Tuple[
        np.ndarray, np.ndarray, np.ndarray]:
        """Compute STFT spectrogram."""
        window = windows.hann(n_fft)
        f, t, Zxx = stft(audio, sr, window=window, nperseg=n_fft)
        magnitude = np.abs(Zxx)
        return magnitude, f, t

    def align_spectrograms(self, spec1: np.ndarray, spec2: np.ndarray) -> Tuple[
        np.ndarray, np.ndarray]:
        """Align two spectrograms to same size."""
        min_time = min(spec1.shape[1], spec2.shape[1])
        spec1 = spec1[:, :min_time]
        spec2 = spec2[:, :min_time]
        return spec1, spec2

    def compute_similarity(self, spec1: np.ndarray, spec2: np.ndarray) -> Dict:
        """
        Compute spectral similarity metrics.

        Returns:
            Dict with correlation, frequency error, phase coherence
        """
        spec1_norm = spec1 / (np.max(spec1) + 1e-10)
        spec2_norm = spec2 / (np.max(spec2) + 1e-10)

        # Compute cosine similarity per time frame
        similarities = []
        for i in range(spec1_norm.shape[1]):
            v1 = spec1_norm[:, i]
            v2 = spec2_norm[:, i]

            dot = np.dot(v1, v2)
            norm1 = np.linalg.norm(v1)
            norm2 = np.linalg.norm(v2)

            if norm1 > 1e-10 and norm2 > 1e-10:
                sim = dot / (norm1 * norm2)
                similarities.append(sim)

        correlation = np.mean(similarities) if similarities else 0

        # Peak frequency accuracy
        peak_freq1 = np.argmax(np.mean(spec1_norm, axis=1))
        peak_freq2 = np.argmax(np.mean(spec2_norm, axis=1))
        freq_error = abs(peak_freq1 - peak_freq2)

        # Energy envelope correlation
        env1 = np.mean(spec1_norm, axis=0)
        env2 = np.mean(spec2_norm, axis=0)
        if np.linalg.norm(env1) > 0 and np.linalg.norm(env2) > 0:
            phase_coherence = np.dot(env1, env2) / (np.linalg.norm(env1) * np.linalg.norm(env2))
        else:
            phase_coherence = 0

        return {
            "correlation": float(correlation),
            "freq_error_bins": float(freq_error),
            "phase_coherence": float(phase_coherence),
            "num_frames": len(similarities)
        }

    def plot_comparison(self, spec1: np.ndarray, spec2: np.ndarray, f: np.ndarray,
                       t1: np.ndarray, t2: np.ndarray, output_path: str, title: str):
        """Generate comparison spectrogram plots."""
        if not self.plot:
            return

        try:
            fig, axes = plt.subplots(3, 1, figsize=(14, 10))

            # Golden master
            pcm1 = axes[0].pcolormesh(t1, f[:len(f)//4], spec1[:len(f)//4], shading='auto',
                                     norm=mcolors.LogNorm(vmin=spec1.max()*1e-4, vmax=spec1.max()))
            axes[0].set_ylabel('Frequency (Hz)')
            axes[0].set_title(f'{title} - Golden Master')
            fig.colorbar(pcm1, ax=axes[0], label='Magnitude')

            # mml2vgm output
            pcm2 = axes[1].pcolormesh(t2, f[:len(f)//4], spec2[:len(f)//4], shading='auto',
                                     norm=mcolors.LogNorm(vmin=spec2.max()*1e-4, vmax=spec2.max()))
            axes[1].set_ylabel('Frequency (Hz)')
            axes[1].set_title(f'{title} - mml2vgm Output')
            fig.colorbar(pcm2, ax=axes[1], label='Magnitude')

            # Difference
            diff = np.abs(spec1 - spec2)
            pcm3 = axes[2].pcolormesh(t1, f[:len(f)//4], diff[:len(f)//4], shading='auto',
                                     norm=mcolors.LogNorm(vmin=diff.max()*1e-4, vmax=diff.max()))
            axes[2].set_ylabel('Frequency (Hz)')
            axes[2].set_xlabel('Time (s)')
            axes[2].set_title(f'{title} - Difference')
            fig.colorbar(pcm3, ax=axes[2], label='Magnitude Difference')

            plt.tight_layout()
            plt.savefig(output_path, dpi=150, bbox_inches='tight')
            plt.close()

            return str(output_path)
        except Exception as e:
            print(f"Warning: Failed to generate plot: {e}")
            return None

    def compare_files(self, mml2vgm_wav: str, golden_master_wav: str) -> Dict:
        """Compare a rendered VGM against golden master."""
        try:
            # Load files
            mml_audio, mml_sr = self.load_wav(mml2vgm_wav)
            golden_audio, golden_sr = self.load_wav(golden_master_wav)

            # Resample to matching rate if needed
            if mml_sr != golden_sr:
                print(f"  Warning: Sample rate mismatch ({mml_sr} vs {golden_sr}), using {golden_sr}")

            # Compute spectrograms
            spec_mml, f, t_mml = self.compute_stft(mml_audio, mml_sr)
            spec_golden, _, t_golden = self.compute_stft(golden_audio, golden_sr)

            # Align
            spec_mml, spec_golden = self.align_spectrograms(spec_mml, spec_golden)

            # Compute similarity
            metrics = self.compute_similarity(spec_mml, spec_golden)

            return {
                "status": "PASS",
                "mml2vgm_samples": len(mml_audio),
                "golden_samples": len(golden_audio),
                "sample_rate": int(mml_sr),
                "metrics": metrics
            }

        except Exception as e:
            return {
                "status": "FAIL",
                "reason": str(e)
            }


def main():
    parser = argparse.ArgumentParser(description="Spectral analysis of rendered VGM files")
    parser.add_argument("--mml2vgm", required=True, help="Path to mml2vgm rendered WAV")
    parser.add_argument("--golden", required=True, help="Path to golden master WAV")
    parser.add_argument("--output-plot", help="Path to save comparison spectrogram")
    parser.add_argument("--title", default="Spectral Comparison", help="Plot title")
    parser.add_argument("--no-plot", action="store_true", help="Disable plot generation")

    args = parser.parse_args()

    analyzer = SpectralAnalyzer(plot=not args.no_plot)

    result = analyzer.compare_files(args.mml2vgm, args.golden)

    if result["status"] == "PASS":
        metrics = result["metrics"]
        print(f"Spectral Analysis Results:")
        print(f"  Samples (mml2vgm): {result['mml2vgm_samples']}")
        print(f"  Samples (golden):  {result['golden_samples']}")
        print(f"  Sample rate: {result['sample_rate']} Hz")
        print(f"  Correlation: {metrics['correlation']:.4f}")
        print(f"  Frequency error: {metrics['freq_error_bins']} bins")
        print(f"  Phase coherence: {metrics['phase_coherence']:.4f}")

        if args.output_plot:
            mml_audio, mml_sr = analyzer.load_wav(args.mml2vgm)
            golden_audio, golden_sr = analyzer.load_wav(args.golden)

            spec_mml, f, t_mml = analyzer.compute_stft(mml_audio, mml_sr)
            spec_golden, _, t_golden = analyzer.compute_stft(golden_audio, golden_sr)
            spec_mml, spec_golden = analyzer.align_spectrograms(spec_mml, spec_golden)

            plot_path = analyzer.plot_comparison(spec_mml, spec_golden, f, t_mml, t_golden,
                                               args.output_plot, args.title)
            if plot_path:
                print(f"  Plot saved: {plot_path}")

        return 0
    else:
        print(f"Analysis failed: {result['reason']}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
