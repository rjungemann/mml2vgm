#!/usr/bin/env python3
"""
Convert Furnace .fur tracker modules to MML format for mml2vgm.

This script parses the binary .fur format and generates MML source code.
The .fur format is documented in the Furnace source code (src/engine/fileOps/fur.cpp).

Usage:
    python convert_fur_to_mml.py <input.fur> <output.mml>
    python convert_fur_to_mml.py --batch <directory>
"""

import sys
import os
import zlib
import struct
import argparse
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple
from enum import Enum

# Furnace magic constants
FUR_MAGIC = b"-Furnace module-"

# System IDs (from Furnace's systemFromFileFur in fileOpsCommon.cpp)
# Mapping of Furnace file format IDs to mml2vgm system names
SYSTEM_ID_TO_NAME = {
    0x00: "NULL",
    0x01: "PSG",         # SN76489
    0x8F: "PSG",         # SN76489 (old format ID)
    0x02: "YM2612",     # Genesis
    0x03: "YM2612X",    # Extended YM2612
    0x04: "YM2612X2",   # Dual YM2612
    0x05: "YM2151",     # OPM
    0x06: "YM2203",     # OPN
    0x07: "YM2608",     # OPNA (PC-98)
    0x08: "YM2610",     # OPNB (Neo Geo)
    0x09: "YM2610B",    # OPNB2
    0x0A: "YM3526",     # OPL
    0x0B: "YM3812",     # OPL2
    0x0C: "YMF262",     # OPL3
    0x0D: "Y8950",      # OPL with ADPCM
    0x0E: "YMF278",     # OPL4
    0x0F: "YM2413",     # OPLL
    0x10: "YM2414",     # OPZ
    0x11: "HuC6280",    # PC Engine
    0x12: "C140",       # Namco
    0x13: "AY8910",     # AY-3-8910
    0x14: "AY8930",     # Microchip AY8930
    0x15: "DMG",        # Game Boy
    0x16: "SMS_PSG",    # SMS PSG
    0x17: "NES",        # NES APU
    # Additional old format IDs
    0xC8: "SM8521",     # Sharp SM8521
    0xD5: "POKEY",      # Atari POKEY (old format)
    0xD6: "GB",         # Game Boy (old format)
    0xA0: "YM2612",     # Genesis (newer format ID)
    # Add more as needed
    0x18: "FDS",        # Famicom Disk System
    0x19: "VRC6",       # Konami VRC6
    0x1A: "VRC7",       # Konami VRC7
    0x1B: "MMC5",       # MMC5
    0x1C: "S5B",        # Sunsoft 5B
    0x1D: "N163",       # Namco 163
    0x1E: "SID",        # C64 SID
    0x1F: "SID2",       # Dual SID
    0x20: "POKEY",      # Atari POKEY
    0x21: "TIA",        # Atari 2600 TIA
    0x22: "SAA1099",    # SAM Coupé
    0x23: "SNES",       # SNES
    0x24: "QSOUND",     # Capcom QSound
    0x25: "YMZ280B",    # PCM DAC
    0x26: "RF5C164",    # Ricoh RF5C164
    0x27: "SEGAPCM",    # Sega PCM
    0x28: "MSM5232",    # OKI MSM5232
    0x29: "K051649",    # Konami K051649
    0x2A: "K053260",    # Konami K053260
    0x2B: "K054539",    # Konami K054539
    0x2C: "C352",       # Namco C352
    0x2D: "GA20",       # Irem GA20
    0x2E: "ES5506",     # Ensoniq ES5506
    0x2F: "X1_010",     # Seta X1-010
    0x30: "C140",       # Namco C140 (duplicate?)
    0x31: "C219",       # Namco C219
}

# MML system names that mml2vgm understands
# Maps our internal names to mml2vgm's system identifiers
MML_SYSTEM_MAP = {
    "SN76489": "$PSG",
    "PSG": "$PSG",
    "YM2612": "$FM1",
    "YM2612X": "$FM1",
    "YM2612X2": "$FM2",
    "YM2151": "$OPM",
    "YM2203": "$OPN",
    "YM2608": "$OPNA",
    "YM2610": "$OPNB",
    "YM2610B": "$OPNB2",
    "YM3526": "$OPL",
    "YM3812": "$OPL2",
    "YMF262": "$OPL3",
    "Y8950": "$OPL",
    "YMF278": "$OPL4",
    "YM2413": "$OPLL",
    "YM2414": "$OPZ",
    "HuC6280": "$HUC",
    "C140": "$C140",
    "C219": "$C219",
    "AY8910": "$AY8910",
    "AY8930": "$AY8930",
    "DMG": "$GB",
    "NES": "$NES",
    "FDS": "$FDS",
    "VRC6": "$VRC6",
    "VRC7": "$VRC7",
    "MMC5": "$MMC5",
    "S5B": "$S5B",
    "N163": "$N163",
    "POKEY": "$POKEY",
    "QSOUND": "$QSOUND",
    "YMZ280B": "$YMZ280B",
    "RF5C164": "$RF5C164",
    "SEGAPCM": "$SEGAPCM",
    "SM8521": "$SM8521",
    "GB": "$GB",
}

# Systems that mml2vgm supports (from SoundChip enum)
SUPPORTED_SYSTEMS = {
    "SN76489", "PSG", "YM2612", "YM2612X", "YM2612X2", "YM2608", "YM2609",
    "YM2610B", "YM2151", "YM3526", "Y8950", "YM3812", "YMF262",
    "YMF271", "YM2413", "YM2203", "RF5C164", "SegaPCM", "HuC6280",
    "C140", "C352", "AY8910", "AY8930", "K051649", "K053260", "K054539",
    "QSound", "NES", "DMG", "GB", "VRC6", "POKEY", "MIDI",
    "SM8521", "SMS_PSG"
}

class FileElementType(Enum):
    SUBSONG = 0
    CHIP_FLAGS = 1
    ASSET_DIR = 2
    INSTRUMENT = 3
    WAVETABLE = 4
    SAMPLE = 5
    PATTERN = 6
    COMPAT_FLAGS = 7
    COMMENTS = 8
    GROOVE = 9
    END = 10
    MAX = 11

@dataclass
class SongInfo:
    name: str = ""
    author: str = ""
    system_name: str = ""
    category: str = ""
    version: int = 0
    tuning: float = 440.0
    auto_system: bool = False
    master_vol: float = 1.0
    num_channels: int = 0
    systems: List[Tuple[str, int]] = field(default_factory=list)  # (system_name, num_channels)
    system_vols: List[float] = field(default_factory=list)
    system_pans: List[float] = field(default_factory=list)
    system_pan_frs: List[float] = field(default_factory=list)
    pattern_length: int = 0
    orders_length: int = 0
    num_instruments: int = 0
    num_wavetables: int = 0
    num_samples: int = 0
    num_patterns: int = 0
    hz: float = 60.0

class BinaryReader:
    """Helper class for reading binary data."""
    
    def __init__(self, data):
        self.data = data
        self.pos = 0
    
    def read(self, n):
        """Read n bytes."""
        if self.pos + n > len(self.data):
            raise EOFError(f"Cannot read {n} bytes at position {self.pos}")
        result = self.data[self.pos:self.pos+n]
        self.pos += n
        return result
    
    def read_u8(self):
        """Read unsigned 8-bit integer."""
        return struct.unpack('<B', self.read(1))[0]
    
    def read_u16(self):
        """Read unsigned 16-bit integer (little-endian)."""
        return struct.unpack('<H', self.read(2))[0]
    
    def read_u32(self):
        """Read unsigned 32-bit integer (little-endian)."""
        return struct.unpack('<I', self.read(4))[0]
    
    def read_i8(self):
        """Read signed 8-bit integer."""
        return struct.unpack('<b', self.read(1))[0]
    
    def read_i16(self):
        """Read signed 16-bit integer (little-endian)."""
        return struct.unpack('<h', self.read(2))[0]
    
    def read_i32(self):
        """Read signed 32-bit integer (little-endian)."""
        return struct.unpack('<i', self.read(4))[0]
    
    def read_f32(self):
        """Read 32-bit float (little-endian)."""
        return struct.unpack('<f', self.read(4))[0]
    
    def read_string(self):
        """Read a null-terminated string."""
        result = b""
        while True:
            byte = self.read(1)
            if byte == b"\x00" or len(byte) == 0:
                break
            result += byte
        return result.decode('utf-8', errors='replace')
    
    def skip(self, n):
        """Skip n bytes."""
        self.pos += n
        if self.pos > len(self.data):
            self.pos = len(self.data)
    
    def seek(self, pos, whence=0):
        """Seek to position. whence: 0=absolute, 1=relative, 2=from end."""
        if whence == 0:
            self.pos = pos
        elif whence == 1:
            self.pos += pos
        elif whence == 2:
            self.pos = len(self.data) - pos
        if self.pos < 0:
            self.pos = 0
        if self.pos > len(self.data):
            self.pos = len(self.data)
        return self.pos < len(self.data)
    
    def tell(self):
        """Get current position."""
        return self.pos
    
    def at_end(self):
        return self.pos >= len(self.data)

class FurParser:
    def __init__(self, filepath):
        self.filepath = filepath
        self.data = None
        self.decompressed = None
        self.song_info = SongInfo()
        self.patterns: List = []
        self.instruments: List = []
        self.subsongs = []
        
    def load(self):
        """Load and decompress the .fur file."""
        with open(self.filepath, 'rb') as f:
            self.data = f.read()
        
        # Check if already uncompressed
        if self.data.startswith(FUR_MAGIC):
            self.decompressed = self.data
        else:
            # Try to decompress
            try:
                self.decompressed = zlib.decompress(self.data)
            except zlib.error as e:
                raise ValueError(f"Failed to decompress .fur file: {e}")
        
        # Check magic on decompressed data
        if not self.decompressed.startswith(FUR_MAGIC):
            # Try alternative magic
            if not self.decompressed.startswith(FUR_MAGIC_DS0):
                raise ValueError(f"Not a valid .fur file (magic mismatch): {self.filepath}")
        
        self.song_info.version = struct.unpack('<H', self.decompressed[16:18])[0]
    
    def parse(self):
        """Parse the decompressed data."""
        reader = BinaryReader(self.decompressed)
        
        # Skip magic (16 bytes)
        reader.skip(16)
        
        # Read version (already read in load, but read it again for consistency)
        self.song_info.version = reader.read_u16()
        
        # Read reserved
        reader.skip(2)
        
        # Read INFO offset
        info_offset = reader.read_u32()
        
        # Seek to INFO
        if not reader.seek(info_offset):
            raise ValueError("Could not seek to INFO section")
        
        # Check INFO magic
        info_magic = reader.read(4)
        
        if self.song_info.version >= 240:
            if info_magic != b"INF2":
                raise ValueError(f"Expected INF2 magic for version {self.song_info.version}, got {info_magic}")
            self._parse_new_format(reader)
        else:
            if info_magic != b"INFO":
                raise ValueError(f"Expected INFO magic for version {self.song_info.version}, got {info_magic}")
            self._parse_old_format(reader)
    
    def _parse_old_format(self, reader):
        """Parse the old (pre-v240) format.
        
        For old format files, the structure is complex and version-dependent.
        For now, we just extract basic metadata and create stub patterns.
        Full pattern parsing for old format would require more work.
        """
        reader.skip(4)  # Skip length (4 bytes)
        
        # Read old header fields
        old_time_base = reader.read_u8()
        speed1 = reader.read_u8()
        speed2 = reader.read_u8()
        arp_len = reader.read_u8()
        self.song_info.hz = reader.read_f32()
        
        self.song_info.pattern_length = reader.read_u16()
        self.song_info.orders_length = reader.read_u16()
        
        hilight_a = reader.read_u8()
        hilight_b = reader.read_u8()
        
        self.song_info.num_instruments = reader.read_u16()
        self.song_info.num_wavetables = reader.read_u16()
        self.song_info.num_samples = reader.read_u16()
        self.song_info.num_patterns = reader.read_u32()
        
        # Read system IDs (256 max, null-terminated)
        systems = []
        for i in range(256):
            if reader.tell() >= len(reader.data):
                break
            sys_id = reader.read_u8()
            if sys_id == 0:
                break
            system_name = SYSTEM_ID_TO_NAME.get(sys_id, f"UNKNOWN_0x{sys_id:02X}")
            systems.append(system_name)
        
        self.song_info.systems = [(s, 0) for s in systems]  # Channel counts not stored in old format
        
        # Read system volumes (signed bytes, /64.0)
        system_vols = []
        for _ in range(len(systems)):
            if reader.tell() >= len(reader.data):
                break
            vol = reader.read_i8()
            system_vols.append(max(0, vol / 64.0))
        self.song_info.system_vols = system_vols
        
        # Read system pans (signed bytes, /127.0)
        system_pans = []
        for _ in range(len(systems)):
            if reader.tell() >= len(reader.data):
                break
            pan = reader.read_i8()
            system_pans.append(pan / 127.0)
        self.song_info.system_pans = system_pans
        
        # Read system flag pointers
        sys_flags_ptr = []
        for _ in range(len(systems)):
            if reader.tell() >= len(reader.data):
                break
            sys_flags_ptr.append(reader.read_u32())
        
        # Read song name and author
        if reader.tell() < len(reader.data):
            self.song_info.name = reader.read_string()
        if reader.tell() < len(reader.data):
            self.song_info.author = reader.read_string()
        
        # Tuning (version >= 33)
        if self.song_info.version >= 33 and reader.tell() < len(reader.data):
            self.song_info.tuning = reader.read_f32()
        
        # Skip compat flags (complex, version-dependent)
        # For old format, we'll just create stub patterns
        # In reality, patterns come after subsong data which is complex
        # For now, just create placeholder patterns
        for i in range(min(self.song_info.num_patterns, 10)):
            self.patterns.append({
                "rows": self.song_info.pattern_length,
                "channels": len(self.song_info.systems) or 1
            })
        
        # Create placeholder instruments
        for i in range(self.song_info.num_instruments):
            self.instruments.append({"name": f"Inst{i+1}", "type": 0})
    
    def _read_compat_flags_old(self, reader):
        """Read compatibility flags for old format."""
        # This is simplified - we skip the compat flags for now
        if self.song_info.version >= 43:
            reader.read_u8()  # properNoiseLayout
        if self.song_info.version >= 43:
            reader.read_u8()  # waveDutyIsVol
        if self.song_info.version >= 45:
            for _ in range(5):
                reader.read_u8()
        if self.song_info.version >= 47:
            reader.read_u8()  # arpNonPorta
    
    def _read_subsong_old(self, reader, systems):
        """Read sub-song data for old format."""
        # This is a simplified version
        # In old format, there's one sub-song
        pass
    
    def _read_instruments_old(self, reader):
        """Read instruments for old format.
        
        In old format, instruments are stored with pointers.
        For now, we skip this and use generic instrument names.
        """
        # Skip instrument data for now - it's complex in old format
        # Just create placeholder instruments
        for i in range(self.song_info.num_instruments):
            self.instruments.append({"name": f"Inst{i+1}", "type": 0})
    
    def _read_patterns_old(self, reader):
        """Read patterns for old format."""
        for i in range(self.song_info.num_patterns):
            try:
                rows = self.song_info.pattern_length  # All patterns have same length in old format
                channels = len(self.song_info.systems)  # Channels = number of systems
                
                # Read pattern data: rows * channels * 5 bytes per cell
                pattern_data = []
                for r in range(rows):
                    row_data = []
                    for c in range(channels):
                        # Each cell is 5 bytes: note, ins, vol, effect, param
                        cell = {
                            'note': reader.read_u8(),
                            'ins': reader.read_u8(),
                            'vol': reader.read_u8(),
                            'effect': reader.read_u8(),
                            'param': reader.read_u8(),
                        }
                        row_data.append(cell)
                    pattern_data.append(row_data)
                
                self.patterns.append({
                    "rows": rows, 
                    "channels": channels,
                    "data": pattern_data
                })
            except EOFError:
                break
    
    def _parse_new_format(self, reader):
        """Parse the new (v240+) format."""
        reader.skip(4)  # Skip length
        
        # Read song metadata
        self.song_info.name = reader.read_string()
        self.song_info.author = reader.read_string()
        self.song_info.system_name = reader.read_string()
        self.song_info.category = reader.read_string()
        
        # Skip Japanese strings
        reader.read_string()  # nameJ
        reader.read_string()  # authorJ
        reader.read_string()  # systemNameJ
        reader.read_string()  # categoryJ
        
        self.song_info.tuning = reader.read_f32()
        self.song_info.auto_system = reader.read_u8() != 0
        
        # Read audio settings
        self.song_info.master_vol = reader.read_f32()
        self.song_info.num_channels = reader.read_u16()
        system_len = reader.read_u16()
        
        # Read system definitions
        total_chans = 0
        for i in range(system_len):
            sys_id = reader.read_u16()
            system_name = SYSTEM_ID_TO_NAME.get(sys_id, f"UNKNOWN_0x{sys_id:04X}")
            chans = reader.read_u16()
            vol = reader.read_f32()
            pan = reader.read_f32()
            pan_fr = reader.read_f32()
            
            self.song_info.systems.append((system_name, chans))
            self.song_info.system_vols.append(vol)
            self.song_info.system_pans.append(pan)
            self.song_info.system_pan_frs.append(pan_fr)
            total_chans += chans
        
        # Read patchbay
        conn_count = reader.read_u32()
        for _ in range(conn_count):
            reader.read_u32()
        patchbay_auto = reader.read_u8() != 0
        
        # Read element pointers (just store them for now, we'll process them later)
        element_data = {}
        while True:
            elem_type = reader.read_u8()
            if elem_type >= FileElementType.END.value:
                break
            
            elem_type_enum = FileElementType(elem_type)
            count = reader.read_u32()
            # Bounds check - don't read more pointers than there is space for
            if count > 10000 or reader.tell() + count * 4 > len(reader.data):
                # Skip this element - pointer count is unreasonable
                break
            pointers = [reader.read_u32() for _ in range(count)]
            
            element_data[elem_type_enum] = pointers
        
        # Now process the elements we care about
        if FileElementType.INSTRUMENT in element_data:
            self.song_info.num_instruments = len(element_data[FileElementType.INSTRUMENT])
            self._read_instruments_new(reader, element_data[FileElementType.INSTRUMENT])
        
        if FileElementType.PATTERN in element_data:
            self.song_info.num_patterns = len(element_data[FileElementType.PATTERN])
            self._read_patterns_new(reader, element_data[FileElementType.PATTERN])
        
        if FileElementType.WAVETABLE in element_data:
            self.song_info.num_wavetables = len(element_data[FileElementType.WAVETABLE])
        
        if FileElementType.SAMPLE in element_data:
            self.song_info.num_samples = len(element_data[FileElementType.SAMPLE])
    
    def _read_instruments_new(self, reader, pointers):
        """Read instruments for new format."""
        # Instrument type constants (from Furnace)
        INST_TYPE_FM = 1
        INST_TYPE_SAMPLE = 2
        INST_TYPE_AY = 3
        INST_TYPE_GB = 4
        INST_TYPE_PCE = 5
        INST_TYPE_NES = 6
        INST_TYPE_C64 = 7
        INST_TYPE_OPLL = 8
        INST_TYPE_OPL = 9
        INST_TYPE_SMS = 10
        
        for ptr in pointers:
            try:
                offset = reader.tell()
                reader.seek(ptr)
                
                # Check magic
                magic = reader.read(4)
                if magic != b"INS2":
                    reader.seek(offset)
                    continue
                
                # Read length (4 bytes)
                ins_length = reader.read_u32()
                
                # Read version (2 bytes)
                ins_version = reader.read_u16()
                
                # Read instrument type
                ins_type = reader.read_u8()
                reader.skip(1)  # reserved
                
                # Read instrument name
                ins_name = reader.read_string()
                
                # Parse based on type
                ins_data = {"name": ins_name, "type": ins_type}
                
                # Read features until end of instrument block
                end_pos = ptr + 8 + ins_length  # magic(4) + length(4)
                
                while reader.tell() < end_pos:
                    feature_id = reader.read_u8()
                    if feature_id == 0xFF:  # End of features
                        break
                    
                    feature_length = reader.read_u32()
                    feature_end = reader.tell() + feature_length
                    
                    if ins_type == INST_TYPE_FM and feature_id == 0:  # FM feature
                        self._read_fm_instrument(reader, ins_data)
                    elif ins_type == INST_TYPE_AY and feature_id == 0:  # AY feature
                        self._read_ay_instrument(reader, ins_data)
                    elif ins_type == INST_TYPE_OPLL and feature_id == 0:  # OPLL feature
                        self._read_opl_instrument(reader, ins_data)
                    else:
                        # Skip unknown feature
                        reader.seek(feature_end)
                    
                    reader.seek(feature_end)
                
                self.instruments.append(ins_data)
                reader.seek(offset)
            except (EOFError, Exception) as e:
                # Skip this instrument
                reader.seek(offset)
                pass
    
    def _read_fm_instrument(self, reader, ins_data):
        """Read FM instrument data."""
        # Read operator count
        op_count_byte = reader.read_u8()
        num_ops = op_count_byte & 0x0F
        op_flags = op_count_byte & 0xF0
        
        enabled_ops = []
        for i in range(4):
            if op_count_byte & (16 << i):
                enabled_ops.append(i)
        
        # Read alg and fb
        next_byte = reader.read_u8()
        alg = (next_byte >> 4) & 0x07
        fb = next_byte & 0x07
        
        # Read fms/ams
        next_byte = reader.read_u8()
        fms2 = (next_byte >> 5) & 0x07
        ams = (next_byte >> 3) & 0x03
        fms = next_byte & 0x07
        
        # Read ams2/ops/opllPreset
        next_byte = reader.read_u8()
        ams2 = (next_byte >> 6) & 0x03
        ops = 4 if (next_byte & 0x20) else 2
        opll_preset = next_byte & 0x1F
        
        # Read block/freq
        if reader.read_u8() > 0:  # Check version or flag
            block = reader.read_u8() & 0x0F
        else:
            block = 0
        
        fm_params = {
            "alg": alg,
            "fb": fb,
            "fms": fms,
            "ams": ams,
            "ops": ops,
            "block": block,
            "operators": []
        }
        
        # Read operators
        for i in range(num_ops):
            op_data = {}
            
            # Read KS/DT/MULT
            byte1 = reader.read_u8()
            op_data["ksr"] = 1 if (byte1 & 0x80) else 0
            op_data["dt"] = (byte1 >> 4) & 0x07
            op_data["mult"] = byte1 & 0x0F
            
            # Read SUS/TL
            byte2 = reader.read_u8()
            op_data["sus"] = 1 if (byte2 & 0x80) else 0
            op_data["tl"] = byte2 & 0x7F
            
            # Read RS/VIB/AR
            byte3 = reader.read_u8()
            op_data["rs"] = (byte3 >> 6) & 0x03
            op_data["vib"] = 1 if (byte3 & 0x20) else 0
            op_data["ar"] = byte3 & 0x1F
            
            # Read AM/KSL/DR
            byte4 = reader.read_u8()
            op_data["am"] = 1 if (byte4 & 0x80) else 0
            op_data["ksl"] = (byte4 >> 5) & 0x03
            op_data["dr"] = byte4 & 0x1F
            
            # Read EGT/KVS/D2R
            byte5 = reader.read_u8()
            op_data["egt"] = 1 if (byte5 & 0x80) else 0
            op_data["kvs"] = (byte5 >> 5) & 0x03
            op_data["d2r"] = byte5 & 0x1F
            
            # Read SL/RR
            byte6 = reader.read_u8()
            op_data["sl"] = (byte6 >> 4) & 0x0F
            op_data["rr"] = byte6 & 0x0F
            
            # Read DVB/SSG-Env
            byte7 = reader.read_u8()
            op_data["dvb"] = (byte7 >> 4) & 0x0F
            op_data["ssg_env"] = byte7 & 0x0F
            
            # Read DAM/DT2/WS
            byte8 = reader.read_u8()
            op_data["dam"] = (byte8 >> 5) & 0x07
            op_data["dt2"] = (byte8 >> 3) & 0x03
            op_data["ws"] = byte8 & 0x07
            
            fm_params["operators"].append(op_data)
        
        ins_data["fm"] = fm_params
    
    def _read_ay_instrument(self, reader, ins_data):
        """Read AY-3-8910 instrument data."""
        # AY instruments have simpler parameters
        # This is a placeholder for now
        pass
    
    def _read_opl_instrument(self, reader, ins_data):
        """Read OPL instrument data."""
        # OPL instruments have their own format
        # This is a placeholder for now
        pass
    
    def _read_patterns_new(self, reader, pointers):
        """Read patterns for new format."""
        for ptr in pointers:
            try:
                offset = reader.tell()
                reader.seek(ptr)
                rows = reader.read_u16()
                channels = reader.read_u16()
                self.patterns.append({"rows": rows, "channels": channels})
                reader.seek(offset)
            except (EOFError, Exception):
                pass
    
    def _read_subsongs_new(self, reader, pointers):
        """Read sub-songs for new format."""
        for ptr in pointers:
            try:
                offset = reader.tell()
                reader.seek(ptr)
                # Simplified - just skip for now
                reader.seek(offset)
            except (EOFError, Exception):
                pass

# Note value mapping from Furnace to MML
# Furnace uses: 0=C, 1=C#, 2=D, ..., 11=B, then next octave
NOTE_NAMES = ['C', 'C+', 'D', 'D+', 'E', 'F', 'F+', 'G', 'G+', 'A', 'A+', 'B']

class MMLGenerator:
    """Generate MML from parsed .fur data."""
    
    def __init__(self, song_info: SongInfo, patterns: List, instruments: List):
        self.song_info = song_info
        self.patterns = patterns
        self.instruments = instruments
    
    def _note_num_to_mml(self, note_num):
        """Convert Furnace note number to MML note string."""
        if note_num == 0:
            return 'r'  # Rest
        if note_num == 0xff:
            return 'r'  # Note off
        if note_num >= 0xfe:
            return 'r'  # Special note values
        
        note_idx = (note_num - 1) % 12
        octave = (note_num - 1) // 12
        
        if note_idx < 0 or note_idx >= len(NOTE_NAMES):
            return 'r'
        
        note_name = NOTE_NAMES[note_idx]
        return f"{note_name}{octave}"
    
    def _effect_to_mml(self, effect, param):
        """Convert Furnace effect to MML commands.
        
        Based on Furnace effect codes from playback.cpp:
        - 0x00: Arpeggio
        - 0x01: Pitch slide up
        - 0x02: Pitch slide down  
        - 0x03: Portamento
        - 0x04: Vibrato
        - 0x05: Vol slide + vibrato
        - 0x06: Vol slide + porta
        - 0x07: Tremolo
        - 0x08: Panning
        - 0x0A: Volume slide
        - 0x0B: Change order (Dxx)
        - 0x0C: Retrigger (Sxx)
        - 0x0D: Next order
        - 0x0F: Speed
        - 0x11: Global volume slide
        - 0xED: Delay
        - 0xFD: Virtual tempo num
        - 0xFE: Virtual tempo den
        """
        if effect == 0:
            return ""
        
        # Map effect code and param to MML
        # Note: In Furnace, effect is the command, param is the value
        effect_commands = {
            0x00: lambda p: f"Q{p:02X}" if p > 0 else "",  # Arpeggio
            0x01: lambda p: f"Q+p{p:02X}" if p > 0 else "",  # Pitch slide up
            0x02: lambda p: f"Q-p{p:02X}" if p > 0 else "",  # Pitch slide down
            0x03: lambda p: f"MP{p:02X}" if p > 0 else "",  # Portamento speed
            0x04: lambda p: f"MV{p:02X}" if p > 0 else "",  # Vibrato
            0x0A: lambda p: self._vol_slide_to_mml(p),  # Volume slide
            0x0B: lambda p: f"D{p:02X}" if p > 0 else "",  # Change order (jump)
            0x0C: lambda p: f"S{p:02X}" if p > 0 else "",  # Retrigger
            0x0F: lambda p: f"t{p:02X}" if p > 0 else "",  # Speed/tempo
        }
        
        handler = effect_commands.get(effect)
        if handler:
            return handler(param)
        
        # For unknown effects, skip them (return empty string)
        # Unmapped effects could be system-specific or extended effects
        return ""
    
    def _vol_slide_to_mml(self, param):
        """Convert volume slide effect to MML.
        
        Furnace volume slide uses a single byte where:
        - High nibble: slide up value
        - Low nibble: slide down value
        E.g., 0x12 = slide up 1, slide down 2
        MML uses: V+value or V-value
        """
        if param == 0:
            return ""
        up = (param >> 4) & 0x0F
        down = param & 0x0F
        parts = []
        if up > 0:
            parts.append(f"V+{up}")
        if down > 0:
            parts.append(f"V-{down}")
        return " ".join(parts) if parts else ""
    
    def _generate_pattern_mml(self, pattern):
        """Generate MML for a single pattern."""
        if 'data' not in pattern:
            return "  ; Pattern data not available\n"
        
        lines = []
        rows = pattern.get('rows', 0)
        channels = pattern.get('channels', 0)
        
        # Group by channel
        for c in range(min(channels, 8)):  # Limit to 8 channels for simplicity
            channel_lines = []
            for r in range(rows):
                cell = pattern['data'][r][c] if r < len(pattern['data']) and c < len(pattern['data'][r]) else None
                if cell:
                    note = self._note_num_to_mml(cell['note'])
                    vol = cell['vol']
                    effect = cell.get('effect', 0)
                    param = cell.get('param', 0)
                    
                    # Build MML cell
                    parts = []
                    if note != 'r':
                        parts.append(note)
                    else:
                        parts.append('r')
                    
                    # Add volume if not default
                    if vol > 0 and vol != 0x40:  # 0x40 is default volume
                        parts.append(f"V{vol}")
                    
                    # Add effect
                    effect_mml = self._effect_to_mml(effect, param)
                    if effect_mml:
                        parts.append(effect_mml)
                    
                    channel_lines.append("".join(parts))
                else:
                    channel_lines.append("r")
            
            if channel_lines:
                lines.append(f"  [{' '.join(channel_lines)}]")
        
        return "\n".join(lines) + "\n"
    
    def generate(self) -> str:
        """Generate MML source code."""
        lines = []
        
        # Add header with song metadata
        if self.song_info.name:
            lines.append(f"; {self.song_info.name}")
        if self.song_info.author:
            lines.append(f"; By: {self.song_info.author}")
        if self.song_info.system_name:
            lines.append(f"; System: {self.song_info.system_name}")
        if self.song_info.category:
            lines.append(f"; Category: {self.song_info.category}")
        lines.append("")
        
        # Add system definitions
        system_defs = []
        for i, (system_name, chans) in enumerate(self.song_info.systems):
            mml_system = MML_SYSTEM_MAP.get(system_name, f"${system_name}")
            system_defs.append(mml_system)
        
        if system_defs:
            lines.append(", ".join(system_defs) + ",")
            lines.append("")
        
        # Add ALL directive
        lines.append("!ALL")
        lines.append("")
        
        # Add tempo if hz is not default
        if self.song_info.hz != 60.0:
            lines.append(f"@T{self.song_info.hz:.2f}")
            lines.append("")
        
        # Add instrument definitions
        for ins in self.instruments:
            ins_name = ins.get('name', 'Unnamed')
            ins_type = ins.get('type', 0)
            
            if 'fm' in ins:
                # Generate FM instrument definition
                fm = ins['fm']
                alg = fm.get('alg', 0)
                fb = fm.get('fb', 0)
                ops = fm.get('operators', [])
                
                # Build operator parameters for mml2vgm
                # mml2vgm FM format: Ealgo,fb,ar1,dr1,sr1,rr1,sl1,tl1,ks1,ml1,dt1,am1
                # For multiple operators: separated by @@
                op_params = []
                for op in ops:
                    ar = op.get('ar', 0)
                    dr = op.get('dr', 0)
                    sr = op.get('d2r', 0)  # or sr?
                    rr = op.get('rr', 0)
                    sl = op.get('sl', 0)
                    tl = op.get('tl', 0)
                    ks = op.get('ksr', 0) * 2 + op.get('ksl', 0)  # Combine KSR and KSL
                    ml = op.get('mult', 0)
                    dt = op.get('dt', 0)
                    am = 1 if op.get('am', False) else 0
                    
                    param_str = f"{ar},{dr},{sr},{rr},{sl},{tl},{ks},{ml},{dt},{am}"
                    op_params.append(param_str)
                
                # Join operators with @ symbol
                if op_params:
                    fm_def = f"E{alg},{fb},{'@@'.join(op_params)}"
                    lines.append(f"${ins_name}, {fm_def}")
                else:
                    lines.append(f"${ins_name},")
            elif ins_type == 3:  # AY
                lines.append(f"${ins_name}, AY")
            else:
                lines.append(f"${ins_name},")
        
        if self.instruments:
            lines.append("")
        
        # Add pattern data
        if self.patterns:
            for i, pat in enumerate(self.patterns):
                lines.append(f"$$ALL")
                lines.append(self._generate_pattern_mml(pat))
        else:
            lines.append("$$ALL")
            lines.append("  ; TODO: Add pattern data")
            lines.append("")
        
        return "\n".join(lines)


def is_system_supported(system_name):
    """Check if a system is supported by mml2vgm."""
    return system_name in SUPPORTED_SYSTEMS or any(
        system_name in s or s in system_name
        for s in SUPPORTED_SYSTEMS
    )


def convert_file(input_path, output_path):
    """Convert a single .fur file to MML."""
    print(f"\nConverting {os.path.basename(input_path)}...")
    
    try:
        parser = FurParser(input_path)
        parser.load()
        parser.parse()
        
        # Check if any system is supported
        supported = any(is_system_supported(s) for s, _ in parser.song_info.systems)
        if not supported:
            print(f"  Skipping: no supported systems found")
            return False
        
        generator = MMLGenerator(parser.song_info, parser.patterns, parser.instruments)
        mml = generator.generate()
        
        with open(output_path, 'w') as f:
            f.write(mml)
        
        print(f"  -> {os.path.basename(output_path)} ({len(mml)} bytes)")
        return True
    except Exception as e:
        print(f"  ERROR: {e}")
        import traceback
        traceback.print_exc()
        return False


def batch_convert(directory, output_dir=None):
    """Batch convert all .fur files in a directory."""
    if output_dir is None:
        output_dir = os.path.join(directory, "mml")
    
    os.makedirs(output_dir, exist_ok=True)
    
    fur_files = []
    for root, dirs, files in os.walk(directory):
        for f in files:
            if f.lower().endswith('.fur') and os.path.isfile(os.path.join(root, f)):
                fur_files.append(os.path.join(root, f))
    
    print(f"Found {len(fur_files)} .fur files")
    
    success = 0
    failed = 0
    skipped = 0
    
    for fur_file in sorted(fur_files):
        rel_path = os.path.relpath(fur_file, directory)
        mml_path = os.path.join(output_dir, os.path.splitext(rel_path)[0] + '.mml')
        os.makedirs(os.path.dirname(mml_path), exist_ok=True)
        
        if convert_file(fur_file, mml_path):
            success += 1
        else:
            # Check if file exists and is non-empty
            if os.path.exists(mml_path) and os.path.getsize(mml_path) > 0:
                skipped += 1
            else:
                failed += 1
    
    print(f"\nDone: {success} converted, {skipped} skipped (unsupported), {failed} failed")


def main():
    parser = argparse.ArgumentParser(description='Convert Furnace .fur files to MML')
    parser.add_argument('input', nargs='?', help='Input .fur file or directory')
    parser.add_argument('output', nargs='?', help='Output .mml file or directory')
    parser.add_argument('--batch', action='store_true', help='Batch convert directory')
    parser.add_argument('--force', action='store_true', help='Force conversion even for unsupported systems')
    
    args = parser.parse_args()
    
    if not args.input:
        parser.print_help()
        return
    
    if args.batch or os.path.isdir(args.input):
        batch_convert(args.input, args.output)
    else:
        if not args.output:
            args.output = os.path.splitext(args.input)[0] + '.mml'
        convert_file(args.input, args.output)


if __name__ == '__main__':
    main()
