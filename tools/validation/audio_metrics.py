#!/usr/bin/env python3
"""
Phase 2 Audio Validation Analysis
Analyzes generated golden master audio files and generates validation metrics
"""

import os
import json
import numpy as np
from pathlib import Path
from typing import Dict, List
from scipy.io import wavfile
from scipy import signal, fft


class AudioMetricsAnalyzer:
    """Analyzes audio files and generates validation metrics"""
    
    def __init__(self, sample_rate: int = 44100):
        self.sample_rate = sample_rate
        self.metrics = {}
    
    def analyze_wav(self, wav_file: str) -> Dict:
        """Analyze single WAV file and extract metrics"""
        try:
            sr, data = wavfile.read(wav_file)
            
            # Convert to mono if stereo
            if len(data.shape) > 1:
                data = np.mean(data, axis=1)
            
            # Normalize to float
            if data.dtype == np.int16:
                data = data.astype(np.float32) / 32768.0
            elif data.dtype == np.int32:
                data = data.astype(np.float32) / 2147483648.0
            
            metrics = {
                'file': os.path.basename(wav_file),
                'sample_rate': sr,
                'duration_seconds': len(data) / sr,
                'num_samples': len(data),
            }
            
            # Compute spectral metrics
            metrics['spectral'] = self._compute_spectral_metrics(data, sr)
            
            # Compute temporal metrics
            metrics['temporal'] = self._compute_temporal_metrics(data)
            
            # Determine status
            metrics['status'] = self._determine_status(metrics)
            
            return metrics
            
        except Exception as e:
            return {
                'file': os.path.basename(wav_file),
                'error': str(e),
                'status': 'ERROR'
            }
    
    def _compute_spectral_metrics(self, audio: np.ndarray, sr: int) -> Dict:
        """Compute frequency domain metrics"""
        # FFT analysis
        n = len(audio)
        window = signal.windows.hann(n)
        windowed = audio * window
        
        fft_result = np.abs(fft.fft(windowed))[:n // 2]
        frequencies = fft.fftfreq(n, 1 / sr)[:n // 2]
        
        # Find peak frequency
        peak_idx = np.argmax(fft_result)
        peak_freq = frequencies[peak_idx]
        peak_magnitude = fft_result[peak_idx]
        
        # Spectral centroid (center of mass in frequency domain)
        centroid = np.sum(frequencies * fft_result) / (np.sum(fft_result) + 1e-10)
        
        # Spectral spread (standard deviation around centroid)
        spread = np.sqrt(np.sum(((frequencies - centroid) ** 2) * fft_result) / 
                        (np.sum(fft_result) + 1e-10))
        
        # Energy in different frequency bands
        energy_low = np.sum(fft_result[(frequencies >= 20) & (frequencies < 250)])  # Bass
        energy_mid = np.sum(fft_result[(frequencies >= 250) & (frequencies < 2000)])  # Mids
        energy_high = np.sum(fft_result[(frequencies >= 2000) & (frequencies < 20000)])  # Treble
        total_energy = np.sum(fft_result)
        
        return {
            'peak_frequency_hz': float(peak_freq),
            'peak_magnitude_db': 20 * np.log10(peak_magnitude + 1e-10),
            'centroid_hz': float(centroid),
            'spread_hz': float(spread),
            'energy_low_fraction': float(energy_low / (total_energy + 1e-10)),
            'energy_mid_fraction': float(energy_mid / (total_energy + 1e-10)),
            'energy_high_fraction': float(energy_high / (total_energy + 1e-10)),
            'total_energy': float(total_energy),
        }
    
    def _compute_temporal_metrics(self, audio: np.ndarray) -> Dict:
        """Compute time domain metrics"""
        # RMS level (loudness)
        rms = np.sqrt(np.mean(audio ** 2))
        rms_db = 20 * np.log10(rms + 1e-10)
        
        # Peak amplitude
        peak = np.max(np.abs(audio))
        peak_db = 20 * np.log10(peak + 1e-10)
        
        # Crest factor (peak/RMS)
        crest_factor = peak / (rms + 1e-10)
        
        # Zero crossing rate (indicator of frequency content)
        zero_crossings = np.sum(np.abs(np.diff(np.sign(audio)))) / 2
        zcr = zero_crossings / len(audio)
        
        return {
            'rms_db': float(rms_db),
            'peak_db': float(peak_db),
            'crest_factor': float(crest_factor),
            'zero_crossing_rate': float(zcr),
        }
    
    def _determine_status(self, metrics: Dict) -> str:
        """Determine PASS/WARN/FAIL status"""
        if 'error' in metrics:
            return 'ERROR'
        
        # Check for valid audio
        spectral = metrics.get('spectral', {})
        temporal = metrics.get('temporal', {})
        
        total_energy = spectral.get('total_energy', 0)
        rms_db = temporal.get('rms_db', -120)
        peak_db = temporal.get('peak_db', -120)
        
        # Criteria: has content if RMS > -60 dB
        if rms_db > -60:
            return 'PASS'
        elif rms_db > -80:
            return 'WARN'
        else:
            return 'FAIL'
    
    def batch_analyze(self, audio_dir: str) -> Dict[str, Dict]:
        """Analyze all WAV files in directory"""
        results = {}
        wav_files = sorted(Path(audio_dir).glob('*.wav'))
        
        print("\n" + "="*70)
        print("AUDIO METRICS ANALYSIS")
        print("="*70)
        print(f"\nDirectory: {audio_dir}")
        print(f"Files:     {len(wav_files)}\n")
        
        for wav_file in wav_files:
            print(f"  Analyzing: {wav_file.name}")
            metrics = self.analyze_wav(str(wav_file))
            results[wav_file.name] = metrics
            
            if metrics['status'] != 'ERROR':
                spectral = metrics.get('spectral', {})
                print(f"    Status: {metrics['status']}")
                print(f"    RMS: {metrics['temporal']['rms_db']:.2f} dB")
                print(f"    Peak Freq: {spectral['peak_frequency_hz']:.1f} Hz")
            else:
                print(f"    Status: ERROR - {metrics['error']}")
        
        # Summary
        statuses = [m['status'] for m in results.values()]
        print(f"\n{'='*70}")
        print(f"ANALYSIS SUMMARY")
        print(f"{'='*70}\n")
        print(f"  Total:  {len(results)}")
        print(f"  ✅ Pass: {statuses.count('PASS')}")
        print(f"  ⚠️  Warn: {statuses.count('WARN')}")
        print(f"  ❌ Fail: {statuses.count('FAIL')}")
        print(f"  ❌ Error: {statuses.count('ERROR')}\n")
        
        pass_rate = (statuses.count('PASS') * 100) // len(statuses) if statuses else 0
        print(f"  Pass Rate: {pass_rate}%\n")
        
        return results


def main():
    """Run audio validation analysis"""
    analyzer = AudioMetricsAnalyzer()
    
    audio_dir = '/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/golden_masters'
    output_file = '/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/audio_metrics.json'
    
    # Analyze all files
    results = analyzer.batch_analyze(audio_dir)
    
    # Save results
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    with open(output_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"✅ Results saved to: {output_file}\n")


if __name__ == '__main__':
    main()
