#!/usr/bin/env python3
"""
Convert Furnace OPL3 FUI instrument files to MML tone definitions.

This script converts Furnace .fui instrument files (OPL3/YMF262) to
mml2vgm-compatible MML tone definitions that can be copy-pasted into .gwi song files.

Usage:
  python3 fui2mml.py [FUI_FILE_or_DIRECTORY] [--output OUTPUT_FILE] [--chip CHIP]
  
  If FUI_FILE_or_DIRECTORY is a .fui file, converts that single file.
  If it's a directory, converts all .fui files in it (recursively).
  
  --output: Write output to this file instead of stdout
  --chip: Chip letter for MML instrument (default: P for YMF262 OPL3)
  --dry-run: Just show what would be done
  
Examples:
  # Convert single file to stdout
  python3 fui2mml.py "docs/OPL3 Patch Pack/Basses/Bass 1.fui"
  
  # Convert single file to text file
  python3 fui2mml.py "docs/OPL3 Patch Pack/Basses/Bass 1.fui" -o bass_1.txt
  
  # Convert entire OPL3 Patch Pack directory
  python3 fui2mml.py "docs/OPL3 Patch Pack" --output opl3_instruments.txt
  
  # Convert with different chip letter (e.g., for YM2608)
  python3 fui2mml.py "docs/OPL3 Patch Pack" -o opl3.txt -c P

Output format (mml2vgm MML syntax):
  '; <instrument name>
  '@ P <number>    (chip letter P for YMF262/OPL3)
     AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
  '@ <op1_ar>,<op1_dr>,<op1_sr>,<op1_rr>,<op1_sl>,<op1_tl>,<op1_ks>,<op1_ml>,<op1_dt>,<op1_am>,<op1_ssg_eg>
  '@ <op2_ar>,<op2_dr>,... (up to 4 operators)
  '@ <op3_ar>,...
  '@ <op4_ar>,...
     AL  FB
  '@ <algorithm>,<feedback>
"""

import os
import sys
import argparse
import struct


def parse_fui_file(filepath):
    """
    Parse a Furnace FUI file and extract OPL3 FM instrument data.
    Returns a dict with instrument metadata and FM parameters.
    
    FUI file format (based on Furnace source code):
    - New format: "FINS" (4 bytes) + version (2 bytes) + type (2 bytes) + features
    - Old format: "-Furnace instr.-" (16 bytes) + length (4 bytes) + version (2 bytes) + type (1 byte)
    
    Features are in format: <code:2> <length:2> <data:length>
      - "NA" feature: name (null-terminated string)
      - "FM" feature: FM instrument data
      - "EN" feature: end of instrument marker
    """
    with open(filepath, 'rb') as f:
        data = f.read()
    
    if len(data) < 4:
        return None
    
    result = {
        'name': '',
        'type': None,
        'version': 0,
        'fm_data': None,
        'filepath': filepath,
        'format': 'unknown'
    }
    
    # Check for new format: starts with "FINS"
    if data[:4] == b'FINS':
        result['format'] = 'new'
        pos = 4
        
        if pos + 2 <= len(data):
            result['version'] = struct.unpack('<H', data[pos:pos+2])[0]
            pos += 2
        
        if pos + 2 <= len(data):
            result['type'] = struct.unpack('<H', data[pos:pos+2])[0]
            pos += 2
        
        # For standalone .fui files, features start immediately after type
        end_pos = len(data)
        
        while pos + 4 <= end_pos:
            feat_code = data[pos:pos+2]
            if pos + 4 > end_pos:
                break
            feat_len = struct.unpack('<H', data[pos+2:pos+4])[0]
            
            if pos + 4 + feat_len > end_pos:
                break
            
            feat_data = data[pos+4:pos+4+feat_len]
            
            if feat_code == b'NA':
                name = feat_data.decode('utf-8', errors='replace')
                name = name.replace('\x00', '').strip()
                result['name'] = name if name else ''
            elif feat_code == b'FM':
                result['fm_data'] = feat_data
            
            pos += 4 + feat_len
            
            if feat_code == b'EN':
                break
        
        return result
    
    # Check for old format: starts with "-Furnace instr.-"
    elif data[:16] == b'-Furnace instr.-':
        result['format'] = 'old'
        pos = 16
        
        if pos + 4 > len(data):
            return None
        data_len = struct.unpack('<I', data[pos:pos+4])[0]
        pos += 4
        
        if pos + 2 > len(data):
            return None
        result['version'] = struct.unpack('<H', data[pos:pos+2])[0]
        pos += 2
        
        if pos + 1 > len(data):
            return None
        result['type'] = data[pos]
        pos += 1
        
        if pos + 1 > len(data):
            return None
        pos += 1  # Skip reserved byte
        
        # Get name
        name_end = data.find(b'\x00', pos)
        if name_end != -1:
            name = data[pos:name_end].decode('utf-8', errors='replace')
            result['name'] = name.strip() if name else ''
        
        # Old format: FM data follows the name
        # We'd need to parse the specific layout, but for now just get basic info
        return result
    
    return None


def parse_fm_data(fm_data, version=219):
    """
    Parse FM data bytes into structured parameters.
    Based on Furnace source code (readFeatureFM function in instrument.cpp).
    
    FM data format:
    - Byte 0: opCount (bits 0-3) + enable flags (bits 4-7 for ops 0-3)
    - Byte 1: algorithm (bits 4-6) + feedback (bits 0-2)
    - Byte 2: fms2 (bits 5-7) + ams (bits 3-4) + fms (bits 0-2)
    - Byte 3: ams2 (bits 6-7) + 4op flag (bit 5) + opllPreset (bits 0-4)
    - Byte 4: block (bits 0-3) [if version >= 224]
    - Then operator data for each operator (8 bytes per operator)
    
    Operator data (8 bytes per operator):
    - Byte 0: ksr (bit 7) + dt (bits 4-6) + mult (bits 0-3)
    - Byte 1: sus (bit 7) + tl (bits 0-6)
    - Byte 2: rs (bits 6-7) + vib (bit 5) + ar (bits 0-4)
    - Byte 3: am (bit 7) + ksl (bits 5-6) + dr (bits 0-4)
    - Byte 4: egt (bit 7) + kvs (bits 5-6) + sr (bits 0-4)
    - Byte 5: sl (bits 4-7) + rr (bits 0-3)
    - Byte 6: dvb (bits 4-7) + ssgEnv (bits 0-3)
    - Byte 7: dam (bits 5-7) + dt2 (bits 3-4) + ws (bits 0-2)
    
    Returns dict with:
    - op_count: number of operators (2 or 4)
    - alg: algorithm (0-7)
    - fb: feedback (0-7)
    - ops: list of operator parameter dicts
    """
    if fm_data is None or len(fm_data) < 4:
        return None
    
    # Byte 0: opCount (bits 0-3) + enable flags (bits 4-7 for ops 0-3)
    op_count = fm_data[0] & 0x0F
    
    # Byte 1: algorithm (bits 4-6) + feedback (bits 0-2)
    alg_fb = fm_data[1]
    alg = (alg_fb >> 4) & 0x07
    fb = alg_fb & 0x07
    
    # Byte 2: fms2 + ams + fms
    fms_byte = fm_data[2]
    fms2 = (fms_byte >> 5) & 0x07
    ams = (fms_byte >> 3) & 0x03
    fms = fms_byte & 0x07
    
    # Byte 3: ams2 + 4op flag + opllPreset
    byte3 = fm_data[3]
    ams2 = (byte3 >> 6) & 0x03
    is_4op = bool(byte3 & 0x20)
    opll_preset = byte3 & 0x1F
    
    # Byte 4: block (only if version >= 224)
    offset = 4
    if version >= 224:
        block = fm_data[4] & 0x0F
        offset = 5
    else:
        block = 0
    
    # Parse operator data
    ops = []
    for i in range(op_count):
        if offset + 8 > len(fm_data):
            break
        
        op_bytes = fm_data[offset:offset+8]
        offset += 8
        
        b0 = op_bytes[0]
        b1 = op_bytes[1]
        b2 = op_bytes[2]
        b3 = op_bytes[3]
        b4 = op_bytes[4]
        b5 = op_bytes[5]
        b6 = op_bytes[6]
        b7 = op_bytes[7]
        
        ops.append({
            'ar': b2 & 0x1F,
            'dr': b3 & 0x1F,
            'sr': b4 & 0x1F,
            'rr': b5 & 0x0F,
            'sl': (b5 >> 4) & 0x0F,
            'tl': b1 & 0x7F,
            'ks': (b3 >> 5) & 0x03,
            'ml': b0 & 0x0F,
            'dt': (b0 >> 4) & 0x07,
            'am': 1 if (b3 & 0x80) else 0,
            'ssg_eg': b6 & 0x0F,
        })
    
    return {
        'op_count': op_count,
        'alg': alg,
        'fb': fb,
        'ops': ops
    }


def fm_to_mml(fm_info, name, instrument_num=0, chip_letter='P'):
    """
    Convert parsed FM data to MML tone definition.
    
    MML format for OPL3 (YMF262):
    '@ P 000       - instrument declaration
       AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG
    '@ 031,000,... - operator 1 parameters
    '@ ...         - operator 2-4 parameters
       AL  FB      - algorithm/feedback header
    '@ 004,000    - algorithm,feedback values
    """
    ops = fm_info['ops']
    
    lines = []
    lines.append(f"'; {name}")
    lines.append(f"'@ {chip_letter} {instrument_num:03d}")
    lines.append("   AR  DR  SR  RR  SL  TL  KS  ML  DT  AM  SSG-EG")
    
    # Operator data (4 lines, even if only 2 operators are used)
    for i in range(4):
        if i < len(ops):
            op = ops[i]
            params = [
                f"{op['ar']:03d}",
                f"{op['dr']:03d}",
                f"{op['sr']:03d}",
                f"{op['rr']:03d}",
                f"{op['sl']:03d}",
                f"{op['tl']:03d}",
                f"{op['ks']:03d}",
                f"{op['ml']:03d}",
                f"{op['dt']:03d}",
                f"{op['am']:03d}",
                f"{op['ssg_eg']:03d}",
            ]
            lines.append(f"'@ {','.join(params)}")
        else:
            # Empty operator (all zeros)
            lines.append("'@ 000,000,000,000,000,000,000,000,000,000,000")
    
    # Algorithm and feedback
    lines.append("   AL  FB")
    lines.append(f"'@ {fm_info['alg']:03d},{fm_info['fb']:03d}")
    
    return lines


def get_chip_from_path(filepath):
    """Try to determine chip type from the file path."""
    parts = filepath.replace('\\', '/').split('/')
    for part in parts:
        part_upper = part.upper()
        if part_upper == 'OPL3 PATCH PACK' or part_upper == 'OPL':
            return 'P'  # YMF262 / OPL3
        elif part_upper in ['OPN', 'OPM', 'FM']:
            return 'M'  # YM2203/YM2608/YM2151
        elif part_upper == 'OPLL':
            return 'L'  # YM2413
        elif part_upper == 'OPZ':
            return 'P'  # OPL2
        elif part_upper in ['NES']:
            return 'N'  # NES APU
        elif part_upper in ['GB', 'DMG']:
            return 'G'  # Game Boy
        elif part_upper in ['AY']:
            return 'A'  # AY8910
        elif part_upper in ['SN7']:
            return 'S'  # SN76489
        elif part_upper in ['SCC', 'K051649']:
            return 'K'  # K051649
        elif part_upper in ['VRC6']:
            return 'V'  # VRC6
    return None


def is_fm_chip(chip_letter):
    """Check if a chip letter represents an FM chip."""
    return chip_letter in ['P', 'M', 'L']  # OPL3, OPN/OPM, OPLL


def convert_fui_to_mml(filepath, instrument_num=0, chip_letter=None):
    """
    Convert a single FUI file to MML tone definition.
    Returns list of MML lines.
    """
    fui_data = parse_fui_file(filepath)
    if fui_data is None:
        return []
    
    name = fui_data.get('name', '')
    if not name:
        name = os.path.basename(filepath).replace('.fui', '').replace('.FUI', '')
    
    # Auto-detect chip letter from path if not specified
    if chip_letter is None:
        chip_letter = 'P'  # Default to OPL3
        path_chip = get_chip_from_path(filepath)
        if path_chip:
            chip_letter = path_chip
    
    # Try to parse FM data
    fm_data = fui_data.get('fm_data')
    if fm_data is not None:
        fm_info = parse_fm_data(fm_data, fui_data.get('version', 219))
        if fm_info:
            return fm_to_mml(fm_info, name, instrument_num, chip_letter)
    
    # Fallback: for FM chips without data, output FM template
    if is_fm_chip(chip_letter):
        return fm_to_mml({
            'op_count': 4,
            'alg': 0,
            'fb': 0,
            'ops': [{'ar':0,'dr':0,'sr':0,'rr':0,'sl':0,'tl':60,'ks':0,'ml':1,'dt':0,'am':0,'ssg_eg':0}] * 4
        }, name, instrument_num, chip_letter)
    
    # For non-FM chips (PSG, etc.), output simple tone definition
    lines = []
    lines.append(f"'; {name}")
    lines.append(f"'@ {chip_letter} {instrument_num:03d}")
    return lines


def process_directory(directory, output_file=None, chip_letter=None):
    """
    Process all FUI files in a directory and its subdirectories.
    """
    fui_files = []
    for root, dirs, files in os.walk(directory):
        for f in files:
            if f.endswith('.fui') or f.endswith('.FUI'):
                fui_files.append(os.path.join(root, f))
    
    fui_files.sort()
    
    all_lines = []
    
    for i, fpath in enumerate(fui_files):
        lines = convert_fui_to_mml(fpath, i, chip_letter)
        if lines:
            all_lines.extend(lines)
            all_lines.append('')  # Blank line between instruments
    
    output = '\n'.join(all_lines)
    
    if output_file:
        with open(output_file, 'w') as f:
            f.write(output)
        print(f"Wrote {len(fui_files)} instruments to {output_file}")
    else:
        print(output)
    
    return len(fui_files)


def process_file(filepath, output_file=None, chip_letter=None):
    """
    Process a single FUI file.
    """
    lines = convert_fui_to_mml(filepath, 0, chip_letter)
    if not lines:
        print(f"Error: Could not parse {filepath}", file=sys.stderr)
        return 0
    
    output = '\n'.join(lines)
    
    if output_file:
        with open(output_file, 'w') as f:
            f.write(output)
        print(f"Wrote instrument to {output_file}")
    else:
        print(output)
    
    return 1


def main():
    parser = argparse.ArgumentParser(
        description='Convert Furnace OPL3 FUI instrument files to MML tone definitions.'
    )
    parser.add_argument(
        'input',
        nargs='?',
        default='docs/OPL3 Patch Pack',
        help='FUI file or directory containing FUI files (default: docs/OPL3 Patch Pack)'
    )
    parser.add_argument(
        '-o', '--output',
        help='Output file (default: stdout)'
    )
    parser.add_argument(
        '-c', '--chip',
        default=None,
        help='Chip letter for MML instrument (default: auto-detect from directory). Use P for OPL3, M for OPN/YM2612, etc.'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Just show what would be done'
    )
    
    args = parser.parse_args()
    input_path = args.input
    
    if args.dry_run:
        if os.path.isfile(input_path) and (input_path.endswith('.fui') or input_path.endswith('.FUI')):
            print(f"Would process: {input_path}")
        elif os.path.isdir(input_path):
            fui_files = []
            for root, dirs, files in os.walk(input_path):
                for f in files:
                    if f.endswith('.fui') or f.endswith('.FUI'):
                        fui_files.append(os.path.join(root, f))
            print(f"Would process {len(fui_files)} FUI files in {input_path}")
        else:
            print(f"Invalid input: {input_path}")
        return
    
    if os.path.isfile(input_path) and (input_path.endswith('.fui') or input_path.endswith('.FUI')):
        process_file(input_path, args.output, args.chip)
    elif os.path.isdir(input_path):
        process_directory(input_path, args.output, args.chip)
    else:
        print(f"Error: {input_path} is not a valid FUI file or directory", file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main()
