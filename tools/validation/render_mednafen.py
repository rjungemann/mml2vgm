#!/usr/bin/env python3
"""
Mednafen VGM Audio Rendering Wrapper

Renders VGM files to audio WAV format using Mednafen's system emulators.
Supports multiple emulated systems (MSX, Sega CD, PC Engine, ColecoVision).
"""

import os
import subprocess
import json
import tempfile
from pathlib import Path
from typing import Optional, Dict, Tuple

# Emulator configuration
MEDNAFEN_PATH = "/opt/homebrew/bin/mednafen"
MEDNAFEN_CONFIG = {
    "sample_rate": 44100,
    "bit_depth": 16,
    "channels": 2,
    "format": "wav",
}

# System configurations for different chips
SYSTEM_CONFIG = {
    "YM2413": {
        "system": "msx",
        "module": "opll",
        "duration": 5,  # seconds
    },
    "Y8950": {
        "system": "msx2",
        "module": "opl",
        "duration": 5,
    },
    "RF5C164": {
        "system": "scd",
        "module": "pcm",
        "duration": 5,
    },
    "C140": {
        "system": "arcade",  # Namco system
        "module": "c140",
        "duration": 5,
    },
    "C352": {
        "system": "arcade",  # Namco system
        "module": "c352",
        "duration": 5,
    },
    "K053260": {
        "system": "arcade",  # Konami system
        "module": "konami",
        "duration": 5,
    },
    "K054539": {
        "system": "arcade",  # Konami system
        "module": "konami",
        "duration": 5,
    },
    "AY8910": {
        "system": "coleco",  # ColecoVision
        "module": "psg",
        "duration": 5,
    },
    "HuC6280": {
        "system": "pce",  # PC Engine
        "module": "psg",
        "duration": 5,
    },
}


class MedfanaenVGMRenderer:
    """Render VGM files to audio using Mednafen system emulation"""

    def __init__(self, mednafen_path: str = MEDNAFEN_PATH):
        self.mednafen_path = mednafen_path
        self.config = MEDNAFEN_CONFIG.copy()
        self.verify_mednafen()

    def verify_mednafen(self) -> bool:
        """Verify Mednafen is installed and accessible"""
        import os
        if not os.path.exists(self.mednafen_path):
            raise RuntimeError(f"Mednafen not found at {self.mednafen_path}")
        
        if not os.access(self.mednafen_path, os.X_OK):
            raise RuntimeError(f"Mednafen is not executable at {self.mednafen_path}")
        
        print(f"✅ Mednafen found at {self.mednafen_path}")
        return True

    def render_vgm_to_wav(
        self,
        vgm_file: str,
        chip: str,
        output_wav: str,
        duration: Optional[int] = None,
    ) -> bool:
        """
        Render a VGM file to WAV audio using Mednafen system emulation
        
        Args:
            vgm_file: Path to input VGM file
            chip: Chip name (YM2413, Y8950, etc.)
            output_wav: Path to output WAV file
            duration: Override duration in seconds
            
        Returns:
            True if successful, False otherwise
        """
        
        if not os.path.exists(vgm_file):
            print(f"❌ VGM file not found: {vgm_file}")
            return False
        
        if chip not in SYSTEM_CONFIG:
            print(f"❌ Chip not supported: {chip}")
            return False
        
        config = SYSTEM_CONFIG[chip]
        dur = duration or config["duration"]
        
        try:
            # Create temporary M3U playlist
            with tempfile.NamedTemporaryFile(mode='w', suffix='.m3u', delete=False) as m3u:
                m3u.write(vgm_file + '\n')
                m3u_path = m3u.name
            
            # Build Mednafen command
            # Note: Direct VGM playback via Mednafen requires specific setup
            # For now, this is a framework for future enhancement
            cmd = [
                self.mednafen_path,
                "-force_aspect", "1",
                "-video.fs", "0",
                "-sounddriver", "wav",
                "-soundfile", output_wav,
                m3u_path,
            ]
            
            print(f"  Rendering {chip}: {os.path.basename(vgm_file)}")
            print(f"    System: {config['system']}, Duration: {dur}s")
            
            # Execute Mednafen
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=dur + 10,
            )
            
            # Clean up temp M3U
            try:
                os.unlink(m3u_path)
            except:
                pass
            
            if result.returncode == 0 and os.path.exists(output_wav):
                size_kb = os.path.getsize(output_wav) / 1024
                print(f"    ✅ Generated: {os.path.basename(output_wav)} ({size_kb:.1f} KB)")
                return True
            else:
                print(f"    ⚠️  Rendering completed but WAV not found")
                if result.stderr:
                    print(f"    Error: {result.stderr[:200]}")
                return False
                
        except subprocess.TimeoutExpired:
            print(f"    ❌ Timeout (>{dur + 10}s)")
            return False
        except Exception as e:
            print(f"    ❌ Error: {str(e)}")
            return False

    def batch_render(
        self,
        vgm_dir: str,
        output_dir: str,
    ) -> Dict[str, bool]:
        """
        Batch render all VGM files in a directory
        
        Args:
            vgm_dir: Directory containing VGM files
            output_dir: Output directory for WAV files
            
        Returns:
            Dictionary mapping VGM filename to render success
        """
        
        os.makedirs(output_dir, exist_ok=True)
        results = {}
        
        vgm_files = sorted(Path(vgm_dir).glob("*.vgm"))
        
        if not vgm_files:
            print(f"❌ No VGM files found in {vgm_dir}")
            return results
        
        print(f"\n{'='*70}")
        print(f"MEDNAFEN VGM AUDIO RENDERING")
        print(f"{'='*70}\n")
        print(f"Input:  {vgm_dir}")
        print(f"Output: {output_dir}")
        print(f"Files:  {len(vgm_files)}")
        print(f"\n{'='*70}\n")
        
        success_count = 0
        fail_count = 0
        
        for vgm_file in vgm_files:
            vgm_name = vgm_file.name
            
            # Determine chip from VGM filename
            chip = self._guess_chip_from_filename(vgm_name)
            if not chip:
                print(f"⚠️  {vgm_name}: Could not determine chip type")
                results[vgm_name] = False
                fail_count += 1
                continue
            
            output_wav = os.path.join(output_dir, vgm_name.replace('.vgm', '.wav'))
            
            if self.render_vgm_to_wav(str(vgm_file), chip, output_wav):
                results[vgm_name] = True
                success_count += 1
            else:
                results[vgm_name] = False
                fail_count += 1
        
        # Print summary
        print(f"\n{'='*70}")
        print(f"RENDERING SUMMARY")
        print(f"{'='*70}\n")
        print(f"  ✅ Successful: {success_count}/{len(vgm_files)}")
        print(f"  ❌ Failed:     {fail_count}/{len(vgm_files)}")
        print(f"  Pass Rate:    {success_count*100//len(vgm_files)}%\n")
        
        return results

    @staticmethod
    def _guess_chip_from_filename(filename: str) -> Optional[str]:
        """Guess chip type from VGM filename"""
        filename_lower = filename.lower()
        
        for chip in SYSTEM_CONFIG.keys():
            if chip.lower() in filename_lower:
                return chip
        
        return None


def main():
    """Example usage of Mednafen VGM renderer"""
    
    renderer = MedfanaenVGMRenderer()
    
    # Paths
    vgm_dir = "/Users/rjungemann/Projects/mml2vgm/validation_results/phase2"
    output_dir = "/Users/rjungemann/Projects/mml2vgm/validation_results/phase2/golden_masters"
    
    # Batch render
    results = renderer.batch_render(vgm_dir, output_dir)
    
    # Save results
    results_file = os.path.join(output_dir, "_rendering_results.json")
    with open(results_file, 'w') as f:
        json.dump(results, f, indent=2)
    
    print(f"\n✅ Results saved to: {results_file}\n")


if __name__ == "__main__":
    main()
