#!/usr/bin/env python3
"""
Audio Spectral Analysis Framework

Analyzes WAV files and compares mml2vgm output against golden masters.
Computes frequency domain, temporal, and perceptual metrics.
"""

import os
import json
import numpy as np
from pathlib import Path
from typing import Dict, Tuple, Optional
from scipy import signal, fft
from scipy.io import wavfile


class SpectralAnalyzer:
    """Performs spectral analysis on audio WAV files"""

    def __init__(self, sample_rate: int = 44100):
        self.sample_rate = sample_rate
        self.freq_resolution = sample_rate / 2048  # Default FFT window

    def load_wav(self, filename: str) -> Tuple[int, np.ndarray]:
        """Load WAV file and return sample rate and audio data"""
        try:
            sr, data = wavfile.read(filename)
            # Convert to mono if stereo
            if len(data.shape) > 1:
                data = np.mean(data, axis=1)
            return sr, data
        except Exception as e:
            print(f"❌ Error loading {filename}: {e}")
            return None, None

    def compute_fft(self, audio: np.ndarray, sample_rate: int) -> Tuple[np.ndarray, np.ndarray]:
        """Compute FFT magnitude and frequency array"""
        n = len(audio)
        window = signal.windows.hann(n)
        windowed = audio * window
        
        fft_result = np.abs(fft.fft(windowed))[:n // 2]
        frequencies = fft.fftfreq(n, 1 / sample_rate)[:n // 2]
        
        return frequencies, fft_result

    def find_fundamental_frequency(self, audio: np.ndarray, sample_rate: int) -> float:
        """Find fundamental frequency (F0) using autocorrelation"""
        # Simple autocorrelation-based F0 detection
        # For full implementation, would use more sophisticated methods
        
        frequencies, magnitude = self.compute_fft(audio, sample_rate)
        
        # Find peak in 20 Hz - 5 kHz range
        mask = (frequencies >= 20) & (frequencies <= 5000)
        peak_idx = np.argmax(magnitude[mask]) + np.where(mask)[0][0]
        
        f0 = frequencies[peak_idx]
        return max(20, min(5000, f0))  # Clamp to reasonable range

    def analyze_harmonics(self, audio: np.ndarray, sample_rate: int, f0: Optional[float] = None) -> Dict:
        """Analyze harmonic content"""
        if f0 is None:
            f0 = self.find_fundamental_frequency(audio, sample_rate)
        
        frequencies, magnitude = self.compute_fft(audio, sample_rate)
        
        # Normalize magnitude
        mag_db = 20 * np.log10(magnitude + 1e-10)
        
        harmonics = {}
        for harmonic in range(1, 11):  # First 10 harmonics
            target_freq = f0 * harmonic
            # Find nearest frequency bin
            idx = np.argmin(np.abs(frequencies - target_freq))
            harmonics[f"harmonic_{harmonic}"] = {
                "frequency": float(frequencies[idx]),
                "magnitude_db": float(mag_db[idx]),
                "error_hz": float(abs(frequencies[idx] - target_freq)),
            }
        
        return {
            "fundamental_hz": float(f0),
            "harmonics": harmonics,
        }

    def analyze_envelope(self, audio: np.ndarray, sample_rate: int) -> Dict:
        """Analyze envelope (ADSR)"""
        # Compute instantaneous energy
        energy = audio ** 2
        
        # Smooth with moving average
        window_size = sample_rate // 100  # 10ms window
        smoothed = np.convolve(energy, np.ones(window_size) / window_size, mode='same')
        
        # Find attack, decay, sustain, release points
        peak_idx = np.argmax(smoothed)
        peak_time = peak_idx / sample_rate
        peak_energy = smoothed[peak_idx]
        
        # Simple envelope characterization
        attack_time = peak_time
        tail_energy = np.mean(smoothed[peak_idx:])
        release_time = (len(audio) - peak_idx) / sample_rate if tail_energy > 0.1 * peak_energy else 0
        
        return {
            "attack_time_ms": float(attack_time * 1000),
            "peak_time_ms": float(peak_time * 1000),
            "release_time_ms": float(release_time * 1000),
            "total_duration_ms": float(len(audio) / sample_rate * 1000),
            "peak_energy_db": float(10 * np.log10(peak_energy + 1e-10)),
            "sustain_level_db": float(10 * np.log10(tail_energy + 1e-10)),
        }

    def compute_spectral_centroid(self, audio: np.ndarray, sample_rate: int) -> float:
        """Compute spectral centroid"""
        frequencies, magnitude = self.compute_fft(audio, sample_rate)
        centroid = np.sum(frequencies * magnitude) / (np.sum(magnitude) + 1e-10)
        return float(centroid)

    def compute_loudness(self, audio: np.ndarray) -> float:
        """Compute approximate loudness in dB"""
        rms = np.sqrt(np.mean(audio ** 2))
        loudness_db = 20 * np.log10(rms + 1e-10)
        return float(loudness_db)

    def analyze_audio(self, filename: str) -> Dict:
        """Comprehensive audio analysis"""
        sr, audio = self.load_wav(filename)
        
        if audio is None:
            return {"status": "ERROR", "filename": filename}
        
        # Normalize audio
        audio = audio / (np.max(np.abs(audio)) + 1e-10)
        
        # Analyze various aspects
        f0 = self.find_fundamental_frequency(audio, sr)
        
        return {
            "status": "OK",
            "filename": filename,
            "sample_rate": sr,
            "duration_ms": float(len(audio) / sr * 1000),
            "loudness_db": self.compute_loudness(audio),
            "spectral_centroid_hz": self.compute_spectral_centroid(audio, sr),
            "harmonics": self.analyze_harmonics(audio, sr, f0),
            "envelope": self.analyze_envelope(audio, sr),
        }


class AudioComparator:
    """Compares two audio files and computes similarity metrics"""

    def __init__(self):
        self.analyzer = SpectralAnalyzer()

    def compare_files(self, file1: str, file2: str, label1: str = "File1", label2: str = "File2") -> Dict:
        """Compare two audio files and return metrics"""
        
        analysis1 = self.analyzer.analyze_audio(file1)
        analysis2 = self.analyzer.analyze_audio(file2)
        
        if analysis1.get("status") != "OK" or analysis2.get("status") != "OK":
            return {
                "status": "ERROR",
                "message": "Failed to analyze one or both files"
            }
        
        # Extract key metrics
        f0_1 = analysis1["harmonics"]["fundamental_hz"]
        f0_2 = analysis2["harmonics"]["fundamental_hz"]
        
        loudness_1 = analysis1["loudness_db"]
        loudness_2 = analysis2["loudness_db"]
        
        centroid_1 = analysis1["spectral_centroid_hz"]
        centroid_2 = analysis2["spectral_centroid_hz"]
        
        # Compute differences
        f0_error_hz = abs(f0_1 - f0_2)
        f0_error_percent = (f0_error_hz / max(f0_1, f0_2)) * 100 if max(f0_1, f0_2) > 0 else 0
        
        loudness_error_db = abs(loudness_1 - loudness_2)
        
        centroid_error_hz = abs(centroid_1 - centroid_2)
        centroid_error_percent = (centroid_error_hz / max(centroid_1, centroid_2)) * 100 if max(centroid_1, centroid_2) > 0 else 0
        
        # Harmonic comparison
        harmonic_errors = []
        for harmonic in range(1, 11):
            key = f"harmonic_{harmonic}"
            h1 = analysis1["harmonics"]["harmonics"][key]
            h2 = analysis2["harmonics"]["harmonics"][key]
            
            mag_diff = abs(h1["magnitude_db"] - h2["magnitude_db"])
            harmonic_errors.append(mag_diff)
        
        mean_harmonic_error_db = float(np.mean(harmonic_errors))
        
        # Determine pass/fail status
        f0_pass = f0_error_hz < 1.0  # < 1 Hz error
        loudness_pass = loudness_error_db < 3.0  # < 3 dB error
        harmonic_pass = mean_harmonic_error_db < 3.0  # < 3 dB error
        
        status = "PASS" if all([f0_pass, loudness_pass, harmonic_pass]) else "WARN" if any([f0_pass, loudness_pass, harmonic_pass]) else "FAIL"
        
        return {
            "status": status,
            "label1": label1,
            "label2": label2,
            "file1": file1,
            "file2": file2,
            "metrics": {
                "fundamental_frequency": {
                    f"{label1}_hz": float(f0_1),
                    f"{label2}_hz": float(f0_2),
                    "error_hz": float(f0_error_hz),
                    "error_percent": float(f0_error_percent),
                    "pass": bool(f0_pass),
                },
                "loudness": {
                    f"{label1}_db": float(loudness_1),
                    f"{label2}_db": float(loudness_2),
                    "error_db": float(loudness_error_db),
                    "pass": bool(loudness_pass),
                },
                "spectral_centroid": {
                    f"{label1}_hz": float(centroid_1),
                    f"{label2}_hz": float(centroid_2),
                    "error_hz": float(centroid_error_hz),
                    "error_percent": float(centroid_error_percent),
                },
                "harmonics": {
                    "mean_error_db": float(mean_harmonic_error_db),
                    "pass": bool(harmonic_pass),
                },
            },
        }

    def batch_compare(self, directory1: str, directory2: str) -> Dict:
        """Compare all WAV files in two directories"""
        
        files1 = sorted(Path(directory1).glob("*.wav"))
        files2 = sorted(Path(directory2).glob("*.wav"))
        
        if not files1:
            return {"status": "ERROR", "message": f"No WAV files in {directory1}"}
        
        if not files2:
            return {"status": "ERROR", "message": f"No WAV files in {directory2}"}
        
        print(f"\n{'='*70}")
        print(f"AUDIO COMPARISON")
        print(f"{'='*70}\n")
        print(f"Reference: {len(files1)} files in {directory1}")
        print(f"Compare:   {len(files2)} files in {directory2}\n")
        
        results = {
            "status": "OK",
            "reference_dir": directory1,
            "compare_dir": directory2,
            "comparisons": [],
            "summary": {
                "total": 0,
                "pass": 0,
                "warn": 0,
                "fail": 0,
            }
        }
        
        # Match files by name
        for file1 in files1:
            matching_file2 = Path(directory2) / file1.name
            
            if not matching_file2.exists():
                print(f"⚠️  No matching file: {file1.name}")
                continue
            
            print(f"Comparing: {file1.name}")
            
            comparison = self.compare_files(
                str(file1),
                str(matching_file2),
                label1="Reference",
                label2="mml2vgm"
            )
            
            results["comparisons"].append(comparison)
            
            # Update summary
            results["summary"]["total"] += 1
            status = comparison.get("status", "UNKNOWN")
            if status in results["summary"]:
                results["summary"][status] += 1
            
            # Print result
            print(f"  {status}: F0={comparison['metrics']['fundamental_frequency']['error_hz']:.2f} Hz, "
                  f"Loudness={comparison['metrics']['loudness']['error_db']:.2f} dB")
        
        # Print summary
        total = results["summary"]["total"]
        if total > 0:
            pass_rate = results["summary"]["pass"] * 100 // total
            print(f"\n{'='*70}")
            print(f"SUMMARY")
            print(f"{'='*70}\n")
            print(f"  Total:  {total}")
            print(f"  ✅ Pass: {results['summary']['pass']}")
            print(f"  ⚠️  Warn: {results['summary']['warn']}")
            print(f"  ❌ Fail: {results['summary']['fail']}")
            print(f"  Pass Rate: {pass_rate}%\n")
        
        return results


def main():
    """Example usage of audio comparison"""
    
    comparator = AudioComparator()
    
    # Paths
    reference_dir = "/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/golden_masters_reference"
    compare_dir = "/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/golden_masters_mml2vgm"
    
    # Batch compare
    results = comparator.batch_compare(reference_dir, compare_dir)
    
    # Save results
    output_file = "/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/audio_comparison_results.json"
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"✅ Results saved to: {output_file}\n")


if __name__ == "__main__":
    main()
