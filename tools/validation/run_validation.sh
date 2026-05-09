#!/bin/bash
# Golden Master Validation Runner
# Executes end-to-end validation: MML → VGM → WAV → Spectral Analysis
#
# Usage:
#   ./run_validation.sh [chip_name] [test_file]
#   ./run_validation.sh ym2151 test_ym2151_envelope.gwi
#
# Environment variables:
#   MEDNAFEN_CMD: Path to mednafen binary (default: /opt/homebrew/bin/mednafen)
#   MAME_CMD: Path to mame binary (default: /opt/homebrew/bin/mame)
#   DOSBOX_X_CMD: Path to dosbox-x binary (default: /opt/homebrew/bin/dosbox-x)
#   VGM2PCM_CMD: Path to vgm2pcm binary (required for WAV conversion)
#   MML2VGM_CMD: Path to mml2vgm-rs binary (default: cargo)

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TESTS_DIR="$PROJECT_ROOT/tests/golden_master"
RESULTS_DIR="$PROJECT_ROOT/validation_results"

# Default commands
MEDNAFEN_CMD="${MEDNAFEN_CMD:-/opt/homebrew/bin/mednafen}"
MAME_CMD="${MAME_CMD:-/opt/homebrew/bin/mame}"
DOSBOX_X_CMD="${DOSBOX_X_CMD:-/opt/homebrew/bin/dosbox-x}"
MML2VGM_CMD="${MML2VGM_CMD:-cargo run --release --}"
VGM2PCM_CMD="${VGM2PCM_CMD:-vgm2pcm}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*"
}

log_error() {
    echo -e "${RED}✗${NC} $*"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $*"
}

# Ensure results directory exists
mkdir -p "$RESULTS_DIR"

# Parse arguments
if [[ $# -lt 1 ]]; then
    log_error "Usage: $0 <chip_name> [test_file]"
    log_error "Example: $0 ym2151 test_ym2151_envelope.gwi"
    echo ""
    echo "Available chips:"
    echo "  - ym2151 (Mednafen arcade driver)"
    echo "  - ym2203 (Mednafen PC-88 driver)"
    echo "  - ym2608 (Mednafen PC-98 driver)"
    echo "  - opl (DOSBox-X)"
    echo "  - segapcm (Mednafen Genesis driver)"
    echo "  - nes (Mesen-X)"
    echo "  - qsound (MAME)"
    exit 1
fi

CHIP_NAME="$1"
TEST_FILE="${2:-}"

# Determine which emulator to use
case "$CHIP_NAME" in
    ym2151|ym2203|ym2608|segapcm)
        EMULATOR="mednafen"
        EMULATOR_CMD="$MEDNAFEN_CMD"
        ;;
    opl2|opl3|y8950)
        EMULATOR="dosbox-x"
        EMULATOR_CMD="$DOSBOX_X_CMD"
        ;;
    nes)
        EMULATOR="mesen-x"
        log_warn "Mesen-X support not yet configured"
        exit 1
        ;;
    qsound)
        EMULATOR="mame"
        EMULATOR_CMD="$MAME_CMD"
        ;;
    *)
        log_error "Unknown chip: $CHIP_NAME"
        exit 1
        ;;
esac

# Find test file
if [[ -z "$TEST_FILE" ]]; then
    # Find first test file for this chip
    TEST_FILE=$(find "$TESTS_DIR/tier1" -name "test_${CHIP_NAME}*.gwi" | head -1)
    if [[ -z "$TEST_FILE" ]]; then
        log_error "No test files found for chip: $CHIP_NAME"
        exit 1
    fi
fi

if [[ ! -f "$TEST_FILE" ]]; then
    log_error "Test file not found: $TEST_FILE"
    exit 1
fi

TEST_BASENAME=$(basename "$TEST_FILE" .gwi)
log_info "Starting validation for: $TEST_BASENAME"
log_info "Chip: $CHIP_NAME"
log_info "Emulator: $EMULATOR"

# Step 1: Compile MML to VGM with mml2vgm
log_info "Step 1: Compiling MML to VGM..."
MML2VGM_OUTPUT="$RESULTS_DIR/${TEST_BASENAME}.vgm"

cd "$PROJECT_ROOT/mml2vgm-rs"
if ! $MML2VGM_CMD "$TEST_FILE" -o "$MML2VGM_OUTPUT" 2>&1; then
    log_error "MML compilation failed"
    exit 1
fi
log_success "VGM compilation complete: $MML2VGM_OUTPUT"

# Step 2: Generate golden master (emulator reference)
log_info "Step 2: Generating golden master reference via $EMULATOR..."
GOLDEN_VGM="$RESULTS_DIR/${TEST_BASENAME}_golden.vgm"

case "$EMULATOR" in
    mednafen)
        log_warn "Mednafen golden master generation not yet automated"
        log_info "Manual step: Play test ROM on Mednafen and export VGM"
        log_info "Place output at: $GOLDEN_VGM"
        ;;
    dosbox-x)
        log_warn "DOSBox-X golden master generation not yet automated"
        log_info "Manual step: Run game in DOSBox-X and export audio"
        ;;
    mame)
        log_warn "MAME golden master generation not yet automated"
        log_info "Manual step: Run game in MAME and export audio"
        ;;
esac

# Step 3: Convert VGM to WAV (both golden and mml2vgm)
log_info "Step 3: Converting VGM files to WAV..."

if command -v "$VGM2PCM_CMD" &> /dev/null; then
    MML2VGM_WAV="$RESULTS_DIR/${TEST_BASENAME}.wav"
    if ! "$VGM2PCM_CMD" "$MML2VGM_OUTPUT" "$MML2VGM_WAV"; then
        log_warn "VGM to WAV conversion failed (is vgm2pcm installed?)"
    else
        log_success "mml2vgm WAV: $MML2VGM_WAV"
    fi

    if [[ -f "$GOLDEN_VGM" ]]; then
        GOLDEN_WAV="$RESULTS_DIR/${TEST_BASENAME}_golden.wav"
        if ! "$VGM2PCM_CMD" "$GOLDEN_VGM" "$GOLDEN_WAV"; then
            log_warn "Golden master VGM to WAV conversion failed"
        else
            log_success "Golden master WAV: $GOLDEN_WAV"
        fi
    fi
else
    log_warn "vgm2pcm not found in PATH"
    log_info "Install: Download from https://www.smspower.org/forums/15417-VGMToolsuite"
fi

# Step 4: Run spectral analysis comparison
log_info "Step 4: Running spectral analysis comparison..."

if [[ -f "$MML2VGM_WAV" && -f "$GOLDEN_WAV" ]]; then
    PLOT_OUTPUT="$RESULTS_DIR/${TEST_BASENAME}_comparison.png"
    if python3 "$SCRIPT_DIR/spectral_analysis.py" "$GOLDEN_WAV" "$MML2VGM_WAV" \
        --threshold 0.95 --plot "$PLOT_OUTPUT" 2>&1 | tee "$RESULTS_DIR/${TEST_BASENAME}_spectral.log"; then
        log_success "Spectral analysis complete"
        log_success "Plot saved to: $PLOT_OUTPUT"
    else
        log_warn "Spectral analysis encountered issues (see log above)"
    fi
else
    log_warn "WAV files not available for spectral analysis"
fi

# Step 5: Run VGM binary comparison
log_info "Step 5: Running VGM binary comparison..."

if [[ -f "$GOLDEN_VGM" ]]; then
    if python3 "$SCRIPT_DIR/vgm_compare.py" "$GOLDEN_VGM" "$MML2VGM_OUTPUT" 2>&1 \
        | tee "$RESULTS_DIR/${TEST_BASENAME}_vgm_compare.log"; then
        log_success "VGM comparison complete"
    else
        log_warn "VGM comparison encountered issues"
    fi
else
    log_warn "Golden master VGM not available for comparison"
fi

# Summary
log_success "Validation complete for: $TEST_BASENAME"
log_info "Results directory: $RESULTS_DIR"
echo ""
echo "Generated files:"
echo "  - $MML2VGM_OUTPUT (mml2vgm VGM)"
echo "  - $MML2VGM_WAV (mml2vgm WAV, if vgm2pcm available)"
echo "  - $PLOT_OUTPUT (spectral analysis plot, if golden master available)"
echo "  - $RESULTS_DIR/${TEST_BASENAME}_*.log (comparison logs)"
