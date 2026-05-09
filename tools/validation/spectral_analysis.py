#!/usr/bin/env python3
"""
Spectral Analysis Framework for Golden Master Validation

Compares VGM output against golden master references using STFT-based
spectral analysis. Produces correlation scores and frequency error metrics.
"""

import numpy as np
from scipy import signal
from scipy.io import wavfile
import matplotlib.pyplot as plt
import argparse
from pathlib import Path
from dataclasses import dataclass
from typing import Tuple, Optional


@dataclass
class ComparisonResult:
    """Results from spectral comparison between two audio files."""
    correlation: float
    freq_error_hz: float
    phase_coherence: float
    passed: bool
    details: str


def load_audio(path: str, sr: Optional[int] = None) -> Tuple[np.ndarray, int]:
    """Load WAV file and return PCM data and sample rate."""
    try:
        sr_file, data = wavfile.read(path)
        if data.dtype not in [np.float32, np.float64]:
            data = data.astype(np.float32) / np.iinfo(data.dtype).max
        if sr and sr != sr_file:
            raise ValueError(f"Sample rate mismatch: expected {sr}, got {sr_file}")
        return data, sr_file
    except Exception as e:
        raise IOError(f"Failed to load {path}: {e}")


def compute_stft(audio: np.ndarray, sr: int, n_fft: int = 512) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Compute STFT and return frequencies, times, and magnitude spectrogram."""
    f, t, Zxx = signal.stft(audio, sr, nperseg=n_fft, window='hann')
    return f, t, np.abs(Zxx)


def compare_spectrograms(
    golden: np.ndarray,
    mml2vgm: np.ndarray,
    sr: int,
    threshold: float = 0.95
) -> ComparisonResult:
    """
    Compare spectrograms using cosine similarity.

    Args:
        golden: Golden master PCM data
        mml2vgm: mml2vgm output PCM data
        sr: Sample rate
        threshold: Passing correlation threshold

    Returns:
        ComparisonResult with correlation, frequency error, and pass status
    """
    # Normalize lengths (pad shorter to match longer)
    min_len = min(len(golden), len(mml2vgm))
    golden = golden[:min_len]
    mml2vgm = mml2vgm[:min_len]

    # Compute STFTs
    f_golden, t_golden, spec_golden = compute_stft(golden, sr)
    f_mml, t_mml, spec_mml = compute_stft(mml2vgm, sr)

    # Flatten spectrograms for comparison
    spec_golden_flat = spec_golden.flatten()
    spec_mml_flat = spec_mml.flatten()

    # Cosine similarity
    correlation = np.dot(spec_golden_flat, spec_mml_flat) / (
        np.linalg.norm(spec_golden_flat) * np.linalg.norm(spec_mml_flat) + 1e-8
    )

    # Frequency error: compare peak frequencies
    golden_peaks = np.argmax(spec_golden, axis=0)
    mml_peaks = np.argmax(spec_mml, axis=0)
    freq_error = np.mean(np.abs(f_golden[golden_peaks] - f_golden[mml_peaks]))

    # Phase coherence: cross-correlation
    phase_coherence = np.max(np.correlate(golden, mml2vgm, mode='same')) / (
        np.sqrt(np.sum(golden**2) * np.sum(mml2vgm**2)) + 1e-8
    )

    passed = correlation >= threshold
    details = (
        f"Correlation: {correlation:.4f} (threshold: {threshold}), "
        f"Frequency error: {freq_error:.2f} Hz, "
        f"Phase coherence: {phase_coherence:.4f}"
    )

    return ComparisonResult(
        correlation=correlation,
        freq_error_hz=freq_error,
        phase_coherence=phase_coherence,
        passed=passed,
        details=details
    )


def plot_comparison(
    golden: np.ndarray,
    mml2vgm: np.ndarray,
    sr: int,
    output_path: str
) -> None:
    """Generate comparison plots (waveform and spectrogram)."""
    fig, axes = plt.subplots(3, 2, figsize=(14, 10))

    # Time-domain waveforms
    t_golden = np.arange(len(golden)) / sr
    t_mml = np.arange(len(mml2vgm)) / sr

    axes[0, 0].plot(t_golden[:sr], golden[:sr], linewidth=0.5)
    axes[0, 0].set_title("Golden Master (1 sec)")
    axes[0, 0].set_ylabel("Amplitude")

    axes[0, 1].plot(t_mml[:sr], mml2vgm[:sr], linewidth=0.5, color='orange')
    axes[0, 1].set_title("mml2vgm (1 sec)")
    axes[0, 1].set_ylabel("Amplitude")

    # Spectrograms
    f_golden, t_golden, spec_golden = compute_stft(golden, sr)
    f_mml, t_mml, spec_mml = compute_stft(mml2vgm, sr)

    im1 = axes[1, 0].pcolormesh(t_golden, f_golden, 20*np.log10(spec_golden + 1e-8),
                                shading='auto', cmap='viridis')
    axes[1, 0].set_title("Golden Master Spectrogram")
    axes[1, 0].set_ylabel("Frequency (Hz)")
    plt.colorbar(im1, ax=axes[1, 0], label="dB")

    im2 = axes[1, 1].pcolormesh(t_mml, f_mml, 20*np.log10(spec_mml + 1e-8),
                                shading='auto', cmap='viridis')
    axes[1, 1].set_title("mml2vgm Spectrogram")
    axes[1, 1].set_ylabel("Frequency (Hz)")
    plt.colorbar(im2, ax=axes[1, 1], label="dB")

    # Difference
    min_len = min(len(spec_golden[0]), len(spec_mml[0]))
    diff = spec_golden[:, :min_len] - spec_mml[:, :min_len]
    im3 = axes[2, 0].pcolormesh(t_golden[:min_len], f_golden,
                                20*np.log10(np.abs(diff) + 1e-8),
                                shading='auto', cmap='RdBu_r')
    axes[2, 0].set_title("Spectrogram Difference")
    axes[2, 0].set_ylabel("Frequency (Hz)")
    axes[2, 0].set_xlabel("Time (s)")
    plt.colorbar(im3, ax=axes[2, 0], label="dB")

    # Magnitude comparison at time slices
    mid_golden = spec_golden[:, len(spec_golden[0])//2]
    mid_mml = spec_mml[:, len(spec_mml[0])//2]
    axes[2, 1].semilogy(f_golden, mid_golden, label="Golden", linewidth=1)
    axes[2, 1].semilogy(f_mml, mid_mml, label="mml2vgm", linewidth=1)
    axes[2, 1].set_title("Magnitude at Mid-point")
    axes[2, 1].set_ylabel("Magnitude")
    axes[2, 1].set_xlabel("Frequency (Hz)")
    axes[2, 1].legend()
    axes[2, 1].grid(True, alpha=0.3)

    plt.tight_layout()
    plt.savefig(output_path, dpi=150)
    print(f"Saved comparison plot to {output_path}")


def main():
    parser = argparse.ArgumentParser(
        description="Spectral analysis comparison tool for golden master validation"
    )
    parser.add_argument("golden", help="Golden master WAV file")
    parser.add_argument("mml2vgm", help="mml2vgm output WAV file")
    parser.add_argument("--threshold", type=float, default=0.95,
                       help="Correlation threshold for pass (default: 0.95)")
    parser.add_argument("--plot", help="Save comparison plot to this path")

    args = parser.parse_args()

    # Load audio files
    try:
        golden, sr_golden = load_audio(args.golden)
        mml2vgm, sr_mml = load_audio(args.mml2vgm, sr=sr_golden)
    except IOError as e:
        print(f"Error: {e}")
        return 1

    # Compare spectrograms
    result = compare_spectrograms(golden, mml2vgm, sr_golden, args.threshold)

    # Output results
    print(f"Comparison Results:")
    print(f"  {result.details}")
    print(f"  Status: {'PASS' if result.passed else 'FAIL'}")

    # Generate plots if requested
    if args.plot:
        plot_comparison(golden, mml2vgm, sr_golden, args.plot)

    return 0 if result.passed else 1


if __name__ == "__main__":
    exit(main())
