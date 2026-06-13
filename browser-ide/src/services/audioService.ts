/**
 * Audio Service
 * 
 * Manages audio playback using WASM chip emulators and VGM player.
 * Handles audio context, sample generation, and playback streaming.
 */

import { wasmService } from './wasmService';
import type { SoundChip, PlayerState } from '@/types';

// ============================================================================
// Types
// ============================================================================

export interface AudioServiceOptions {
  sampleRate?: number;
  bufferSize?: number;
  outputChannels?: number;
}

export interface AudioPlaybackOptions {
  chips: SoundChip[];
  volume?: number;
  loop?: boolean;
}

export interface AudioStatus {
  isPlaying: boolean;
  isPaused: boolean;
  currentTime: number;
  duration: number;
  sampleRate: number;
  chips: SoundChip[];
}

export interface AudioEventListener {
  onPlay?: () => void;
  onPause?: () => void;
  onStop?: () => void;
  onTimeUpdate?: (time: number) => void;
  onEnd?: () => void;
  onError?: (error: Error) => void;
}

export interface AudioRuntimeDebugInfo {
  isPlaying: boolean;
  isPaused: boolean;
  vgmDataLength: number;
  parsedCommandCount: number;
  nextCommandIndex: number;
  pendingCommandCount: number;
  appliedWriteCount: number;
  skippedWriteCount: number;
  generatedBufferCount: number;
  lastPeak: number;
  silentBufferCount: number;
  emittedSilenceWarning: boolean;
  chips: SoundChip[];
}

interface ParsedVgmCommand {
  timeSamples: number;
  chip: SoundChip;
  addr: number;
  data: number;
}

interface ParsedVgmStream {
  commands: ParsedVgmCommand[];
  /** Total length in samples (header field 0x18). 0 if absent. */
  totalSamples: number;
  /** Length of the loop body in samples (header field 0x20). 0 = no loop. */
  loopSamples: number;
  /**
   * Index into `commands` of the first command at or after the loop point.
   * `-1` if the VGM declares no loop (header field `0x1C` is zero) or if
   * the loop offset points past end of stream.
   */
  loopCommandIndex: number;
  /** Stream-time of `commands[loopCommandIndex]`, in samples. */
  loopSampleStart: number;
}

/**
 * Inspect a VGM byte stream's header and return the list of chips it declares.
 * Each chip occupies one little-endian u32 clock field at a fixed offset;
 * a nonzero value (with the high bit ignored, which marks dual-chip use)
 * indicates the chip is present.
 *
 * Only chips that also exist in the `SoundChip` union are reported — anything
 * else the chip player can't simulate anyway.
 */
const VGM_CHIP_CLOCK_OFFSETS: Array<{ offset: number; minVersion: number; chip: SoundChip }> = [
  { offset: 0x0C, minVersion: 0x100, chip: 'SN76489' },
  { offset: 0x10, minVersion: 0x100, chip: 'YM2413' },
  { offset: 0x2C, minVersion: 0x110, chip: 'YM2612' },
  { offset: 0x30, minVersion: 0x110, chip: 'YM2151' },
  { offset: 0x38, minVersion: 0x151, chip: 'SegaPCM' },
  { offset: 0x40, minVersion: 0x151, chip: 'RF5C164' },
  { offset: 0x44, minVersion: 0x151, chip: 'YM2203' },
  { offset: 0x48, minVersion: 0x151, chip: 'YM2608' },
  { offset: 0x4C, minVersion: 0x151, chip: 'YM2610B' },
  { offset: 0x50, minVersion: 0x151, chip: 'YM3812' },
  { offset: 0x54, minVersion: 0x151, chip: 'YM3526' },
  { offset: 0x58, minVersion: 0x151, chip: 'Y8950' },
  { offset: 0x5C, minVersion: 0x151, chip: 'YMF262' },
  { offset: 0x74, minVersion: 0x161, chip: 'AY8910' },
  { offset: 0x80, minVersion: 0x161, chip: 'DMG' },
  { offset: 0x84, minVersion: 0x161, chip: 'NES' },
  { offset: 0x9C, minVersion: 0x161, chip: 'K054539' },
  { offset: 0xA0, minVersion: 0x161, chip: 'HuC6280' },
  { offset: 0xA4, minVersion: 0x161, chip: 'C140' },
  { offset: 0xAC, minVersion: 0x161, chip: 'K053260' },
  { offset: 0xB0, minVersion: 0x161, chip: 'POKEY' },
  { offset: 0xB4, minVersion: 0x161, chip: 'QSound' },
  { offset: 0xC4, minVersion: 0x171, chip: 'C352' },
  { offset: 0xE0, minVersion: 0x171, chip: 'VRC6' },
];

const detectChipsFromVgmHeader = (data: Uint8Array): SoundChip[] => {
  if (data.length < 0x40) return [];
  // Magic check: "Vgm "
  if (data[0] !== 0x56 || data[1] !== 0x67 || data[2] !== 0x6D || data[3] !== 0x20) return [];

  const readU32 = (offset: number): number => {
    if (offset + 4 > data.length) return 0;
    return (
      data[offset] |
      (data[offset + 1] << 8) |
      (data[offset + 2] << 16) |
      (data[offset + 3] << 24)
    ) >>> 0;
  };

  const version = readU32(0x08);
  const found: SoundChip[] = [];
  for (const { offset, minVersion, chip } of VGM_CHIP_CLOCK_OFFSETS) {
    if (version < minVersion) continue;
    // Mask the top bit — VGM uses it as "dual chip" flag.
    const clock = readU32(offset) & 0x7fffffff;
    if (clock !== 0) found.push(chip);
  }
  return found;
};

/** Merge two chip lists preserving order and dropping duplicates. */
const mergeChips = (a: SoundChip[], b: SoundChip[]): SoundChip[] => {
  const seen = new Set<SoundChip>();
  const out: SoundChip[] = [];
  for (const chip of [...a, ...b]) {
    if (!seen.has(chip)) {
      seen.add(chip);
      out.push(chip);
    }
  }
  return out;
};

const decodeSn76489Write = (value: number, latchedAddr: number): { addr: number; data: number; nextLatchedAddr: number } => {
  if ((value & 0x80) !== 0) {
    const nextLatchedAddr = 0x80 | ((value >> 4) & 0x07);
    return {
      addr: nextLatchedAddr,
      data: value & 0x3f,
      nextLatchedAddr,
    };
  }

  return {
    addr: latchedAddr,
    data: value & 0x3f,
    nextLatchedAddr: latchedAddr,
  };
};

// ============================================================================
// Audio Service Class
// ============================================================================

/**
 * Service for managing audio playback in the browser IDE.
 * 
 * This class provides a high-level API for:
 * - Playing compiled MML/VGM data
 * - Real-time chip emulation playback
 * - Position tracking
 * - Event handling
 */
export class AudioService {
  private static instance: AudioService | null = null;
  
  // Audio context
  private audioContext: AudioContext | null = null;
  private audioWorkletNode: AudioWorkletNode | null = null;
  private audioDestination: AudioNode | null = null;
  
  // Sample buffer and queue
  private sampleBuffer: Float32Array | null = null;
  private isProcessing = false;

  // SharedArrayBuffer ring buffer for low-latency audio
  private sharedArrayBuffer: SharedArrayBuffer | null = null;
  private sharedRingBuffer: Float32Array | null = null;
  private sharedRingBufferBytes: Int32Array | null = null;
  private usingSharedArrayBuffer = false;
  
  // Playback state
  private _isPlaying = false;
  private _isPaused = false;
  private _startTime = 0;
  private _pauseTime = 0;
  private _currentTime = 0;
  
  // VGM player state
  private vgmPlayerId: string | null = null;
  private vgmData: Uint8Array | null = null;
  
  // Chip player state
  private chipPlayerId: string | null = null;
  private chips: SoundChip[] = [];
  
  // Per-chip volume settings (0.0 to 1.0)
  private chipVolumes: Map<SoundChip, number> = new Map();
  private chipMuteStates: Map<SoundChip, boolean> = new Map();
  private chipSoloStates: Map<SoundChip, boolean> = new Map();
  
  // Playback options
  private _volume = 1.0;
  private _loop = false;
  private _playbackRate = 1.0;
  
  // Sample rate and buffer configuration
  private sampleRate = 44100;
  private bufferSize = 4096;
  private outputChannels = 2;
  
  // Event listeners
  private listeners: AudioEventListener[] = [];
  
  // Animation frame for time updates
  private animationFrameId: number | null = null;
  private silentBufferCount = 0;
  private emittedSilenceWarning = false;
  private vgmCommands: ParsedVgmCommand[] = [];
  private nextVgmCommandIndex = 0;
  private vgmSampleCursor = 0;
  /** Total samples reported by the VGM header (0 if absent). */
  private vgmTotalSamples = 0;
  /** Length of the VGM loop body in samples (0 = no loop). */
  private vgmLoopSamples = 0;
  /** Index in `vgmCommands` of the loop entry, or -1 if no loop. */
  private vgmLoopCommandIndex = -1;
  /** Stream-time of the loop entry command, in samples. */
  private vgmLoopSampleStart = 0;
  /**
   * Number of samples by which the stream cursor has been shifted backward to
   * account for loop wraps. `streamPosition = targetSample - vgmLoopShift`.
   * Lets the audio thread keep advancing `targetSample` monotonically while
   * the VGM stream replays the loop body.
   */
  private vgmLoopShift = 0;
  private vgmLoopCount = 0;
  private appliedWriteCount = 0;
  private skippedWriteCount = 0;
  private generatedBufferCount = 0;
  private lastPeak = 0;
  private waveformRing = new Float32Array(4096);
  private waveformWriteIndex = 0;
  private waveformHasWrapped = false;

  private resetRuntimeDebugCounters(): void {
    this.silentBufferCount = 0;
    this.emittedSilenceWarning = false;
    this.vgmCommands = [];
    this.nextVgmCommandIndex = 0;
    this.vgmSampleCursor = 0;
    this.vgmTotalSamples = 0;
    this.vgmLoopSamples = 0;
    this.vgmLoopCommandIndex = -1;
    this.vgmLoopSampleStart = 0;
    this.vgmLoopShift = 0;
    this.vgmLoopCount = 0;
    this.appliedWriteCount = 0;
    this.skippedWriteCount = 0;
    this.generatedBufferCount = 0;
    this.lastPeak = 0;
    this.waveformRing.fill(0);
    this.waveformWriteIndex = 0;
    this.waveformHasWrapped = false;
  }

  private captureWaveformSamples(samples: Float32Array): void {
    const channelCount = Math.max(1, this.outputChannels);
    for (let i = 0; i < samples.length; i += channelCount) {
      let mono = 0;
      const availableChannels = Math.min(channelCount, samples.length - i);
      for (let c = 0; c < availableChannels; c++) {
        mono += samples[i + c];
      }
      mono /= availableChannels;

      this.waveformRing[this.waveformWriteIndex] = mono;
      this.waveformWriteIndex = (this.waveformWriteIndex + 1) % this.waveformRing.length;
      if (this.waveformWriteIndex === 0) {
        this.waveformHasWrapped = true;
      }
    }
  }

  /**
   * Check whether VGM data contains any register-write commands that can produce audio.
   */
  private hasAudibleVgmCommands(data: Uint8Array): boolean {
    if (!data || data.length < 0x40) {
      return false;
    }

    // VGM data offset is at 0x34 and is relative to 0x34.
    const dataOffset = (data[0x34] | (data[0x35] << 8) | (data[0x36] << 16) | (data[0x37] << 24)) >>> 0;
    let offset = dataOffset === 0 ? 0x40 : (0x34 + dataOffset);
    if (offset >= data.length) {
      offset = 0x40;
    }

    while (offset < data.length) {
      const cmd = data[offset++];

      // Known register write commands across common chips.
      if (
        cmd === 0x50 || // SN76489 write
        cmd === 0x51 || // YM2413 write
        cmd === 0x52 || // YM2612 port 0 write
        cmd === 0x53 || // YM2612 port 1 write
        cmd === 0x54 || // YM2151 write
        cmd === 0x55 || // YM2203 write
        cmd === 0x56 || // YM2608 port 0 write
        cmd === 0x57 || // YM2608 port 1 write
        cmd === 0x58 || // YM2610 port 0 write
        cmd === 0x59 || // YM2610 port 1 write
        cmd === 0x5A || // YM3812 write
        cmd === 0x5B || // YM3526 write
        cmd === 0x5C || // Y8950 write
        cmd === 0x5E || // YMF262 port 0 write
        cmd === 0x5F // YMF262 port 1 write
      ) {
        return true;
      }

      // Advance over command parameters so scan remains aligned.
      if (cmd >= 0x70 && cmd <= 0x7F) {
        continue; // short wait, no payload
      }

      switch (cmd) {
        case 0x00:
          // Some generated streams include explicit zero padding between waits.
          break;
        case 0x50:
          offset += 1;
          break;
        case 0x51:
        case 0x52:
        case 0x53:
        case 0x54:
        case 0x55:
        case 0x56:
        case 0x57:
        case 0x58:
        case 0x59:
        case 0x5A:
        case 0x5B:
        case 0x5C:
        case 0x5E:
        case 0x5F:
          offset += 2;
          break;
        case 0x61:
          // Standard VGM uses 16-bit waits, but some streams pad to 32-bit.
          if (offset + 4 <= data.length && data[offset + 2] === 0x00 && data[offset + 3] === 0x00) {
            offset += 4;
          } else {
            offset += 2;
          }
          break;
        case 0x62:
        case 0x63:
        case 0x66:
          break;
        case 0x67:
          // Data block: 0x67 0x66 tt ss ss ss ss [data]
          if (offset + 6 <= data.length && data[offset] === 0x66) {
            const size = (
              data[offset + 2] |
              (data[offset + 3] << 8) |
              (data[offset + 4] << 16) |
              (data[offset + 5] << 24)
            ) >>> 0;
            offset += 6 + size;
          }
          break;
        default:
          // Unknown command: stop scanning rather than risk desync.
          return false;
      }
    }

    return false;
  }

  /**
   * Parse VGM command stream into register writes with sample timestamps.
   *
   * Also extracts the looping metadata from the header (offsets 0x18, 0x1C,
   * 0x20) so the playback loop can wrap back to the loop point when the VGM
   * ends. See VGM 1.71 spec section 3 ("Header").
   */
  private parseVgmStream(data: Uint8Array): ParsedVgmStream {
    const empty: ParsedVgmStream = {
      commands: [],
      totalSamples: 0,
      loopSamples: 0,
      loopCommandIndex: -1,
      loopSampleStart: 0,
    };
    if (!data || data.length < 0x40) {
      return empty;
    }

    const readU32 = (off: number): number => {
      if (off + 4 > data.length) return 0;
      return (data[off] | (data[off + 1] << 8) | (data[off + 2] << 16) | (data[off + 3] << 24)) >>> 0;
    };

    const commands: ParsedVgmCommand[] = [];
    const dataOffset = readU32(0x34);
    let offset = dataOffset === 0 ? 0x40 : (0x34 + dataOffset);
    if (offset >= data.length) {
      offset = 0x40;
    }

    const totalSamples = readU32(0x18);
    const loopSamples = readU32(0x20);
    // Header 0x1C: loop offset RELATIVE TO 0x1C. 0 = no loop.
    const loopRel = readU32(0x1C);
    const loopAbsByte = loopRel === 0 ? -1 : (0x1C + loopRel);

    let loopCommandIndex = -1;
    let loopSampleStart = 0;
    let currentTime = 0;
    let snLatchedAddr = 0x80;

    const push = (chip: SoundChip, addr: number, regData: number) => {
      commands.push({
        timeSamples: currentTime,
        chip,
        addr: addr & 0xff,
        data: regData & 0xff,
      });
    };

    while (offset < data.length) {
      // Loop-point detection: when our read cursor first reaches or crosses
      // the loop-start byte, mark the NEXT command we'll push as the loop
      // entry. This is checked before consuming each opcode so the marker
      // lands on the command itself, not on a preceding wait.
      if (loopAbsByte >= 0 && loopCommandIndex === -1 && offset >= loopAbsByte) {
        loopCommandIndex = commands.length;
        loopSampleStart = currentTime;
      }

      const cmd = data[offset++];

      if (cmd >= 0x70 && cmd <= 0x7f) {
        currentTime += (cmd & 0x0f) + 1;
        continue;
      }

      switch (cmd) {
        case 0x50: {
          if (offset + 1 > data.length) break;
          const value = data[offset++];
          const decoded = decodeSn76489Write(value, snLatchedAddr);
          snLatchedAddr = decoded.nextLatchedAddr;
          push('SN76489', decoded.addr, decoded.data);
          break;
        }
        case 0x52:
        case 0x53: {
          if (offset + 2 > data.length) break;
          const addr = data[offset++];
          const regData = data[offset++];
          push('YM2612', addr, regData);
          break;
        }
        case 0x54: {
          if (offset + 2 > data.length) break;
          const addr = data[offset++];
          const regData = data[offset++];
          push('YM2151', addr, regData);
          break;
        }
        case 0x55: {
          if (offset + 2 > data.length) break;
          const addr = data[offset++];
          const regData = data[offset++];
          push('YM2203', addr, regData);
          break;
        }
        case 0x56:
        case 0x57: {
          if (offset + 2 > data.length) break;
          const addr = data[offset++];
          const regData = data[offset++];
          push('YM2608', addr, regData);
          break;
        }
        case 0x5A: {
          if (offset + 2 > data.length) break;
          const addr = data[offset++];
          const regData = data[offset++];
          push('YM3812', addr, regData);
          break;
        }
        case 0x5B: {
          if (offset + 2 > data.length) break;
          const addr = data[offset++];
          const regData = data[offset++];
          push('YM3526', addr, regData);
          break;
        }
        case 0x5C: {
          if (offset + 2 > data.length) break;
          const addr = data[offset++];
          const regData = data[offset++];
          push('Y8950', addr, regData);
          break;
        }
        case 0x5E:
        case 0x5F: {
          if (offset + 2 > data.length) break;
          const addr = data[offset++];
          const regData = data[offset++];
          push('YMF262', addr, regData);
          break;
        }
        case 0x61: {
          if (offset + 2 > data.length) break;
          const wait = data[offset] | (data[offset + 1] << 8);
          // Accept streams that pad 0x61 waits to 32-bit values.
          if (offset + 4 <= data.length && data[offset + 2] === 0x00 && data[offset + 3] === 0x00) {
            offset += 4;
          } else {
            offset += 2;
          }
          currentTime += wait;
          break;
        }
        case 0x62:
          currentTime += 735;
          break;
        case 0x63:
          currentTime += 882;
          break;
        case 0x66:
          return { commands, totalSamples, loopSamples, loopCommandIndex, loopSampleStart };
        case 0x67: {
          // Data block: 0x67 0x66 tt ss ss ss ss [data]
          if (offset + 6 > data.length) return { commands, totalSamples, loopSamples, loopCommandIndex, loopSampleStart };
          if (data[offset] !== 0x66) return { commands, totalSamples, loopSamples, loopCommandIndex, loopSampleStart };
          const size = (
            data[offset + 2] |
            (data[offset + 3] << 8) |
            (data[offset + 4] << 16) |
            (data[offset + 5] << 24)
          ) >>> 0;
          offset += 6 + size;
          break;
        }
        case 0x00:
          // Padding/no-op in some generated streams.
          break;
        default:
          // Unknown command: stop to avoid stream desync.
          return finalizeStream();
      }
    }

    return finalizeStream();

    function finalizeStream(): ParsedVgmStream {
      // A loop offset that lands past the last command is meaningless —
      // collapse it to "no loop" so the playback loop doesn't spin on an
      // unreachable target.
      let finalLoopIdx = loopCommandIndex;
      let finalLoopStart = loopSampleStart;
      if (finalLoopIdx >= commands.length) {
        finalLoopIdx = -1;
        finalLoopStart = 0;
      }
      return {
        commands,
        totalSamples,
        loopSamples,
        loopCommandIndex: finalLoopIdx,
        loopSampleStart: finalLoopStart,
      };
    }
  }

  /**
   * Apply pending VGM register writes up to the target sample index.
   */
  private async applyPendingVgmCommands(targetSample: number): Promise<void> {
    if (!this.chipPlayerId || this.vgmCommands.length === 0) {
      this.vgmSampleCursor = targetSample;
      return;
    }

    // Stream position is wall-clock target minus accumulated loop shift.
    // When we wrap, we extend `vgmLoopShift` so command timestamps (which are
    // absolute within the VGM) line up with the freshly-rewound cursor.
    const canLoop = this._loop && this.vgmLoopCommandIndex >= 0 && this.vgmLoopSamples > 0;

    // Outer loop handles the case where the loop body is shorter than one
    // audio quantum, so we might need to wrap multiple times in one call.
    // The chip player itself never resets — register state carries through.
    // Safety cap so a degenerate VGM (e.g. zero-length loop body that slipped
    // past the parser check) can't spin the JS thread forever.
    let safety = 8;
    while (safety-- > 0) {
      const streamPosition = targetSample - this.vgmLoopShift;

      while (this.nextVgmCommandIndex < this.vgmCommands.length) {
        const command = this.vgmCommands[this.nextVgmCommandIndex];
        if (command.timeSamples > streamPosition) {
          break;
        }

        if (this.chips.includes(command.chip)) {
          await wasmService.writeChipRegister(this.chipPlayerId, command.chip, command.addr, command.data);
          this.appliedWriteCount += 1;
        } else {
          this.skippedWriteCount += 1;
        }

        this.nextVgmCommandIndex += 1;
      }

      // Did we just consume the last command, or pass the declared total?
      const reachedEnd =
        this.nextVgmCommandIndex >= this.vgmCommands.length ||
        (this.vgmTotalSamples > 0 && streamPosition >= this.vgmTotalSamples);

      if (!reachedEnd) {
        break;
      }

      if (canLoop) {
        // Rewind the stream to the loop entry. Bumping `vgmLoopShift` keeps
        // `targetSample` monotonic so wall-clock time tracking stays sane.
        this.vgmLoopShift += this.vgmLoopSamples;
        this.nextVgmCommandIndex = this.vgmLoopCommandIndex;
        this.vgmLoopCount += 1;
        continue; // process commands at the new (rewound) position
      }

      // Non-looping end-of-stream: emit once.
      if (this._isPlaying) {
        this.emitEnd();
        this._isPlaying = false;
      }
      break;
    }

    this.vgmSampleCursor = targetSample;
  }
  
  // ========================================================================
  // Singleton
  // ========================================================================
  
  public static getInstance(): AudioService {
    if (!AudioService.instance) {
      AudioService.instance = new AudioService();
    }
    return AudioService.instance;
  }
  
  private constructor() {
    // Private constructor for singleton
  }
  
  // ========================================================================
  // Initialization
  // ========================================================================
  
  /**
   * Initialize the audio service with options.
   */
  public async init(options: AudioServiceOptions = {}): Promise<void> {
    // Merge options
    this.sampleRate = options.sampleRate || 44100;
    this.bufferSize = options.bufferSize || 4096;
    this.outputChannels = options.outputChannels || 2;
    
    // Initialize SharedArrayBuffer ring buffer if supported
    if (this.hasSharedArrayBufferSupport()) {
      this.initSharedRingBuffer();
    } else {
      console.log('[AudioService] SharedArrayBuffer not supported, using postMessage fallback');
      this.usingSharedArrayBuffer = false;
    }
    
    // Create audio context
    try {
      this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)({
        sampleRate: this.sampleRate,
      });
      
      // Setup AudioWorklet
      await this.setupAudioWorklet();
      
      console.log('[AudioService] Initialized successfully');
    } catch (error) {
      console.error('[AudioService] Initialization failed:', error);
      throw error;
    }
  }

  /**
   * Check if SharedArrayBuffer is supported with the required headers.
   */
  private hasSharedArrayBufferSupport(): boolean {
    // Check if SharedArrayBuffer is available
    if (typeof SharedArrayBuffer === 'undefined') {
      return false;
    }
    
    // Check if the required COOP/COEP headers are set
    // These are needed for SharedArrayBuffer to work with AudioWorklet
    try {
      // Try to create a small SharedArrayBuffer as a test
      // This will throw if headers are not set
      if (typeof crossOriginIsolated === 'boolean') {
        return crossOriginIsolated;
      }
      // Fallback check: try to create one
      // Note: In some browsers, this might not throw but still fail in AudioWorklet
      return true;
    } catch (e) {
      return false;
    }
  }

  /**
   * Initialize the SharedArrayBuffer ring buffer.
   * Uses a circular buffer with Atomics for synchronization.
   */
  private initSharedRingBuffer(): void {
    // Calculate capacity: enough for ~50ms of audio at 44100Hz stereo
    // 44100 samples/sec * 0.05 sec * 2 channels = 4410 samples
    const capacity = this.sampleRate * this.outputChannels;
    
    // Create SharedArrayBuffer: Float32Array for samples + Int32Array for indices
    // Float32Array: capacity * 4 bytes per sample
    // Int32Array: 4 integers (readIndex, writeIndex, capacity, bufferSize) * 4 bytes each = 16 bytes
    const bufferSize = (capacity * 4) + 16;
    
    try {
      this.sharedArrayBuffer = new SharedArrayBuffer(bufferSize);
      this.sharedRingBuffer = new Float32Array(this.sharedArrayBuffer, 0, capacity);
      this.sharedRingBufferBytes = new Int32Array(this.sharedArrayBuffer, capacity * 4, 4);
      
      // Initialize indices
      this.sharedRingBufferBytes[0] = 0; // readIndex
      this.sharedRingBufferBytes[1] = 0; // writeIndex
      this.sharedRingBufferBytes[2] = capacity; // capacity
      this.sharedRingBufferBytes[3] = this.bufferSize; // audio buffer size
      
      this.usingSharedArrayBuffer = true;
      console.log('[AudioService] SharedArrayBuffer ring buffer initialized, capacity:', capacity);
    } catch (e) {
      console.warn('[AudioService] Failed to create SharedArrayBuffer:', e);
      this.usingSharedArrayBuffer = false;
    }
  }

  /**
   * Ensure the browser AudioContext is running before playback.
   * Some browsers (especially in strict autoplay contexts) keep it suspended
   * until explicitly resumed from a user gesture.
   */
  private async ensureAudioContextRunning(): Promise<void> {
    if (!this.audioContext) {
      return;
    }

    if (this.audioContext.state === 'running') {
      return;
    }

    console.warn('[AudioService] AudioContext state before resume:', this.audioContext.state);
    await this.audioContext.resume();
    console.warn('[AudioService] AudioContext state after resume:', this.audioContext.state);
  }
  
  /**
   * Setup AudioWorklet processor for sample playback.
   */
  private async setupAudioWorklet(): Promise<void> {
    if (!this.audioContext) {
      throw new Error('AudioContext not initialized');
    }
    
    try {
      // Check if AudioWorklet is supported
      if (!window.AudioWorklet) {
        console.warn('[AudioService] AudioWorklet not supported, falling back to ScriptProcessorNode');
        this.setupScriptProcessorNode();
        return;
      }
      
      // Define the AudioWorklet processor
      // When SharedArrayBuffer is available, use it for zero-copy sample transfer
      // Otherwise fall back to postMessage-based transfer
      const useSharedBuffer = this.usingSharedArrayBuffer && this.sharedArrayBuffer;
      
      const processorCode = useSharedBuffer ? `
        class MMLAudioProcessor extends AudioWorkletProcessor {
          constructor() {
            super();
            this.sharedRingBuffer = null;
            this.sharedRingBufferBytes = null;
            this.channelCount = 0;
            this.bufferSize = 0;
          }
          
          process(inputs, outputs, parameters) {
            const output = outputs[0];
            const channelCount = output.length;
            const samplesNeeded = output[0].length;
            
            // Initialize on first process call
            if (!this.sharedRingBuffer) {
              this.port.onmessage = (e) => {
                if (e.data.type === 'sharedBuffer') {
                  this.sharedRingBuffer = e.data.ringBuffer;
                  this.sharedRingBufferBytes = e.data.ringBufferBytes;
                  this.channelCount = e.data.channelCount || ${this.outputChannels};
                }
              };
              
              // If buffer not yet received, output silence
              for (let i = 0; i < samplesNeeded; i++) {
                for (let c = 0; c < channelCount; c++) {
                  output[c][i] = 0;
                }
              }
              return true;
            }
            
            const ringBuffer = this.sharedRingBuffer;
            const ringBufferBytes = this.sharedRingBufferBytes;
            const capacity = ringBufferBytes[2];

            // The producer (chip player) writes INTERLEAVED STEREO: every
            // stereo frame is 2 consecutive floats (L, R). Capacity is always
            // a multiple of 2 and the producer always writes in pairs, so the
            // wraparound math stays frame-aligned.
            const STRIDE = 2;

            // Non-blocking drain. Atomics.wait is forbidden on the AudioWorklet
            // render thread, so we read what's available, pad the remainder with
            // silence, and rely on the next process() call to consume more.
            const readIndex = Atomics.load(ringBufferBytes, 0);
            const writeIndex = Atomics.load(ringBufferBytes, 1);

            let available = writeIndex - readIndex;
            if (available < 0) {
              available += capacity;
            }
            const framesAvailable = (available / STRIDE) | 0;
            const framesToRead = framesAvailable < samplesNeeded ? framesAvailable : samplesNeeded;

            for (let i = 0; i < framesToRead; i++) {
              const base = (readIndex + i * STRIDE) % capacity;
              const left = ringBuffer[base];
              const right = ringBuffer[(base + 1) % capacity];
              if (channelCount === 1) {
                output[0][i] = (left + right) * 0.5;
              } else {
                output[0][i] = left;
                output[1][i] = right;
                for (let c = 2; c < channelCount; c++) {
                  output[c][i] = (left + right) * 0.5;
                }
              }
            }
            for (let i = framesToRead; i < samplesNeeded; i++) {
              for (let c = 0; c < channelCount; c++) {
                output[c][i] = 0;
              }
            }

            if (framesToRead > 0) {
              const newReadIndex = (readIndex + framesToRead * STRIDE) % capacity;
              Atomics.store(ringBufferBytes, 0, newReadIndex);
              Atomics.notify(ringBufferBytes, 0, 1);
            }

            return true;
          }
        }
        
        registerProcessor('mml-audio-processor', MMLAudioProcessor);
      ` : `
        class MMLAudioProcessor extends AudioWorkletProcessor {
          constructor() {
            super();
            this.sampleBuffer = null;
            this.bufferIndex = 0;
            this.port.onmessage = (e) => {
              if (e.data.type === 'samples') {
                this.sampleBuffer = e.data.samples;
                this.bufferIndex = 0;
              }
            };
          }
          
          process(inputs, outputs, parameters) {
            const output = outputs[0];
            const channelCount = output.length;
            const samplesNeeded = output[0].length;
            
            if (!this.sampleBuffer || this.bufferIndex >= this.sampleBuffer.length) {
              // Fill with silence
              for (let i = 0; i < samplesNeeded; i++) {
                for (let c = 0; c < channelCount; c++) {
                  output[c][i] = 0;
                }
              }
              // Request more samples
              this.port.postMessage({ type: 'needSamples', count: samplesNeeded * channelCount });
              return true;
            }
            
            // Copy samples from buffer
            for (let i = 0; i < samplesNeeded; i++) {
              if (this.bufferIndex < this.sampleBuffer.length) {
                const sample = this.sampleBuffer[this.bufferIndex++];
                for (let c = 0; c < channelCount; c++) {
                  output[c][i] = sample;
                }
              } else {
                for (let c = 0; c < channelCount; c++) {
                  output[c][i] = 0;
                }
              }
            }
            
            // Request more samples if buffer is getting low
            if (this.bufferIndex > this.sampleBuffer.length * 0.8) {
              this.port.postMessage({ type: 'needSamples', count: samplesNeeded * channelCount });
            }
            
            return true;
          }
        }
        
        registerProcessor('mml-audio-processor', MMLAudioProcessor);
      `;
      
      // Load the processor
      const blob = new Blob([processorCode], { type: 'application/javascript' });
      const processorUrl = URL.createObjectURL(blob);
      
      await this.audioContext.audioWorklet.addModule(processorUrl);
      
      // Create the worklet node
      this.audioWorkletNode = new AudioWorkletNode(
        this.audioContext,
        'mml-audio-processor',
        {
          numberOfInputs: 0,
          numberOfOutputs: 1,
          outputChannelCount: [this.outputChannels],
        }
      );
      
      // Connect to destination
      this.audioWorkletNode.connect(this.audioContext.destination);
      this.audioDestination = this.audioWorkletNode;
      
      // If using SharedArrayBuffer, pass the buffer to the worklet
      if (useSharedBuffer && this.sharedArrayBuffer && this.sharedRingBuffer && this.sharedRingBufferBytes) {
        try {
          this.audioWorkletNode.port.postMessage({
            type: 'sharedBuffer',
            ringBuffer: this.sharedRingBuffer,
            ringBufferBytes: this.sharedRingBufferBytes,
            channelCount: this.outputChannels,
          });
          console.log('[AudioService] SharedArrayBuffer passed to AudioWorklet');
        } catch (e) {
          console.warn('[AudioService] Failed to pass SharedArrayBuffer to AudioWorklet:', e);
          this.usingSharedArrayBuffer = false;
        }
      }
      
      // Setup message handler
      this.audioWorkletNode.port.onmessage = (e) => {
        if (e.data.type === 'needSamples') {
          this.generateMoreSamples(e.data.count);
        }
      };
      
      // Setup error handler
      this.audioWorkletNode.port.onmessageerror = (e) => {
        console.error('[AudioService] AudioWorklet error:', e);
        this.emitError(new Error('AudioWorklet processing error'));
      };
      
      console.log('[AudioService] AudioWorklet processor loaded');
      
    } catch (error) {
      console.warn('[AudioService] AudioWorklet setup failed, falling back:', error);
      this.setupScriptProcessorNode();
    }
  }

  /**
   * Write samples to the SharedArrayBuffer ring buffer.
   *
   * Non-blocking: writes only what currently fits, then returns. `Atomics.wait`
   * is forbidden on the main thread (which is where the sample-generation loop
   * runs), so back-pressure is handled by the caller — `startSampleGeneration`
   * reschedules via `setTimeout(generateSamples, 0)`, giving the AudioWorklet
   * consumer time to drain. Any samples that don't fit this tick are dropped;
   * with the ring sized at 2× the worklet quantum this only happens if the
   * consumer has stalled, in which case dropping is the right behaviour.
   *
   * Returns the number of samples actually written.
   */
  private writeSamplesToRingBuffer(samples: Float32Array): number {
    if (!this.sharedRingBuffer || !this.sharedRingBufferBytes) {
      return 0;
    }

    const ringBuffer = this.sharedRingBuffer;
    const ringBufferBytes = this.sharedRingBufferBytes;
    const capacity = ringBufferBytes[2];

    const readIndex = Atomics.load(ringBufferBytes, 0);
    const writeIndex = Atomics.load(ringBufferBytes, 1);

    let availableSpace = readIndex - writeIndex;
    if (availableSpace <= 0) {
      availableSpace += capacity;
    }
    // Reserve one stereo frame (2 floats) so the buffer-full and buffer-empty
    // conditions stay distinguishable AND writeIndex stays even (matching the
    // worklet's stride-2 read). `samples` from the chip player is always an
    // even-length interleaved-stereo array.
    const spaceForSamples = Math.max(0, (availableSpace - 2) & ~1);
    const samplesToWrite = Math.min(samples.length & ~1, spaceForSamples);
    if (samplesToWrite <= 0) {
      return 0;
    }

    for (let i = 0; i < samplesToWrite; i++) {
      ringBuffer[(writeIndex + i) % capacity] = samples[i];
    }

    const newWriteIndex = (writeIndex + samplesToWrite) % capacity;
    Atomics.store(ringBufferBytes, 1, newWriteIndex);
    Atomics.notify(ringBufferBytes, 1, 1);

    return samplesToWrite;
  }

  /**
   * Fallback to ScriptProcessorNode for browsers without AudioWorklet.
   */
  private setupScriptProcessorNode(): void {
    if (!this.audioContext) return;
    
    try {
      this.audioWorkletNode = (this.audioContext as any).createScriptProcessor(
        this.bufferSize,
        0,
        this.outputChannels
      );
      
      this.audioWorkletNode.onaudioprocess = (e: any) => {
        const output = e.outputBuffer;
        const channelCount = output.numberOfChannels;
        const samplesNeeded = output.length;
        
        for (let c = 0; c < channelCount; c++) {
          const channelData = output.getChannelData(c);
          for (let i = 0; i < samplesNeeded; i++) {
            // Fill with samples or silence
            if (this.sampleBuffer && this.bufferIndex < this.sampleBuffer.length) {
              channelData[i] = this.sampleBuffer[this.bufferIndex++];
            } else {
              channelData[i] = 0;
            }
          }
        }
        
        // Request more samples
        this.generateMoreSamples(samplesNeeded * 2);
      };
      
      this.audioWorkletNode.connect(this.audioContext.destination);
      this.audioDestination = this.audioWorkletNode;
      
      console.log('[AudioService] Using ScriptProcessorNode fallback');
    } catch (error) {
      console.error('[AudioService] Failed to create ScriptProcessorNode:', error);
    }
  }
  
  // ========================================================================
  // VGM Playback
  // ========================================================================
  
  /**
   * Load and play VGM data.
   * Uses chip player for real-time emulation of VGM commands.
   */
  public async playVGM(data: Uint8Array, options: AudioPlaybackOptions = { chips: [], volume: 1.0 }): Promise<void> {
    // Ensure audio is initialized
    if (!this.audioContext) {
      await this.init();
    }
    
    // Stop any current playback
    this.stop();

    await this.ensureAudioContextRunning();
    
    // Store VGM data
    this.vgmData = data;
    this.resetRuntimeDebugCounters();
    this._volume = options.volume || 1.0;
    this._loop = options.loop || false;
    
    try {
      if (!this.hasAudibleVgmCommands(data)) {
        console.warn('[AudioService] VGM command scan found no recognizable register-write commands; continuing playback attempt.');
      }

      // Decide which chips the chip player should instantiate.
      //
      // Trust the VGM header first — it declares each chip via a nonzero clock
      // field. The caller's `options.chips` is only used to *augment* the
      // header-detected set, never to replace it, because a VGM containing
      // YM2612 writes will be silent if the chip player isn't given a YM2612
      // (every write to `command.chip='YM2612'` gets skipped in
      // `applyPendingVgmCommands`).
      const detectedChips = detectChipsFromVgmHeader(data);
      const requestedChips = options.chips.length > 0 ? options.chips : [];
      const chipsToUse = mergeChips(detectedChips, requestedChips);
      if (chipsToUse.length === 0) {
        // Last-resort fallback so we still attempt playback if header parsing failed.
        chipsToUse.push('YM2608', 'SN76489');
      }
      console.log('[AudioService] Chips for playback:', chipsToUse,
        '(detected:', detectedChips, 'requested:', requestedChips, ')');

      // Create chip player
      await this.createChipPlayer(chipsToUse, this.sampleRate);

      // Parse VGM commands + loop metadata; reset playback cursor.
      const stream = this.parseVgmStream(data);
      this.vgmCommands = stream.commands;
      this.vgmTotalSamples = stream.totalSamples;
      this.vgmLoopSamples = stream.loopSamples;
      this.vgmLoopCommandIndex = stream.loopCommandIndex;
      this.vgmLoopSampleStart = stream.loopSampleStart;
      this.vgmLoopShift = 0;
      this.vgmLoopCount = 0;
      this.nextVgmCommandIndex = 0;
      this.vgmSampleCursor = 0;
      console.log('[AudioService] Parsed VGM stream:',
        'commands=', this.vgmCommands.length,
        'totalSamples=', this.vgmTotalSamples,
        'loopSamples=', this.vgmLoopSamples,
        'loopCmdIdx=', this.vgmLoopCommandIndex);
      if (this.vgmCommands.length === 0) {
        throw new Error(
          'Compiled VGM contains no playable register-write commands. Check MML part/channel syntax (for example, use @0/@1 instead of legacy labels).'
        );
      }
      
      // TODO: Load VGM data into chip player registers
      // For now, we'll use the compile approach
      // In the future, we should parse VGM commands and send to chip registers
      
      // For Phase 4, we'll use the compileStore to compile and get data
      // Then play via chip emulation
      
      // Start playback with empty data (samples generated by chip player)
      this.startPlayback();
      
      // Emit play event
      this.emitPlay();
      
    } catch (error) {
      console.error('[AudioService] VGM playback failed:', error);
      this.emitError(error as Error);
    }
  }
  
  // ========================================================================
  // Chip Playback
  // ========================================================================
  
  /**
   * Create a chip player with specified chips.
   */
  public async createChipPlayer(chips: SoundChip[], sampleRate: number = 44100): Promise<string> {
    this.chips = chips;
    this.chipPlayerId = await wasmService.createChipPlayer(sampleRate);

    // Add all chips
    for (const chip of chips) {
      await wasmService.addChipToPlayer(this.chipPlayerId, chip);
    }

    // Apply any pre-existing mixer state (volume/mute/solo) to the new player.
    // Without this, sliders the user moved before pressing Play silently revert
    // to 1.0 when a fresh chip player is instantiated.
    this.pushEffectiveChipGains();

    return this.chipPlayerId;
  }
  
  /**
   * Play MML directly using chip emulation.
   */
  public async playMML(mml: string, chips: SoundChip[] = []): Promise<void> {
    // Ensure audio is initialized
    if (!this.audioContext) {
      await this.init();
    }
    
    // Stop any current playback
    this.stop();
    
    try {
      // Create chip player
      await this.createChipPlayer(chips, this.sampleRate);
      
      // Compile MML to get commands
      // TODO: This should use the compileStore
      const options = { format: 'vgm', target_chips: chips };
      const result = await wasmService.compile(mml, options);
      
      if (result.data && result.data.length > 0) {
        // For now, play via VGM player
        await this.playVGM(result.data, { chips, volume: this._volume });
      }
      
    } catch (error) {
      console.error('[AudioService] MML playback failed:', error);
      this.emitError(error as Error);
    }
  }
  
  // ========================================================================
  // Playback Control
  // ========================================================================
  
  /**
   * Start playback (after load).
   */
  private startPlayback(): void {
    if (!this.audioContext || !this.audioDestination) return;
    
    this._isPlaying = true;
    this._isPaused = false;
    this.silentBufferCount = 0;
    this.emittedSilenceWarning = false;
    this._startTime = this.audioContext.currentTime - (this._pauseTime / 1000);
    
    // Start time tracking
    this.startTimeTracking();
    
    // Start sample generation
    this.startSampleGeneration();
    
    console.log('[AudioService] Playback started');
  }
  
  /**
   * Start generating samples from WASM player.
   */
  private startSampleGeneration(): void {
    if (!this.chipPlayerId && !this.vgmPlayerId) return;
    
    const generateSamples = async () => {
      if (!this._isPlaying || this.isProcessing) return;
      
      this.isProcessing = true;
      
      try {
        // Generate samples from the appropriate player
        let samples: Float32Array;
        
        if (this.vgmPlayerId) {
          // VGM player doesn't have direct sample generation - need to use chip player
          // For VGM playback, we use the chip player approach
          samples = new Float32Array(this.bufferSize * this.outputChannels);
        } else if (this.chipPlayerId) {
          if (this.vgmCommands.length > 0) {
            await this.applyPendingVgmCommands(this.vgmSampleCursor + Math.round(this.bufferSize * this._playbackRate));
          }
          samples = await wasmService.generateSamples(this.chipPlayerId, this.bufferSize);
        } else {
          this.isProcessing = false;
          return;
        }
        
        // Apply volume
        if (this._volume !== 1.0) {
          for (let i = 0; i < samples.length; i++) {
            samples[i] *= this._volume;
          }
        }

        // Lightweight silence detector to expose no-audio conditions.
        let peak = 0;
        for (let i = 0; i < samples.length; i++) {
          const v = Math.abs(samples[i]);
          if (v > peak) peak = v;
        }
        if (peak < 1e-6) {
          this.silentBufferCount += 1;
        } else {
          this.silentBufferCount = 0;
        }
        this.lastPeak = peak;
        this.generatedBufferCount += 1;
        this.captureWaveformSamples(samples);
        if (!this.emittedSilenceWarning && this.silentBufferCount > 25) {
          this.emittedSilenceWarning = true;
          console.warn('[AudioService] Generated samples remain silent. VGM command rendering to chip registers is likely not implemented yet.');
          this.emitError(new Error('Playback is running but generated audio is silent.'));
        }
        
        // If using SharedArrayBuffer, write directly to the ring buffer
        if (this.usingSharedArrayBuffer && this.sharedRingBuffer && this.sharedRingBufferBytes) {
          this.writeSamplesToRingBuffer(samples);
        } else {
          // Fallback: Store samples for playback
          this.sampleBuffer = samples;
          this.bufferIndex = 0;
          
          // Send to audio worklet
          if (this.audioWorkletNode) {
            this.audioWorkletNode.port.postMessage({
              type: 'samples',
              samples: samples,
            });
          }
        }
        
      } catch (error) {
        console.error('[AudioService] Sample generation error:', error);
      } finally {
        this.isProcessing = false;
        
        // Continue loop
        if (this._isPlaying) {
          setTimeout(generateSamples, 0);
        }
      }
    };
    
    // Start the loop
    generateSamples();
  }
  
  /**
   * Generate more samples when requested by AudioWorklet.
   */
  private generateMoreSamples(count: number): void {
    if (!this._isPlaying || this.isProcessing) return;
    
    // Trigger sample generation
    this.startSampleGeneration();
  }
  
  /**
   * Start time tracking loop.
   */
  private startTimeTracking(): void {
    const updateTime = () => {
      if (!this._isPlaying || !this.audioContext) {
        if (this.animationFrameId) {
          cancelAnimationFrame(this.animationFrameId);
          this.animationFrameId = null;
        }
        return;
      }
      
      this._currentTime = (this.audioContext.currentTime - this._startTime) * 1000; // in ms
      this.emitTimeUpdate(this._currentTime);
      
      this.animationFrameId = requestAnimationFrame(updateTime);
    };
    
    updateTime();
  }
  
  /**
   * Pause playback.
   */
  public pause(): void {
    if (!this._isPlaying) return;
    
    this._isPaused = true;
    this._isPlaying = false;
    this._pauseTime = this._currentTime;
    
    // Stop sample generation
    // Note: AudioWorklet will continue playing buffered samples
    
    this.emitPause();
    console.log('[AudioService] Playback paused');
  }
  
  /**
   * Resume playback.
   */
  public resume(): void {
    if (!this._isPaused) return;
    
    this._isPaused = false;
    this._isPlaying = true;
    this._startTime = (this.audioContext?.currentTime || 0) - (this._pauseTime / 1000);
    
    // Restart sample generation
    this.startSampleGeneration();
    this.startTimeTracking();
    
    this.emitPlay();
    console.log('[AudioService] Playback resumed');
  }
  
  /**
   * Stop playback.
   */
  public stop(): void {
    if (!this._isPlaying && !this._isPaused) return;
    
    this._isPlaying = false;
    this._isPaused = false;
    this._currentTime = 0;
    this._pauseTime = 0;
    this.resetRuntimeDebugCounters();
    
    // Stop sample generation
    this.isProcessing = false;
    
    // Stop time tracking
    if (this.animationFrameId) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
    
    // Stop WASM players
    if (this.vgmPlayerId) {
      wasmService.stopVgm(this.vgmPlayerId).catch(console.error);
      wasmService.destroyVgmPlayer(this.vgmPlayerId);
      this.vgmPlayerId = null;
    }
    
    if (this.chipPlayerId) {
      wasmService.destroyChipPlayer(this.chipPlayerId);
      this.chipPlayerId = null;
    }
    
    // Clear sample queue
    this.sampleQueue = [];
    this.sampleBuffer = null;
    this.bufferIndex = 0;
    
    this.emitStop();
    console.log('[AudioService] Playback stopped');
  }
  
  /**
   * Seek to a specific position (in milliseconds).
   */
  public seek(time: number): void {
    if (!this._isPlaying && !this._isPaused) return;
    
    this._currentTime = time;
    this._startTime = (this.audioContext?.currentTime || 0) - (time / 1000);
    
    // TODO: Seek in WASM player
    
    this.emitTimeUpdate(this._currentTime);
    console.log('[AudioService] Seek to', time, 'ms');
  }
  
  // ========================================================================
  // Volume Control
  // ========================================================================
  
  /**
   * Set master volume (0.0 to 1.0).
   */
  public setVolume(volume: number): void {
    this._volume = Math.max(0, Math.min(1, volume));
    console.log('[AudioService] Volume set to', this._volume);
  }
  
  /**
   * Get current volume.
   */
  public getVolume(): number {
    return this._volume;
  }
  
  // ========================================================================
  // Loop Control
  // ========================================================================
  
  /**
   * Set loop mode.
   */
  public setLoop(loop: boolean): void {
    this._loop = loop;
    console.log('[AudioService] Loop', loop ? 'enabled' : 'disabled');
  }
  
  /**
   * Get loop mode.
   */
  public isLooping(): boolean {
    return this._loop;
  }

  /**
   * Set playback rate (1.0 = normal, 2.0 = 2x speed, 0.5 = half speed).
   * Affects how quickly VGM commands are advanced per audio buffer, changing
   * both tempo and pitch proportionally.
   */
  public setPlaybackRate(rate: number): void {
    this._playbackRate = Math.max(0.25, Math.min(4.0, rate));
  }

  /**
   * Get current playback rate.
   */
  public getPlaybackRate(): number {
    return this._playbackRate;
  }

  // ========================================================================
  // Status
  // ========================================================================
  
  /**
   * Get current playback status.
   */
  public getStatus(): AudioStatus {
    return {
      isPlaying: this._isPlaying,
      isPaused: this._isPaused,
      currentTime: this._currentTime,
      duration: 0, // TODO: Calculate duration from VGM data
      sampleRate: this.sampleRate,
      chips: this.chips,
    };
  }
  
  /**
   * Check if currently playing.
   */
  public isPlaying(): boolean {
    return this._isPlaying;
  }

  /**
   * Get a live snapshot of playback diagnostics for runtime debugging.
   */
  public getRuntimeDebugInfo(): AudioRuntimeDebugInfo {
    const parsedCommandCount = this.vgmCommands.length;
    const nextCommandIndex = this.nextVgmCommandIndex;
    return {
      isPlaying: this._isPlaying,
      isPaused: this._isPaused,
      vgmDataLength: this.vgmData?.length || 0,
      parsedCommandCount,
      nextCommandIndex,
      pendingCommandCount: Math.max(0, parsedCommandCount - nextCommandIndex),
      appliedWriteCount: this.appliedWriteCount,
      skippedWriteCount: this.skippedWriteCount,
      generatedBufferCount: this.generatedBufferCount,
      lastPeak: this.lastPeak,
      silentBufferCount: this.silentBufferCount,
      emittedSilenceWarning: this.emittedSilenceWarning,
      chips: [...this.chips],
    };
  }

  /**
   * Get a copy of the most recent mono waveform samples for visualization.
   */
  public getWaveformSnapshot(length: number = 512): Float32Array {
    const targetLength = Math.max(1, Math.floor(length));
    const result = new Float32Array(targetLength);

    const available = this.waveformHasWrapped
      ? this.waveformRing.length
      : this.waveformWriteIndex;
    if (available === 0) {
      return result;
    }

    const copyCount = Math.min(targetLength, available);
    const ringSize = this.waveformRing.length;
    const start = (this.waveformWriteIndex - copyCount + ringSize) % ringSize;
    const dstOffset = targetLength - copyCount;

    for (let i = 0; i < copyCount; i++) {
      const srcIndex = (start + i) % ringSize;
      result[dstOffset + i] = this.waveformRing[srcIndex];
    }

    return result;
  }
  
  // Event Handling
  // ========================================================================
  /**
   * Check if currently paused.
   */
  public isPaused(): boolean {
    return this._isPaused;
  }
  
  // ========================================================================
  // Per-Chip Volume Control
  // ========================================================================
  
  /**
   * Set volume for a specific chip (0.0 to 1.0).
   */
  public setChipVolume(chip: SoundChip, volume: number): void {
    this.chipVolumes.set(chip, Math.max(0, Math.min(1, volume)));
    this.pushEffectiveChipGains();
  }

  /**
   * Get volume for a specific chip.
   */
  public getChipVolume(chip: SoundChip): number {
    return this.chipVolumes.get(chip) ?? 1.0;
  }

  /**
   * Set mute state for a specific chip.
   */
  public setChipMuted(chip: SoundChip, muted: boolean): void {
    this.chipMuteStates.set(chip, muted);
    this.pushEffectiveChipGains();
  }

  /**
   * Get mute state for a specific chip.
   */
  public isChipMuted(chip: SoundChip): boolean {
    return this.chipMuteStates.get(chip) || false;
  }

  /**
   * Set solo state for a specific chip.
   *
   * Solo is global across chips: if any chip is soloed, every non-soloed chip
   * is silenced. Pushing gains after every flip keeps the chip player's mixer
   * in sync.
   */
  public setChipSolo(chip: SoundChip, solo: boolean): void {
    this.chipSoloStates.set(chip, solo);
    this.pushEffectiveChipGains();
  }

  /**
   * Recompute each active chip's effective gain (mute/solo/volume combined)
   * and push it to the WASM chip player. No-op if no chip player exists.
   *
   * Safe to call from React event handlers (slider drags, button clicks) —
   * the WASM call runs on the JS main thread and is a small map update.
   */
  private pushEffectiveChipGains(): void {
    if (!this.chipPlayerId) return;
    for (const chip of this.chips) {
      const gain = this.getEffectiveChipVolume(chip);
      // Best-effort — log but don't throw if a stale chip slips through.
      wasmService.setChipGain(this.chipPlayerId, chip, gain).catch(err => {
        console.warn(`[AudioService] setChipGain failed for ${chip}:`, err);
      });
    }
  }
  
  /**
   * Get solo state for a specific chip.
   */
  public isChipSolo(chip: SoundChip): boolean {
    return this.chipSoloStates.get(chip) || false;
  }
  
  /**
   * Check if any chip is soloed.
   */
  public hasSoloChips(): boolean {
    return Array.from(this.chipSoloStates.values()).some(s => s);
  }
  
  /**
   * Get effective volume for a chip considering solo/mute.
   */
  public getEffectiveChipVolume(chip: SoundChip): number {
    const muted = this.isChipMuted(chip);
    const solo = this.isChipSolo(chip);
    const hasSolo = this.hasSoloChips();
    const volume = this.getChipVolume(chip);
    
    // If muted, volume is 0
    if (muted) return 0;
    
    // If there are soloed chips and this chip is not soloed, volume is 0
    if (hasSolo && !solo) return 0;
    
    // Otherwise, use the chip's volume
    return volume;
  }
  
  /**
   * Reset all chip volume/mute/solo states.
   */
  public resetChipStates(): void {
    this.chipVolumes.clear();
    this.chipMuteStates.clear();
    this.chipSoloStates.clear();
  }
  
  // ========================================================================
  // Event Handling
  // ================================================================================================================================================
  // Event Handling
  // ========================================================================
  
  /**
   * Add an event listener.
   */
  public addEventListener(listener: AudioEventListener): void {
    this.listeners.push(listener);
  }
  
  /**
   * Remove an event listener.
   */
  public removeEventListener(listener: AudioEventListener): void {
    this.listeners = this.listeners.filter(l => l !== listener);
  }
  
  /**
   * Clear all event listeners.
   */
  public clearEventListeners(): void {
    this.listeners = [];
  }
  
  private emitPlay(): void {
    this.listeners.forEach(l => l.onPlay?.());
  }
  
  private emitPause(): void {
    this.listeners.forEach(l => l.onPause?.());
  }
  
  private emitStop(): void {
    this.listeners.forEach(l => l.onStop?.());
  }
  
  private emitTimeUpdate(time: number): void {
    this.listeners.forEach(l => l.onTimeUpdate?.(time));
  }
  
  private emitEnd(): void {
    this.listeners.forEach(l => l.onEnd?.());
  }
  
  private emitError(error: Error): void {
    this.listeners.forEach(l => l.onError?.(error));
  }
  
  // Cleanup buffer index tracking
  private bufferIndex = 0;
  
  // ========================================================================
  // Cleanup
  // ========================================================================
  
  /**
   * Cleanup resources.
   */
  public destroy(): void {
    this.stop();
    
    if (this.animationFrameId) {
      cancelAnimationFrame(this.animationFrameId);
      this.animationFrameId = null;
    }
    
    this.listeners = [];
    this.sampleQueue = [];
    this.sampleBuffer = null;
    
    // Cleanup WASM players
    if (this.vgmPlayerId) {
      wasmService.destroyVgmPlayer(this.vgmPlayerId);
      this.vgmPlayerId = null;
    }
    
    if (this.chipPlayerId) {
      wasmService.destroyChipPlayer(this.chipPlayerId);
      this.chipPlayerId = null;
    }
    
    console.log('[AudioService] Destroyed');
  }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const audioService = new AudioService();
