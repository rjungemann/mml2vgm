#!/usr/bin/env python3
"""
VGM Binary Validation Tool

Performs comprehensive validation of VGM register write sequences at the binary level.
This validates that the compiler generates correct chip commands and timing information.
"""

import struct
import json
import sys
from pathlib import Path
from datetime import datetime
from collections import defaultdict
from typing import Dict, List, Tuple

class VgmValidator:
    """Validate VGM files at the binary level."""

    # VGM command constants
    CMD_PSG = 0x50
    CMD_FM = 0x51
    CMD_FM2 = 0x52
    CMD_YM2151 = 0x54
    CMD_YM2203 = 0x55
    CMD_YM2608 = 0x56
    CMD_YM3812 = 0x5A
    CMD_YM3526 = 0x5B
    CMD_Y8950 = 0x5C
    CMD_YMF262 = 0x5E
    CMD_RF5C164 = 0x68
    CMD_C140 = 0x7F
    CMD_C352 = 0x8E
    CMD_AY8910 = 0xA0
    CMD_SEGAPCM = 0xA4
    CMD_DMG = 0xB3
    CMD_NESAPU = 0xB4
    CMD_VRC6 = 0xB6
    CMD_HUC6280 = 0xB9
    CMD_K053260 = 0xBA
    CMD_QSOUND = 0xC4
    CMD_K051649 = 0xD2
    CMD_K054539 = 0xD3
    CMD_WAIT_SAMPLES = 0x60
    CMD_WAIT_SAMPLES_2 = 0x61
    CMD_WAIT_735 = 0x62
    CMD_WAIT_882 = 0x63
    CMD_END = 0x66

    COMMAND_NAMES = {
        0x50: "PSG",
        0x51: "FM",
        0x52: "FM2",
        0x54: "YM2151",
        0x55: "YM2203",
        0x56: "YM2608",
        0x5A: "YM3812",
        0x5B: "YM3526",
        0x5C: "Y8950",
        0x5E: "OPL3",
        0x68: "RF5C164",
        0x7F: "C140",
        0x8E: "C352",
        0xA0: "AY8910",
        0xA4: "SegaPCM",
        0xB3: "DMG",
        0xB4: "NES APU",
        0xB6: "VRC6",
        0xB9: "HuC6280",
        0xBA: "K053260",
        0xC4: "QSound",
        0xD2: "K051649",
        0xD3: "K054539",
        0x60: "WAIT1",
        0x61: "WAITn",
        0x62: "WAIT735",
        0x63: "WAIT882",
        0x66: "END",
    }

    def __init__(self, path: str):
        self.path = Path(path)
        self.data = self.path.read_bytes()
        self.pos = 0
        self.errors = []
        self.warnings = []
        self.registers = defaultdict(list)
        self.waits = []
        self.current_time = 0

    def validate_header(self) -> bool:
        """Validate VGM file header."""
        if len(self.data) < 0x40:
            self.errors.append("File too small for VGM header")
            return False

        if self.data[:4] != b'Vgm ':
            self.errors.append("Invalid VGM signature")
            return False

        # Get EOF offset
        eof_offset = struct.unpack('<I', self.data[0x04:0x08])[0]
        if eof_offset + 4 != len(self.data):
            self.warnings.append(f"File size mismatch (expected {eof_offset+4}, got {len(self.data)})")

        # Check version
        version = struct.unpack('<I', self.data[0x08:0x0C])[0]
        if version < 0x150:
            self.warnings.append(f"Old VGM version: 0x{version:04X}")

        return True

    def parse_commands(self) -> int:
        """Parse all VGM commands and validate structure."""
        # Determine command start position
        version = struct.unpack('<I', self.data[0x08:0x0C])[0] if len(self.data) >= 0x0C else 0x150

        if version >= 0x150 and len(self.data) >= 0x38:
            # VGM 1.50+: data offset is at 0x34
            data_offset = struct.unpack('<I', self.data[0x34:0x38])[0]
            if data_offset == 0:
                self.pos = 0x40  # Default if offset is 0
            else:
                self.pos = 0x34 + data_offset
        else:
            # VGM 1.49 and below
            self.pos = 0x40

        command_count = 0

        while self.pos < len(self.data):
            if self.pos >= len(self.data):
                self.errors.append("Unexpected EOF while parsing commands")
                break

            cmd = self.data[self.pos]
            self.pos += 1
            command_count += 1

            try:
                if cmd == self.CMD_END:
                    break

                elif cmd == self.CMD_WAIT_SAMPLES:
                    self.current_time += 1
                    self.waits.append((self.current_time, 1))

                elif cmd == self.CMD_WAIT_735:
                    self.current_time += 735
                    self.waits.append((self.current_time, 735))

                elif cmd == self.CMD_WAIT_882:
                    self.current_time += 882
                    self.waits.append((self.current_time, 882))

                elif cmd == self.CMD_WAIT_SAMPLES_2:
                    if self.pos + 2 > len(self.data):
                        self.errors.append("Incomplete WAIT_SAMPLES_2 command")
                        break
                    wait_samples = struct.unpack('<H', self.data[self.pos:self.pos+2])[0]
                    self.pos += 2
                    self.current_time += wait_samples
                    self.waits.append((self.current_time, wait_samples))

                elif cmd in [self.CMD_FM, self.CMD_YM2151, self.CMD_YM2203,
                            self.CMD_YM2608, self.CMD_YM3812, self.CMD_YM3526, self.CMD_Y8950,
                            self.CMD_YMF262, self.CMD_RF5C164, self.CMD_C140, self.CMD_C352,
                            self.CMD_AY8910, self.CMD_SEGAPCM, self.CMD_DMG, self.CMD_NESAPU,
                            self.CMD_VRC6, self.CMD_HUC6280, self.CMD_K053260, self.CMD_QSOUND,
                            self.CMD_K051649, self.CMD_K054539]:
                    if self.pos + 2 > len(self.data):
                        self.errors.append(f"Incomplete {self.COMMAND_NAMES.get(cmd, 'UNKNOWN')} command")
                        break

                    register = self.data[self.pos]
                    value = self.data[self.pos+1]
                    self.pos += 2

                    chip_name = self.COMMAND_NAMES.get(cmd, f"UNKNOWN_{cmd:02X}")
                    self.registers[chip_name].append({
                        "time": self.current_time,
                        "register": register,
                        "value": value
                    })

                elif cmd == self.CMD_PSG:
                    if self.pos >= len(self.data):
                        self.errors.append("Incomplete PSG command")
                        break
                    value = self.data[self.pos]
                    self.pos += 1
                    self.registers["PSG"].append({
                        "time": self.current_time,
                        "register": value >> 5,
                        "value": value & 0x1F
                    })

                elif cmd == 0x67:  # Data block
                    if self.pos + 1 >= len(self.data):
                        self.errors.append("Incomplete data block header")
                        break
                    data_type = self.data[self.pos]
                    self.pos += 1
                    if self.pos + 4 > len(self.data):
                        self.errors.append("Incomplete data block size")
                        break
                    size = struct.unpack('<I', self.data[self.pos:self.pos+4])[0]
                    self.pos += 4 + size

                else:
                    if cmd not in [0x30, 0x3F]:  # Known optional commands
                        self.warnings.append(f"Unknown command: 0x{cmd:02X} at offset 0x{self.pos-1:04X}")

            except Exception as e:
                self.errors.append(f"Parse error at offset 0x{self.pos:04X}: {e}")
                break

        return command_count

    def analyze_register_patterns(self) -> Dict:
        """Analyze register write patterns for correctness."""
        analysis = {}

        for chip, writes in self.registers.items():
            if not writes:
                continue

            analysis[chip] = {
                "total_writes": len(writes),
                "unique_registers": len(set(w["register"] for w in writes)),
                "time_range": (min(w["time"] for w in writes), max(w["time"] for w in writes)),
                "register_freq": defaultdict(int),
                "timing_gaps": [],
                "issues": []
            }

            # Analyze register frequency
            for write in writes:
                analysis[chip]["register_freq"][write["register"]] += 1

            # Check timing consistency
            if len(writes) > 1:
                times = sorted(set(w["time"] for w in writes))
                gaps = [times[i+1] - times[i] for i in range(len(times)-1)]
                if gaps:
                    analysis[chip]["timing_gaps"] = {
                        "min": min(gaps),
                        "max": max(gaps),
                        "avg": sum(gaps) / len(gaps)
                    }

            # Chip-specific validation
            if chip == "YM2151":
                analysis[chip].update(self._validate_ym2151(writes))
            elif chip == "YM2203":
                analysis[chip].update(self._validate_ym2203(writes))
            elif chip == "NES" or chip == "PSG":
                analysis[chip].update(self._validate_nes(writes))

        return analysis

    def _validate_ym2151(self, writes: List) -> Dict:
        """Validate YM2151-specific patterns."""
        issues = []
        has_key_on = any(w["register"] == 0x08 for w in writes)

        if not has_key_on:
            issues.append("Missing key on/off register (0x08)")

        # Check for operator parameter writes
        op_regs = sum(1 for w in writes if 0x20 <= w["register"] <= 0x3F)
        if op_regs == 0:
            issues.append("No operator parameter writes detected")

        return {
            "has_key_on": has_key_on,
            "operator_writes": op_regs,
            "issues": issues
        }

    def _validate_ym2203(self, writes: List) -> Dict:
        """Validate YM2203-specific patterns."""
        issues = []
        has_mode = any(w["register"] == 0x27 for w in writes)
        has_key_on = any(w["register"] == 0x28 for w in writes)

        if not has_mode:
            issues.append("Missing mode register (0x27)")
        if not has_key_on:
            issues.append("Missing key on/off register (0x28)")

        return {
            "has_mode": has_mode,
            "has_key_on": has_key_on,
            "issues": issues
        }

    def _validate_nes(self, writes: List) -> Dict:
        """Validate NES-specific patterns."""
        issues = []
        if not writes:
            issues.append("No PSG commands found")

        return {
            "psg_writes": len(writes),
            "issues": issues
        }

    def generate_report(self) -> Dict:
        """Generate comprehensive validation report."""
        self.validate_header()
        command_count = self.parse_commands()
        analysis = self.analyze_register_patterns()

        return {
            "file": str(self.path),
            "timestamp": datetime.now().isoformat(),
            "file_size": len(self.data),
            "status": "PASS" if not self.errors else "FAIL",
            "summary": {
                "commands": command_count,
                "register_writes": sum(len(w) for w in self.registers.values()),
                "wait_commands": len(self.waits),
                "chips": list(self.registers.keys()),
                "total_duration_samples": self.current_time
            },
            "errors": self.errors,
            "warnings": self.warnings,
            "analysis": analysis
        }


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Validate VGM file structure and register writes")
    parser.add_argument("vgm_file", help="Path to VGM file")
    parser.add_argument("--json", action="store_true", help="Output as JSON")
    parser.add_argument("--verbose", action="store_true", help="Verbose output")

    args = parser.parse_args()

    validator = VgmValidator(args.vgm_file)
    report = validator.generate_report()

    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"VGM Validation Report: {args.vgm_file}")
        print("=" * 70)

        if report["status"] == "FAIL":
            print(f"Status: FAIL")
            for error in report["errors"]:
                print(f"  ERROR: {error}")
        else:
            print(f"Status: PASS")

        if report["warnings"]:
            for warning in report["warnings"]:
                print(f"  WARNING: {warning}")

        print()
        print(f"Summary:")
        print(f"  File size: {report['file_size']} bytes")
        print(f"  Commands: {report['summary']['commands']}")
        print(f"  Register writes: {report['summary']['register_writes']}")
        print(f"  Duration: {report['summary']['total_duration_samples']} samples " +
              f"({report['summary']['total_duration_samples']/48000:.2f}s @ 48kHz)")
        print()

        for chip, analysis in report["analysis"].items():
            print(f"{chip}:")
            print(f"  Total writes: {analysis['total_writes']}")
            print(f"  Unique registers: {analysis['unique_registers']}")
            if "issues" in analysis and analysis["issues"]:
                for issue in analysis["issues"]:
                    print(f"    ⚠ {issue}")
            print()

    return 0 if report["status"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
