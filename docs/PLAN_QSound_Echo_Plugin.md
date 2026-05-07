# Plan: QSound Echo — VST3/CLAP Effect Plugin

## Overview

A stereo echo/reverb plugin that implements the QSound DL-1425 delay algorithm: a circular buffer with a per-voice send mix, a global feedback coefficient, and a one-pole moving-average low-pass in the feedback loop. The combination produces the characteristic warm, mono-summed echo of CPS1/CPS2 arcade hardware.

This document assesses viability and provides a complete implementation plan following the architecture of the existing sibling projects [combover-vst3](../../combover-vst3) (effect) and [chiaro](../../chiaro) (instrument).

---

## Viability Assessment

### What makes QSound echo distinctive

The QSound echo algorithm differs from a conventional stereo delay in four ways that produce a recognisable sound:

1. **Mono echo send.** Both input channels are summed before entering the delay buffer, so the echo tail is always mono even when the dry signal is stereo. This is a classic "vintage spring" characteristic.
2. **Moving-average damping in the feedback loop.** Each recirculation pass is blended with the previous sample `(echo + last) × 0.5`, darkening high frequencies with every bounce. The tail accumulates warmth.
3. **×4 feedback gain scaling.** The hardware multiplies the filtered echo by 4× before adding input, so a feedback coefficient of 0.25 equals unity. Above 0.25 the tail grows. This creates a tight, controlled danger zone.
4. **Native rate granularity.** Delay time is quantised to steps of ~41.6 µs (1 / 24038 Hz), creating unusual non-musical delay lengths that feel "off-grid" in a pleasing way.

### Market position

There is no commercial plugin that specifically emulates the QSound delay topology. Analogous vintage effects (RE-201 tape echo, spring reverb) are crowded but QSound is undocumented territory. Game music producers and Capcom aesthetics fans are a real niche. The authentic parameter range (57–170 ms) overlaps with slapback and short room reverb — both useful creative tools.

### Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Algorithm too simple to justify a standalone plugin | Medium | Add authentic and extended modes; expose moving-average order as a parameter |
| Authentic delay range (57–170 ms) too narrow for general use | Low | Allow 0–2000 ms range; "QSound Mode" toggle restricts to authentic range |
| Mono echo feel limiting | Low | Add a stereo width control that widens the echo output via a small L/R detune |
| Feedback clips without warning | Medium | Show a soft-clip indicator in the UI; clamp the internal buffer before blowup |

### Verdict

**Viable.** The algorithm is concise enough to implement correctly in a few days, the sound is genuinely distinctive, and the combover-vst3 codebase provides a nearly drop-in template. The main creative investment is in the UI and parameter tuning.

---

## DSP Algorithm

### Core loop (per stereo sample pair)

```cpp
// Inputs: in_l, in_r  →  Outputs: out_l, out_r
// State: buffer[BUF_SIZE*2], write_pos, last_filtered, delay_len

// 1. Read from delay line
size_t read_pos = (write_pos + BUF_SIZE * 2 - delay_len * 2) % (BUF_SIZE * 2);
float echo_l = buffer[read_pos];
float echo_r = buffer[read_pos + 1];

// 2. Moving-average low-pass on the mono-summed echo
float mono_echo = (echo_l + echo_r) * 0.5f;
float filtered  = (mono_echo + last_filtered) * damping;   // damping=0.5 is authentic
last_filtered   = mono_echo;

// 3. Recirculate: clamp to prevent blowup
float feedback_signal = std::clamp(filtered * feedback * 4.0f, -1.0f, 1.0f);

// 4. Write sum of input send + feedback into delay line
float write_sample = (in_l + in_r) * 0.5f * send + feedback_signal;
buffer[write_pos]     = write_sample;   // mono echo stored as equal L/R
buffer[write_pos + 1] = write_sample;
write_pos = (write_pos + 2) % (BUF_SIZE * 2);

// 5. Mix dry + wet
out_l = in_l + echo_l * wet;
out_r = in_r + echo_r * wet;
```

The authentic QSound uses `damping = 0.5` (fixed). The plugin exposes this as a knob (0.0 = no damping, 1.0 = fully damped). Setting `damping` to 0.0 bypasses the low-pass entirely and gives a brighter, flatter tail.

### Delay buffer sizing

| Parameter | Authentic QSound | Plugin range |
|---|---|---|
| Sample rate for delay | 24038 Hz | 24038 Hz (fixed) |
| `delay_len` register range | 0x055A–0x0FFF (1370–4095 samples) | 0–4096 samples |
| Delay in ms (authentic) | 57–170 ms | 0–2000 ms (extended) |
| Buffer size (extended) | — | 48077 stereo frames (2000 ms × 24038) |

When the host sample rate differs from 24038 Hz the buffer is addressed in native-rate samples and resampled on the way in/out, preserving the characteristic granularity.

### Stereo width extension (non-authentic)

An optional stereo spread parameter widens the echo output by splitting the mono echo into a Haas-effect pair: right channel reads `detune_samples` positions later than left. This is separate from the authentic algorithm and controlled by a toggle.

---

## Parameters

Following the combover-vst3 parameter model:

```cpp
enum class ParamId : std::uint32_t
{
    DelayTime   = 1000,  // 0–2000 ms; default 105 ms (authentic centre)
    Feedback    = 1001,  // 0–100 %; >25 % = growing tail; default 20 %
    Damping     = 1002,  // 0–100 %; default 50 % (authentic)
    SendLevel   = 1003,  // 0–100 %; input into the delay bus; default 80 %
    WetMix      = 1004,  // 0–100 %; default 40 %
    QSoundMode  = 1005,  // toggle; clamps delay to 57–170 ms when on
    StereoWidth = 1006,  // 0–100 %; 0 = mono echo (authentic); default 0 %
};
```

**Conversion functions (in `parameter_model.h`):**

```cpp
// DelayTime: normalized 0..1 → 0..2000 ms
inline double normalizedToDelayMs(double norm) { return norm * 2000.0; }

// Feedback: normalized 0..1 → 0..1.0 coefficient
// Internally multiplied by 4 in DSP; >0.25 causes growth
inline double normalizedToFeedback(double norm) { return norm; }

// Damping: normalized 0..1 → 0..1 (0.5 authentic)
inline double normalizedToDamping(double norm) { return norm; }

// delay_len in native samples from ms
inline size_t msToDelaySamples(double ms) {
    return static_cast<size_t>((ms / 1000.0) * 24038.0);
}

// QSound mode: clamp delay to authentic range
inline double clampToQSoundRange(double ms) {
    return std::clamp(ms, 57.0, 170.0);
}
```

---

## Project Structure

Mirror combover-vst3 exactly, replacing the comb-filter DSP with the echo engine:

```
qsound-echo-vst3/
├── CMakeLists.txt
├── Justfile
├── src/
│   ├── plugin/
│   │   ├── processor.h/cpp      # AudioEffect subclass; process() template
│   │   ├── controller.h/cpp     # EditControllerEx1; parameter registration
│   │   ├── entry.cpp            # DEF_CLASS2 factory entry
│   │   ├── ids.h                # Plugin/component UIDs
│   │   └── param_ids.h          # ParamId enum + kPitchBendParamId
│   ├── core/
│   │   ├── parameter_model.h    # ParamId, ParameterDescriptor[], conversions
│   │   ├── plugin_state.h       # PluginState (7 normalized doubles)
│   │   ├── echo_engine.h/cpp    # QSound echo DSP (EchoEngine class)
│   │   └── preset_manager.h/cpp # JSON preset load/save
│   ├── ui/
│   │   ├── imgui_editor.h/cpp
│   │   ├── editor_core.h/cpp
│   │   ├── imgui_editor_mac.mm
│   │   ├── imgui_editor_win32.cpp
│   │   └── imgui_editor_linux.cpp
│   ├── standalone/
│   │   ├── runtime.h/cpp
│   │   ├── main_mac.mm
│   │   ├── main_win.cpp
│   │   └── main_linux.cpp
│   └── clap/                    # Optional CLAP wrapper
│       ├── clap_plugin.h/cpp
│       ├── clap_process.h/cpp
│       ├── clap_params.h/cpp
│       └── clap_entry.cpp
├── tests/
│   ├── test_echo_engine.cpp     # Catch2 unit tests
│   └── CMakeLists.txt
└── resources/
    └── presets/
        ├── CPS2 Authentic.json
        ├── Long Hall.json
        └── Phase Invert.json    # Negative feedback preset
```

### EchoEngine class interface

```cpp
class EchoEngine {
public:
    struct Parameters {
        double delay_ms     = 105.0;
        double feedback     = 0.20;
        double damping      = 0.50;
        double send_level   = 0.80;
        double wet_mix      = 0.40;
        bool   qsound_mode  = false;
        double stereo_width = 0.0;
    };

    void reset(double host_sample_rate);
    void setParameters(const Parameters& p);
    void process(float in_l, float in_r, float& out_l, float& out_r);
    bool isClipping() const;   // true if buffer hit ±1.0 in last block

private:
    static constexpr size_t NATIVE_RATE  = 24038;
    static constexpr size_t MAX_DELAY_MS = 2000;
    static constexpr size_t BUF_FRAMES   = NATIVE_RATE * MAX_DELAY_MS / 1000 + 1;

    std::array<float, BUF_FRAMES * 2> buffer_ {};
    size_t  write_pos_    = 0;
    size_t  delay_frames_ = 0;
    float   feedback_     = 0.0f;
    float   damping_      = 0.5f;
    float   send_         = 0.8f;
    float   wet_          = 0.4f;
    float   last_filtered_= 0.0f;
    size_t  haas_offset_  = 0;        // for stereo width
    bool    clipping_     = false;

    double  host_rate_    = 44100.0;

    // Resampling state (host rate → 24038 Hz for buffer indexing)
    double  phase_acc_    = 0.0;
    float   last_in_l_    = 0.0f;
    float   last_in_r_    = 0.0f;
};
```

The buffer is always addressed in 24038 Hz-equivalent frames. When the host rate is 44100 Hz, a linear interpolating downsampler converts input before writing, and upsamples the read position for output. This preserves delay granularity independent of host sample rate.

---

## UI Design

Following the combover-vst3 ImGui pattern, a single-screen editor with:

```
┌─────────────────────────────────────────────────────┐
│  QSound Echo                            [QSound ●]  │
│                                                      │
│  DELAY                                               │
│  ┌──────────┐  0–2000 ms                            │
│  │  105 ms  │  [————●————————]                      │
│  └──────────┘                                        │
│                                                      │
│  FEEDBACK    DAMPING     SEND      WET               │
│  [──●──]     [────●]     [──●──]   [──●──]           │
│   20 %        50 %        80 %      40 %             │
│                                                      │
│  STEREO WIDTH                       [CLIPPING ○]    │
│  [●────────]   0 %                                   │
│                                                      │
│  [CPS2 Authentic ▼]  [Save]  [Reset]                │
└─────────────────────────────────────────────────────┘
```

UI elements of note:
- **Delay knob** is large and central (the defining parameter)
- **QSound Mode** toggle (top-right) visually constrains the delay slider to 57–170 ms and snaps to native-rate steps
- **Clipping indicator** (top-right red dot) lights when the buffer saturated in the previous block — feedback > 25% territory
- **Feedback** colours: green (0–20%), yellow (20–25%), red (>25%) to communicate the stability threshold
- **Preset dropdown** with 3 factory presets: *CPS2 Authentic*, *Long Hall*, *Phase Invert*

---

## Build Setup (CMakeLists.txt sketch)

Identical FetchContent dependencies to combover-vst3:

```cmake
cmake_minimum_required(VERSION 3.25)
project(qsound-echo-vst3)

include(FetchContent)

FetchContent_Declare(vst3sdk
    GIT_REPOSITORY https://github.com/steinbergmedia/vst3sdk.git
    GIT_TAG        master)
FetchContent_Declare(imgui
    GIT_REPOSITORY https://github.com/ocornut/imgui.git
    GIT_TAG        master)
FetchContent_Declare(miniaudio
    GIT_REPOSITORY https://github.com/mackron/miniaudio.git
    GIT_TAG        master)
FetchContent_Declare(nlohmann_json
    GIT_REPOSITORY https://github.com/nlohmann/json.git
    GIT_TAG        v3.11.3)
FetchContent_Declare(Catch2
    GIT_REPOSITORY https://github.com/catchorg/Catch2.git
    GIT_TAG        v3.7.0)
FetchContent_MakeAvailable(vst3sdk imgui miniaudio nlohmann_json Catch2)

# Core DSP library (no VST3 dependency)
add_library(qsound_echo_core STATIC
    src/core/echo_engine.cpp
    src/core/preset_manager.cpp)
target_include_directories(qsound_echo_core PUBLIC src)

# VST3 plugin
smtg_add_vst3plugin(qsound-echo
    src/plugin/processor.cpp
    src/plugin/controller.cpp
    src/plugin/entry.cpp)
target_link_libraries(qsound-echo PRIVATE qsound_echo_core)

# Tests
add_executable(test_echo tests/test_echo_engine.cpp)
target_link_libraries(test_echo PRIVATE qsound_echo_core Catch2::Catch2WithMain)
include(CTest)
catch_discover_tests(test_echo)
```

**Justfile:**

```just
configure *FLAGS:
    cmake -B build -G Ninja \
        -DBUILD_IMGUI_UI=ON \
        -DBUILD_STANDALONE=ON \
        {{FLAGS}}

build:
    cmake --build build

test:
    ctest --test-dir build --output-on-failure

install:
    cp -r build/VST3/qsound-echo.vst3 ~/Library/Audio/Plug-Ins/VST3/

build-clap:
    just configure -DBUILD_CLAP=ON
    just build
```

---

## Testing Strategy

### Unit tests (Catch2)

1. `test_echo_silence_passthrough` — zero input with zero send → zero output
2. `test_echo_tail_decays` — impulse input with feedback < 0.25 → tail approaches zero
3. `test_echo_unity_feedback_sustains` — feedback = 0.25 → echo level stable after impulse
4. `test_echo_growing_feedback_clips` — feedback = 1.0 → buffer clamped, `isClipping()` returns true
5. `test_authentic_delay_range` — QSound mode constrains `delay_ms` to 57–170 ms
6. `test_native_rate_granularity` — QSound mode snaps to multiples of 1/24038 s
7. `test_negative_feedback_phase` — negative feedback → echo inverts polarity on first tap
8. `test_damping_zero_is_brighter` — damping=0 → higher-frequency content preserved in tail vs damping=0.5

### Smoke test

A Python script renders a 440 Hz sine burst through the standalone harness and checks:
- Output is non-silent when wet > 0 and send > 0
- No NaN/Inf in output buffer
- Duration extends beyond input burst (echo tail present)

---

## Phased Work Plan

### Phase 1 — Core DSP (1–2 days)

1. Create `src/core/echo_engine.h/cpp` with `EchoEngine` class
2. Implement the buffer, feedback loop, damping, and resampling
3. Write all 8 Catch2 unit tests
4. Verify against the QSound reference algorithm in `mml2vgm-rs/src/chips/qsound.rs`

### Phase 2 — VST3 scaffolding (1 day)

Copy and adapt combover-vst3's processor/controller/entry structure:
1. Generate new plugin UIDs (`uuidgen`)
2. Replace `CombFilter` with `EchoEngine` in `Processor`
3. Declare 7 parameters in `Controller::initialize()`
4. Wire automation reader per parameter in `process()`

### Phase 3 — ImGui UI (1–2 days)

1. Port combover-vst3's ImGui editor skeleton
2. Add delay slider with QSound-mode visual constraint
3. Add feedback colour zones (green/yellow/red)
4. Add clipping indicator LED
5. Add preset dropdown with 3 factory presets

### Phase 4 — Standalone harness + CLAP (1 day)

1. Copy standalone from combover-vst3; wire EchoEngine
2. Enable CLAP adapter (`BUILD_CLAP=ON`); test in a CLAP host

### Phase 5 — Polish (½ day)

1. Tune factory presets to interesting settings
2. Write README with QSound algorithm explanation
3. Smoke test on macOS and Linux

---

## Deferred / Out of Scope

- **Q1 FIR filter on wet path** — adds 45-sample latency; perceptually minor; deferred to v2
- **16-voice send matrix** — authentic QSound has per-voice send levels; a plugin with a single mix bus is simpler and more usable
- **ADPCM playback** — instrument territory; out of scope for an effect plugin
- **Tempo sync** — non-authentic; could be added as a simple BPM→ms conversion layer

---

## References

| Source | Notes |
|---|---|
| [combover-vst3](../../combover-vst3) | Direct architecture template (effect plugin) |
| [chiaro](../../chiaro) | Reference for larger parameter sets and synth-engine isolation |
| [docs/PLAN_QSound.md](./PLAN_QSound.md) | QSound hardware details, register map, algorithm derivation |
| MAME `qsoundhle.cpp` | Authoritative echo algorithm source |
