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
  private sampleQueue: Float32Array[] = [];
  private isProcessing = false;
  
  // Playback state
  private _isPlaying = false;
  private _isPaused = false;
  private _startTime = 0;
  private _pauseTime = 0;
  private _currentTime = 0;
  
  // VGM player state
  private vgmPlayerId: string | null = null;
  private vgmData: Uint8Array | null = null;
  private vgmSampleRate = 44100;
  
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
   */
  private parseVgmCommands(data: Uint8Array): ParsedVgmCommand[] {
    if (!data || data.length < 0x40) {
      return [];
    }

    const commands: ParsedVgmCommand[] = [];
    const dataOffset = (data[0x34] | (data[0x35] << 8) | (data[0x36] << 16) | (data[0x37] << 24)) >>> 0;
    let offset = dataOffset === 0 ? 0x40 : (0x34 + dataOffset);
    if (offset >= data.length) {
      offset = 0x40;
    }

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
          return commands;
        case 0x67: {
          // Data block: 0x67 0x66 tt ss ss ss ss [data]
          if (offset + 6 > data.length) return commands;
          if (data[offset] !== 0x66) return commands;
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
          return commands;
      }
    }

    return commands;
  }

  /**
   * Apply pending VGM register writes up to the target sample index.
   */
  private async applyPendingVgmCommands(targetSample: number): Promise<void> {
    if (!this.chipPlayerId || this.vgmCommands.length === 0) {
      this.vgmSampleCursor = targetSample;
      return;
    }

    while (this.nextVgmCommandIndex < this.vgmCommands.length) {
      const command = this.vgmCommands[this.nextVgmCommandIndex];
      if (command.timeSamples > targetSample) {
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
      const processorCode = `
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
              this.port.postMessage({ type: 'needSamples', count: samplesNeeded * 2 });
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
              this.port.postMessage({ type: 'needSamples', count: samplesNeeded * 2 });
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

      // For VGM playback, we use chip player with the chips specified in the VGM
      // Get chips from VGM header or use provided chips
      const chipsToUse = options.chips.length > 0 ? options.chips : ['YM2608', 'SN76489'];
      
      // Create chip player
      await this.createChipPlayer(chipsToUse, this.sampleRate);

      // Parse VGM commands and reset playback cursor.
      this.vgmCommands = this.parseVgmCommands(data);
      this.nextVgmCommandIndex = 0;
      this.vgmSampleCursor = 0;
      console.log('[AudioService] Parsed VGM commands:', this.vgmCommands.length);
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
            await this.applyPendingVgmCommands(this.vgmSampleCursor + this.bufferSize);
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
        
        // Store samples for playback
        this.sampleBuffer = samples;
        this.bufferIndex = 0;
        
        // Send to audio worklet
        if (this.audioWorkletNode) {
          this.audioWorkletNode.port.postMessage({
            type: 'samples',
            samples: samples,
          });
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
    console.log(`[AudioService] Chip volume set: ${chip}=${volume}`);
  }
  
  /**
   * Get volume for a specific chip.
   */
  public getChipVolume(chip: SoundChip): number {
    return this.chipVolumes.get(chip) || 1.0;
  }
  
  /**
   * Set mute state for a specific chip.
   */
  public setChipMuted(chip: SoundChip, muted: boolean): void {
    this.chipMuteStates.set(chip, muted);
    console.log(`[AudioService] Chip muted: ${chip}=${muted}`);
  }
  
  /**
   * Get mute state for a specific chip.
   */
  public isChipMuted(chip: SoundChip): boolean {
    return this.chipMuteStates.get(chip) || false;
  }
  
  /**
   * Set solo state for a specific chip.
   */
  public setChipSolo(chip: SoundChip, solo: boolean): void {
    this.chipSoloStates.set(chip, solo);
    console.log(`[AudioService] Chip solo: ${chip}=${solo}`);
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
