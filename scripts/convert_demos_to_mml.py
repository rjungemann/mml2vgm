#!/usr/bin/env python3
"""
Convert Furnace demo tracks to MML stubs based on directory structure.

This script creates MML stub files for Furnace demo tracks organized by system.
It maps directory names to mml2vgm system identifiers and creates basic MML files.

Usage:
    python convert_demos_to_mml.py <demos_directory> <output_directory>
"""

import os
import sys
import argparse

# Mapping of Furnace demo directory names to mml2vgm system identifiers
# Based on mml2vgm-rs/src/lib.rs SoundChip enum
DIRECTORY_TO_SYSTEMS = {
    # Supported systems
    'ay8910': ['$AY8910'],
    'ay8930': ['$AY8930'],
    'gameboy': ['$GB'],
    'genesis': ['$FM1'],
    'msx': ['$AY8910'],  # MSX typically uses AY-3-8910
    'nes': ['$NES'],
    'opl': ['$OPL2', '$OPL3'],  # OPL directory could have OPL2 or OPL3
    'opm': ['$OPM'],
    'pc98': ['$OPNA'],
    'pce': ['$HUC'],
    'sn7': ['$PSG'],
    
    # Additional mappings
    'arcade': ['$OPM', '$OPL3', '$C140'],  # Arcade could have various
    'c64': ['$SID'],  # Not sure if supported
    'snes': ['$SNES'],  # Check if supported
    
    # Multichip - use common systems
    'multichip': ['$FM1', '$PSG'],
}

# Systems supported by mml2vgm (from SoundChip enum in lib.rs)
SUPPORTED_MML_SYSTEMS = {
    '$PSG', '$FM1', '$FM2', '$OPM', '$OPN', '$OPNA', '$OPNB', '$OPNB2',
    '$OPL', '$OPL2', '$OPL3', '$OPL4', '$OPLL', '$OPZ',
    '$HUC', '$C140', '$C352', '$AY8910', '$AY8930', '$GB',
    '$NES', '$FDS', '$VRC6', '$VRC7', '$MMC5', '$S5B', '$N163',
    '$POKEY', '$QSOUND', '$YMZ280B', '$RF5C164', '$SEGAPCM'
}


def get_systems_for_directory(dirname):
    """Get MML system identifiers for a directory name."""
    dirname_lower = dirname.lower()
    return DIRECTORY_TO_SYSTEMS.get(dirname_lower, [])


def is_directory_supported(dirname):
    """Check if a directory contains supported systems."""
    systems = get_systems_for_directory(dirname)
    return any(s in SUPPORTED_MML_SYSTEMS for s in systems)


def create_mml_stub(fur_path, output_path, systems):
    """Create a basic MML stub file for a .fur track."""
    basename = os.path.splitext(os.path.basename(fur_path))[0]
    
    lines = [
        f"; {basename}",
        f"; Converted from Furnace .fur file",
        "",
    ]
    
    # Add system definitions
    if systems:
        lines.append(", ".join(systems) + ",")
        lines.append("")
    
    # Add ALL directive
    lines.append("!ALL")
    lines.append("")
    
    # Add stub pattern
    lines.append("$$ALL")
    lines.append("  ; TODO: Add pattern data from Furnace")
    lines.append("")
    
    mml_content = "\n".join(lines)
    
    with open(output_path, 'w') as f:
        f.write(mml_content)
    
    return len(mml_content)


def convert_directory(input_dir, output_dir):
    """Convert all .fur files in a directory structure to MML."""
    os.makedirs(output_dir, exist_ok=True)
    
    converted = 0
    skipped = 0
    
    for root, dirs, files in os.walk(input_dir):
        # Get relative path
        rel_root = os.path.relpath(root, input_dir)
        out_root = os.path.join(output_dir, rel_root)
        os.makedirs(out_root, exist_ok=True)
        
        # Get systems for this directory
        dirname = os.path.basename(root)
        systems = get_systems_for_directory(dirname)
        
        # Only process if directory has supported systems
        if not systems:
            # Check parent directories
            parent_rel = os.path.dirname(rel_root)
            while parent_rel and parent_rel != '.':
                parent_systems = get_systems_for_directory(os.path.basename(parent_rel))
                if parent_systems:
                    systems = parent_systems
                    break
                parent_rel = os.path.dirname(parent_rel)
        
        if not systems:
            # Use generic systems based on file content (would need parsing)
            # For now, skip
            continue
        
        for f in files:
            if not f.lower().endswith('.fur'):
                continue
            
            fur_path = os.path.join(root, f)
            mml_basename = os.path.splitext(f)[0] + '.mml'
            mml_path = os.path.join(out_root, mml_basename)
            
            try:
                size = create_mml_stub(fur_path, mml_path, systems)
                print(f"  Created: {rel_root}/{mml_basename} ({size} bytes)")
                converted += 1
            except Exception as e:
                print(f"  ERROR: {fur_path} - {e}")
                skipped += 1
    
    print(f"\nDone: {converted} converted, {skipped} skipped")


def main():
    parser = argparse.ArgumentParser(description='Convert Furnace demo tracks to MML stubs')
    parser.add_argument('input_dir', help='Input directory containing .fur files')
    parser.add_argument('output_dir', nargs='?', default='mml_output', 
                        help='Output directory for MML files')
    
    args = parser.parse_args()
    
    if not os.path.isdir(args.input_dir):
        print(f"Error: {args.input_dir} is not a directory")
        sys.exit(1)
    
    print(f"Converting Furnace demos from {args.input_dir}...")
    print(f"Output: {args.output_dir}")
    
    convert_directory(args.input_dir, args.output_dir)


if __name__ == '__main__':
    main()
