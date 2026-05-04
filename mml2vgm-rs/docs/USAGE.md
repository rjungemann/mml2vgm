# mml2vgm-rs Usage Guide

This guide provides example commands for using the mml2vgm-rs command-line tool.

## Help and Information Commands

Get help about the tool and its options:

```bash
# Show full help
./target/debug/mml2vgm-rs --help

# Show version information
./target/debug/mml2vgm-rs --version

# List all supported sound chips
./target/debug/mml2vgm-rs --list-chips

# List all supported output formats
./target/debug/mml2vgm-rs --list-formats
```

## Basic Compilation

Compile MML files to game music formats:

```bash
# Compile to VGM format (default)
./target/debug/mml2vgm-rs /tmp/test.mml

# Compile with verbose output
./target/debug/mml2vgm-rs /tmp/test.mml -v

# Compile with debug output (very verbose)
./target/debug/mml2vgm-rs /tmp/test.mml --debug
```

## Validation and Checking

Validate MML files without generating output:

```bash
# Validate only (no compilation)
./target/debug/mml2vgm-rs /tmp/test.mml --check

# Validate with verbose output
./target/debug/mml2vgm-rs /tmp/test.mml --check -v
```

## Output Format Selection

Compile to different game music formats:

```bash
# Compile to VGM format (default)
./target/debug/mml2vgm-rs /tmp/test.mml -f vgm

# Compile to XGM format (Mega Drive)
./target/debug/mml2vgm-rs /tmp/test.mml -f xgm

# Compile to XGM2 format (Mega Drive extended)
./target/debug/mml2vgm-rs /tmp/test.mml -f xgm2

# Compile to ZGM format (Extended VGM)
./target/debug/mml2vgm-rs /tmp/test.mml -f zgm
```

## Custom Output Path

Specify where to save the compiled file:

```bash
# Compile with custom output filename
./target/debug/mml2vgm-rs /tmp/test.mml -o /tmp/my_song.vgm

# Compile with custom format and output path
./target/debug/mml2vgm-rs /tmp/test.mml -f xgm -o /tmp/my_song.xgm

# Compile to a different directory
./target/debug/mml2vgm-rs /tmp/test.mml -o ~/music/output.vgm
```

## Creating Test Files

Create sample MML files to test compilation:

```bash
# Simple test file
cat > /tmp/test.mml << 'EOF'
{ Title=Test Song }
'F o4 c4 d4 e4 f4
EOF

# More complex test file with multiple parts
cat > /tmp/complex.mml << 'EOF'
{ Title=Complex Song }
'F o4 c4 d4 e4 f4 g4 a4 b4 >c4
'S o3 c2 c2 d2 d2
'B o2 c1 c1 d1 d1
EOF
```

## Testing Complex Files

Compile and validate more complex MML files:

```bash
# Compile the complex file with verbose output
./target/debug/mml2vgm-rs /tmp/complex.mml -v

# Validate the complex file without compiling
./target/debug/mml2vgm-rs /tmp/complex.mml --check -v

# Compile to multiple formats
./target/debug/mml2vgm-rs /tmp/complex.mml -f vgm -o /tmp/complex_vgm.vgm
./target/debug/mml2vgm-rs /tmp/complex.mml -f xgm -o /tmp/complex_xgm.xgm
./target/debug/mml2vgm-rs /tmp/complex.mml -f zgm -o /tmp/complex_zgm.zgm
```

## Error Handling

Test error handling and edge cases:

```bash
# Try to compile a non-existent file
./target/debug/mml2vgm-rs /tmp/nonexistent.mml 2>&1

# Validate a non-existent file
./target/debug/mml2vgm-rs /tmp/nonexistent.mml --check 2>&1
```

## Verifying Output

Check the generated files:

```bash
# List all generated output files
ls -lh /tmp/*.vgm /tmp/*.xgm /tmp/*.zgm 2>/dev/null

# Check file information
file /tmp/test.vgm /tmp/test.xgm 2>/dev/null

# Compare file sizes across formats
echo "VGM:" && wc -c < /tmp/test_vgm.vgm
echo "XGM:" && wc -c < /tmp/test_xgm.xgm
echo "XGM2:" && wc -c < /tmp/test_xgm2.xgm
echo "ZGM:" && wc -c < /tmp/test_zgm.zgm

# Display hexdump of generated VGM file
hexdump -C /tmp/test.vgm | head -20
```

## Batch Processing

Process multiple files:

```bash
# Compile all MML files in a directory to VGM
for file in /tmp/*.mml; do
    ./target/debug/mml2vgm-rs "$file" -f vgm
done

# Compile all MML files to multiple formats
for file in /tmp/*.mml; do
    base=$(basename "$file" .mml)
    ./target/debug/mml2vgm-rs "$file" -f vgm -o "/tmp/${base}.vgm"
    ./target/debug/mml2vgm-rs "$file" -f xgm -o "/tmp/${base}.xgm"
    ./target/debug/mml2vgm-rs "$file" -f zgm -o "/tmp/${base}.zgm"
done

# List all compiled outputs
ls -1 /tmp/*.vgm /tmp/*.xgm /tmp/*.zgm 2>/dev/null | sort
```

## Advanced Options

Use advanced compilation options:

```bash
# Compile with include paths (for MML includes)
./target/debug/mml2vgm-rs /tmp/test.mml -I /tmp/includes -I ~/.mml_includes

# Compile with custom clock count
./target/debug/mml2vgm-rs /tmp/test.mml --clock-count 192

# Specify target sound chips (if supported)
./target/debug/mml2vgm-rs /tmp/test.mml -c YM2612 -c SN76489

# Output trace information (for debugging)
./target/debug/mml2vgm-rs /tmp/test.mml --trace -o /tmp/test.vgm

# Combine multiple options
./target/debug/mml2vgm-rs /tmp/test.mml \
  -f xgm \
  -o /tmp/output.xgm \
  -v \
  --clock-count 192 \
  -I ~/.mml_includes
```

## Release Build

Build an optimized release version:

```bash
# Build release binary
cargo build --release

# Run the release binary (faster)
./target/release/mml2vgm-rs /tmp/test.mml -v

# Benchmark compilation
time ./target/release/mml2vgm-rs /tmp/complex.mml -f vgm -o /tmp/bench.vgm
```

## Troubleshooting

Debug compilation issues:

```bash
# Compile with debug output to see detailed logs
./target/debug/mml2vgm-rs /tmp/test.mml --debug

# Validate before compiling to catch syntax errors
./target/debug/mml2vgm-rs /tmp/test.mml --check -v

# Check if file exists and is readable
test -r /tmp/test.mml && echo "File is readable" || echo "File not found or not readable"

# Verify output file was created
test -f /tmp/test.vgm && echo "Output created" || echo "Output not created"
```
