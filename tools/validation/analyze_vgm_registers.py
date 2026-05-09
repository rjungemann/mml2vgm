#!/usr/bin/env python3
"""
VGM Register Analysis Tool

Analyzes VGM register writes and generates a detailed report of chip
initialization, note timing, and envelope patterns.
"""

import struct
from pathlib import Path
from collections import defaultdict
from typing import List, Dict, Tuple

class VgmAnalyzer:
    """Analyze VGM register writes in detail."""
    
    def __init__(self, vgm_path: str):
        self.path = Path(vgm_path)
        self.data = self.path.read_bytes()
        self.chip_commands = defaultdict(list)
        self.timestamps = []
        self.current_time = 0
        
    def parse(self):
        """Parse VGM and extract register sequences."""
        if self.data[:4] != b'Vgm ':
            raise ValueError("Not a valid VGM file")
        
        pos = 0x34
        current_time = 0
        
        # Get header info
        header_info = self._parse_header()
        
        while pos < len(self.data):
            cmd = self.data[pos]
            pos += 1
            
            if cmd == 0x66:  # End
                break
            elif cmd == 0x60:  # Wait 1 sample
                current_time += 1
            elif cmd == 0x61:  # Wait n samples
                wait = struct.unpack('<H', self.data[pos:pos+2])[0]
                pos += 2
                current_time += wait
                self.timestamps.append(current_time)
            elif cmd == 0x54:  # YM2151
                reg = self.data[pos]
                val = self.data[pos+1]
                pos += 2
                self.chip_commands['YM2151'].append((current_time, reg, val))
            elif cmd == 0x55:  # YM2203
                reg = self.data[pos]
                val = self.data[pos+1]
                pos += 2
                self.chip_commands['YM2203'].append((current_time, reg, val))
            elif cmd == 0x5B:  # OPL2
                reg = self.data[pos]
                val = self.data[pos+1]
                pos += 2
                self.chip_commands['OPL2'].append((current_time, reg, val))
            elif cmd == 0x5E:  # OPL3
                reg = self.data[pos]
                val = self.data[pos+1]
                pos += 2
                self.chip_commands['OPL3'].append((current_time, reg, val))
            elif cmd == 0x50:  # PSG
                val = self.data[pos]
                pos += 1
                self.chip_commands['SN76489'].append((current_time, val >> 5, val & 0x1F))
            elif cmd == 0xB0:  # NES pulse 1
                val = self.data[pos]
                pos += 1
                self.chip_commands['NES'].append((current_time, 0x4000, val))
            elif cmd == 0xB1:  # NES pulse 2
                val = self.data[pos]
                pos += 1
                self.chip_commands['NES'].append((current_time, 0x4004, val))
            elif cmd == 0xB2:  # NES triangle
                val = self.data[pos]
                pos += 1
                self.chip_commands['NES'].append((current_time, 0x4008, val))
            elif cmd == 0xB3:  # NES noise
                val = self.data[pos]
                pos += 1
                self.chip_commands['NES'].append((current_time, 0x400C, val))
            elif cmd == 0x67:  # Data block
                pos += 1
                size = struct.unpack('<I', self.data[pos:pos+4])[0]
                pos += 4 + size
            else:
                pass
        
        return header_info
    
    def _parse_header(self) -> Dict:
        """Parse VGM header for metadata."""
        info = {}
        try:
            # Clock frequencies at offsets 0x8C, 0x94, 0x9C, etc.
            ym2151_clock = struct.unpack('<I', self.data[0x8C:0x90])[0]
            if ym2151_clock > 0:
                info['ym2151_clock'] = ym2151_clock
                
            ym2203_clock = struct.unpack('<I', self.data[0x94:0x98])[0]
            if ym2203_clock > 0:
                info['ym2203_clock'] = ym2203_clock
        except:
            pass
        
        return info
    
    def analyze(self) -> Dict:
        """Generate analysis report."""
        header = self.parse()
        
        analysis = {
            'file': str(self.path),
            'size_bytes': len(self.data),
            'header': header,
            'chips': {},
            'timing': {
                'wait_points': len(self.timestamps),
                'total_samples': self.timestamps[-1] if self.timestamps else 0,
            }
        }
        
        for chip, commands in self.chip_commands.items():
            analysis['chips'][chip] = {
                'total_writes': len(commands),
                'time_range': (min(t for t,r,v in commands), max(t for t,r,v in commands)) if commands else (0, 0),
                'registers_used': len(set(r for t,r,v in commands)),
                'first_write': commands[0] if commands else None,
                'last_write': commands[-1] if commands else None,
                'sample_writes': commands[:5] if len(commands) > 5 else commands,
            }
        
        return analysis

def print_analysis(analysis: Dict):
    """Pretty-print analysis results."""
    print("=" * 70)
    print(f"VGM File: {Path(analysis['file']).name}")
    print("=" * 70)
    print(f"File size: {analysis['size_bytes']} bytes")
    print(f"Wait points: {analysis['timing']['wait_points']}")
    print(f"Total duration: {analysis['timing']['total_samples']} samples (~{analysis['timing']['total_samples']/48000:.2f}s at 48kHz)")
    print()
    
    for chip, info in analysis['chips'].items():
        print(f"Chip: {chip}")
        print(f"  Total register writes: {info['total_writes']}")
        print(f"  Registers used: {info['registers_used']}")
        print(f"  Time range: {info['time_range'][0]}-{info['time_range'][1]} samples")
        if info['first_write']:
            t, r, v = info['first_write']
            print(f"  First write: time={t}, reg=0x{r:02X}, val=0x{v:02X}")
        if info['last_write']:
            t, r, v = info['last_write']
            print(f"  Last write:  time={t}, reg=0x{r:02X}, val=0x{v:02X}")
        
        if info['sample_writes']:
            print(f"  Sample writes (first 5):")
            for t, r, v in info['sample_writes'][:5]:
                print(f"    time={t:6d} reg=0x{r:02X} val=0x{v:02X}")
        print()

if __name__ == "__main__":
    import sys
    if len(sys.argv) < 2:
        print("Usage: analyze_vgm_registers.py <vgm_file>")
        sys.exit(1)
    
    analyzer = VgmAnalyzer(sys.argv[1])
    analysis = analyzer.analyze()
    print_analysis(analysis)
