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
    
    // Store VGM data
    this.vgmData = data;
    this._volume = options.volume || 1.0;
    this._loop = options.loop || false;
    
    try {
      // For VGM playback, we use chip player with the chips specified in the VGM
      // Get chips from VGM header or use provided chips
      const chipsToUse = options.chips.length > 0 ? options.chips : ['YM2608', 'SN76489'];
      
      // Create chip player
      await this.createChipPlayer(chipsToUse, this.sampleRate);
      
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
   * Check if currently paused.
   */
  public isPaused(): boolean {
    return this._isPaused;
  }
  
  // ========================================================================
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
