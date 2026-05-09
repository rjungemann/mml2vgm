#!/usr/bin/env python3
"""
VGM Binary Comparison Tool

Compares two VGM files by extracting and comparing register writes.
Outputs timing variance and register accuracy metrics.
"""

import struct
import argparse
from pathlib import Path
from dataclasses import dataclass
from typing import List, Dict, Tuple
from collections import defaultdict


@dataclass
class RegisterWrite:
    """A single register write command in VGM."""
    time_sample: int
    chip_id: int
    register: int
    value: int
    command_type: str  # e.g., "YM2151", "SN76489"


class VgmParser:
    """Parse VGM 1.7x files and extract register writes."""

    # VGM command constants
    CMD_PSG = 0x50
    CMD_FM = 0x51
    CMD_FM2 = 0x52
    CMD_SEGA_PCM = 0x67
    CMD_OPL2 = 0x5B
    CMD_OPL3 = 0x5E
    CMD_DUAL_FM = 0x30
    CMD_WAIT_SAMPLES = 0x61
    CMD_WAIT_735 = 0x62
    CMD_WAIT_882 = 0x63
    CMD_DATA_BLOCK = 0x67
    CMD_END = 0x66

    def __init__(self, path: str):
        self.path = Path(path)
        self.data = self.path.read_bytes()
        self.pos = 0
        self.registers: List[RegisterWrite] = []
        self.current_time = 0

    def parse(self) -> List[RegisterWrite]:
        """Parse the VGM file and return register writes."""
        # Verify VGM header
        if self.data[:4] != b'Vgm ':
            raise ValueError("Not a valid VGM file")

        # Skip to command section (0x34 for VGM 1.50+)
        self.pos = 0x34
        current_time = 0

        while self.pos < len(self.data):
            cmd = self.data[self.pos]
            self.pos += 1

            if cmd == self.CMD_END:
                break
            elif cmd == self.CMD_WAIT_SAMPLES:
                # Wait 1 sample
                current_time += 1
            elif cmd == self.CMD_WAIT_735:
                # Wait 735 samples
                current_time += 735
            elif cmd == self.CMD_WAIT_882:
                # Wait 882 samples
                current_time += 882
            elif cmd == 0x61:  # Wait n samples
                wait_samples = struct.unpack('<H', self.data[self.pos:self.pos+2])[0]
                self.pos += 2
                current_time += wait_samples
            elif cmd == self.CMD_FM:  # YM2413
                register = self.data[self.pos]
                value = self.data[self.pos+1]
                self.pos += 2
                self.registers.append(RegisterWrite(current_time, 0, register, value, "YM2413"))
            elif cmd == self.CMD_PSG:  # SN76489
                value = self.data[self.pos]
                self.pos += 1
                self.registers.append(RegisterWrite(current_time, 0, value >> 5, value & 0x1F, "SN76489"))
            elif cmd == self.CMD_OPL2:  # OPL2
                register = self.data[self.pos]
                value = self.data[self.pos+1]
                self.pos += 2
                self.registers.append(RegisterWrite(current_time, 0, register, value, "OPL2"))
            elif cmd == self.CMD_OPL3:  # OPL3
                register = self.data[self.pos]
                value = self.data[self.pos+1]
                self.pos += 2
                self.registers.append(RegisterWrite(current_time, 0, register, value, "OPL3"))
            elif cmd == self.CMD_DATA_BLOCK:
                # Skip data block
                self.pos += 1  # data type
                size = struct.unpack('<I', self.data[self.pos:self.pos+4])[0]
                self.pos += 4 + size
            else:
                # Skip unknown command (simple implementation)
                break

        return self.registers


@dataclass
class ComparisonResult:
    """Results from VGM comparison."""
    total_writes: int
    matched_writes: int
    timing_variance_max: int
    timing_variance_avg: float
    register_accuracy: float
    passed: bool


def compare_vgm_files(golden_path: str, mml2vgm_path: str, tolerance_samples: int = 2) -> ComparisonResult:
    """
    Compare two VGM files.

    Args:
        golden_path: Path to golden master VGM
        mml2vgm_path: Path to mml2vgm output VGM
        tolerance_samples: Timing tolerance in samples

    Returns:
        ComparisonResult with accuracy metrics
    """
    # Parse both files
    golden_parser = VgmParser(golden_path)
    mml_parser = VgmParser(mml2vgm_path)

    golden_writes = golden_parser.parse()
    mml_writes = mml_parser.parse()

    # Group writes by chip and register
    def group_writes(writes: List[RegisterWrite]) -> Dict[Tuple[int, int], List[RegisterWrite]]:
        grouped = defaultdict(list)
        for w in writes:
            grouped[(w.chip_id, w.register)].append(w)
        return grouped

    golden_grouped = group_writes(golden_writes)
    mml_grouped = group_writes(mml_writes)

    # Compare
    matched = 0
    timing_variance = []
    total = len(golden_writes)

    for key in golden_grouped:
        if key in mml_grouped:
            golden_vals = golden_grouped[key]
            mml_vals = mml_grouped[key]

            # Compare first write at least
            for g, m in zip(golden_vals, mml_vals):
                if g.value == m.value:
                    matched += 1
                    timing_variance.append(abs(g.time_sample - m.time_sample))

    accuracy = (matched / total * 100) if total > 0 else 0
    max_variance = max(timing_variance) if timing_variance else 0
    avg_variance = sum(timing_variance) / len(timing_variance) if timing_variance else 0
    passed = accuracy >= 95 and max_variance <= 5  # Thresholds for pass

    return ComparisonResult(
        total_writes=total,
        matched_writes=matched,
        timing_variance_max=max_variance,
        timing_variance_avg=avg_variance,
        register_accuracy=accuracy,
        passed=passed
    )


def main():
    parser = argparse.ArgumentParser(
        description="Compare two VGM files for register write accuracy"
    )
    parser.add_argument("golden", help="Golden master VGM file")
    parser.add_argument("mml2vgm", help="mml2vgm output VGM file")
    parser.add_argument("--tolerance", type=int, default=2,
                       help="Timing tolerance in samples (default: 2)")

    args = parser.parse_args()

    try:
        result = compare_vgm_files(args.golden, args.mml2vgm, args.tolerance)

        print(f"VGM Comparison Results:")
        print(f"  Total register writes: {result.total_writes}")
        print(f"  Matched writes: {result.matched_writes}/{result.total_writes}")
        print(f"  Register accuracy: {result.register_accuracy:.1f}%")
        print(f"  Timing variance (max): {result.timing_variance_max} samples")
        print(f"  Timing variance (avg): {result.timing_variance_avg:.2f} samples")
        print(f"  Status: {'PASS' if result.passed else 'FAIL'}")

        return 0 if result.passed else 1

    except Exception as e:
        print(f"Error: {e}")
        return 1


if __name__ == "__main__":
    exit(main())
