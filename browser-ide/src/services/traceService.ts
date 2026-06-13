/**
 * Trace Service
 * 
 * Manages real-time trace playback for MML debugging and visualization.
 * Tracks current playback position, active parts, register changes, etc.
 */

import { audioService } from './audioService';
import { wasmService } from './wasmService';
import type { Position, PartInfo, TraceEvent, SourceMap, SourceMapEvent } from '@/types';
import type { AudioEventListener } from './audioService';

// ============================================================================
// Types
// ============================================================================

export interface TraceStatus {
  isTracing: boolean;
  isPaused: boolean;
  currentTime: number;
  currentPosition: Position | null;
  activeParts: Set<string>;
  recentEvents: TraceEvent[];
}

export interface TraceEventListener {
  onTraceStart?: () => void;
  onTraceStop?: () => void;
  onTracePause?: () => void;
  onTraceResume?: () => void;
  onPositionUpdate?: (position: Position, time: number) => void;
  onPartEvent?: (partIndex: number, event: string) => void;
  onRegisterWrite?: (chip: string, addr: number, data: number, time: number) => void;
  onTraceError?: (error: Error) => void;
}

// ============================================================================
// Trace Service Class
// ============================================================================

/**
 * Service for managing real-time trace playback.
 * 
 * This class provides:
 * - Position tracking in the editor
 * - Active part highlighting
 * - Register change logging
 * - Event history
 */
export class TraceService {
  private static instance: TraceService | null = null;
  
  // Trace state
  private _isTracing = false;
  private _isPaused = false;
  private _currentTime = 0;
  private _startTime = 0;
  private _pauseTime = 0;
  
  // Position tracking
  private _currentPosition: Position | null = null;
  private _lastCompileResult: {
    data?: Uint8Array;
    partCount: number;
    duration: number;
    timingMap: Map<number, Position>; // time (ms) -> position
    sourceMap?: SourceMap; // note events with timing and source positions
  } | null = null;
  
  // Part tracking
  private _activeParts: Set<string> = new Set();

  // Active note events (from source map)
  private _activeNoteEvents: SourceMapEvent[] = [];

  // Event history
  private _recentEvents: TraceEvent[] = [];
  private _maxEvents = 1000;
  
  // Audio service listener
  private audioListenerId: string | null = null;
  
  // Event listeners
  private listeners: TraceEventListener[] = [];
  
  // ========================================================================
  // Singleton
  // ========================================================================
  
  public static getInstance(): TraceService {
    if (!TraceService.instance) {
      TraceService.instance = new TraceService();
    }
    return TraceService.instance;
  }
  
  private constructor() {
    // Private constructor for singleton
  }
  
  // ========================================================================
  // Initialization
  // ========================================================================
  
  /**
   * Initialize trace service with compile result.
   */
  public init(compileResult: {
    data?: Uint8Array;
    partCount: number;
    duration: number;
    timingMap?: Map<number, Position>;
    sourceMap?: SourceMap;
  }): void {
    this._lastCompileResult = {
      data: compileResult.data,
      partCount: compileResult.partCount,
      duration: compileResult.duration,
      timingMap: compileResult.timingMap || new Map(),
      sourceMap: compileResult.sourceMap,
    };

    console.log('[TraceService] Initialized with', compileResult.partCount, 'parts and',
      compileResult.sourceMap?.events.length || 0, 'source map events');
  }
  
  /**
   * Clear trace state.
   */
  public clear(): void {
    this._isTracing = false;
    this._isPaused = false;
    this._currentTime = 0;
    this._startTime = 0;
    this._pauseTime = 0;
    this._currentPosition = null;
    this._activeParts.clear();
    this._activeNoteEvents = [];
    this._recentEvents = [];
    
    // Remove audio listener
    if (this.audioListenerId) {
      audioService.removeEventListener(this.audioListenerId);
      this.audioListenerId = null;
    }
    
    console.log('[TraceService] Cleared');
  }
  
  // ========================================================================
  // Trace Control
  // ========================================================================
  
  /**
   * Start trace playback.
   */
  public start(): void {
    if (this._isTracing) return;
    
    this.clear();
    this._isTracing = true;
    
    // Setup audio listener
    this.setupAudioListener();
    
    this.emitTraceStart();
    console.log('[TraceService] Trace started');
  }
  
  /**
   * Stop trace playback.
   */
  public stop(): void {
    if (!this._isTracing) return;
    
    this._isTracing = false;
    this._isPaused = false;
    
    // Remove audio listener
    if (this.audioListenerId) {
      audioService.removeEventListener(this.audioListenerId);
      this.audioListenerId = null;
    }
    
    this.emitTraceStop();
    console.log('[TraceService] Trace stopped');
  }
  
  /**
   * Pause trace playback.
   */
  public pause(): void {
    if (!this._isTracing || this._isPaused) return;
    
    this._isPaused = true;
    this._pauseTime = this._currentTime;
    
    this.emitTracePause();
    console.log('[TraceService] Trace paused');
  }
  
  /**
   * Resume trace playback.
   */
  public resume(): void {
    if (!this._isTracing || !this._isPaused) return;
    
    this._isPaused = false;
    this._startTime = Date.now() - this._pauseTime;
    
    this.emitTraceResume();
    console.log('[TraceService] Trace resumed');
  }
  
  /**
   * Setup audio service listener for time updates.
   */
  private setupAudioListener(): void {
    if (!this.audioListenerId) {
      const listener: AudioEventListener = {
        onPlay: () => {
          this._startTime = Date.now();
        },
        onPause: () => {
          this._pauseTime = this._currentTime;
        },
        onStop: () => {
          this.clear();
        },
        onTimeUpdate: (time: number) => {
          this.handleTimeUpdate(time);
        },
        onError: (error) => {
          this.emitTraceError(error);
        },
      };
      
      audioService.addEventListener(listener);
      this.audioListenerId = Date.now().toString(); // Simple unique ID
    }
  }
  
  /**
   * Handle time update from audio service.
   */
  private handleTimeUpdate(time: number): void {
    this._currentTime = time;

    // Update active parts AND derive the cursor position from the same scan.
    // Position must come from the source map (sample-accurate per-note
    // line/col) — the legacy `timingMap` was a uniform linear sweep across
    // source lines, which painted the editor cursor through header /
    // instrument-definition lines that never actually play.
    this.updateActiveParts(time);
    this.updatePositionFromActiveNotes(time);

    if (this._currentPosition) {
      this.emitPositionUpdate(this._currentPosition, time);
    }
  }

  /**
   * Drive the editor cursor from the source-map note events.
   *
   * Priority order:
   *   1. The earliest currently-active note (lowest `sample_start` among
   *      events with `sample_start <= currentSample < sample_end`). If
   *      multiple parts have a note ringing simultaneously we pick the
   *      one that started first — keeps the cursor steady on the
   *      "lead" voice instead of flickering between parts at every
   *      time-update.
   *   2. If nothing is currently active (a rest, or before the first
   *      note in a part with a delayed entry), fall back to the most
   *      recent note that has already started — i.e. the last visible
   *      "where we were" so the cursor doesn't snap back to line 1.
   *   3. If no note has started yet, leave the position as-is.
   *
   * The legacy `timingMap` is ignored entirely — it was a
   * lines-per-second fiction that didn't track actual playback.
   */
  private updatePositionFromActiveNotes(time: number): void {
    const sourceMap = this._lastCompileResult?.sourceMap;
    if (!sourceMap || sourceMap.events.length === 0) return;

    const currentSample = (time * 44100) / 1000;

    let activeWinner: SourceMapEvent | null = null;
    let pastWinner: SourceMapEvent | null = null;

    for (const event of sourceMap.events) {
      if (event.sample_start > currentSample) continue; // future

      if (currentSample < event.sample_end) {
        // Currently ringing.
        if (!activeWinner || event.sample_start < activeWinner.sample_start) {
          activeWinner = event;
        }
      } else {
        // Already finished — track the most recent for fallback.
        if (!pastWinner || event.sample_end > pastWinner.sample_end) {
          pastWinner = event;
        }
      }
    }

    const chosen = activeWinner ?? pastWinner;
    if (chosen) {
      this._currentPosition = {
        line: chosen.line,
        column: chosen.col_start,
      };
    }
  }
  
  /**
   * Update active parts based on time.
   * Uses the source map to find notes that are currently playing.
   */
  private updateActiveParts(time: number): void {
    const sourceMap = this._lastCompileResult?.sourceMap;

    if (sourceMap && sourceMap.events.length > 0) {
      // Convert time from milliseconds to samples (44100 Hz sample rate)
      const currentSample = (time * 44100) / 1000;

      // Find all notes currently playing
      const activeNotes = sourceMap.events.filter(
        event => event.sample_start <= currentSample && currentSample < event.sample_end
      );

      this._activeNoteEvents = activeNotes;

      // Extract unique parts from active notes
      const activeParts = new Set<string>();
      activeNotes.forEach(event => {
        activeParts.add(event.part);
      });

      this._activeParts = activeParts;
    } else {
      // Fallback to simple cycling if no source map
      const partCount = this._lastCompileResult?.partCount || 0;

      if (partCount === 0) {
        this._activeParts = new Set();
        this._activeNoteEvents = [];
        return;
      }

      // Simple simulation: cycle through parts
      const partsPerSecond = 1;
      const activePartIndex = Math.floor((time / 1000) * partsPerSecond) % partCount;

      const activeParts = new Set<string>();
      activeParts.add(`part-${activePartIndex}`);

      if (partCount > 1) {
        const prevIndex = (activePartIndex + partCount - 1) % partCount;
        const nextIndex = (activePartIndex + 1) % partCount;
        activeParts.add(`part-${prevIndex}`);
        activeParts.add(`part-${nextIndex}`);
      }

      this._activeParts = activeParts;
      this._activeNoteEvents = [];
    }

    // Emit part events for active parts
    this._activeParts.forEach(partId => {
      const index = parseInt(partId.replace('part-', '') || '0');
      this.emitPartEvent(index, 'note-on');
    });
  }
  
  // ========================================================================
  // Position Management
  // ========================================================================
  
  /**
   * Set the current position (used for manual seek).
   */
  public setPosition(position: Position): void {
    this._currentPosition = position;
    this.emitPositionUpdate(position, this._currentTime);
  }
  
  /**
   * Get the current position.
   */
  public getPosition(): Position | null {
    return this._currentPosition;
  }
  
  /**
   * Jump to a specific position.
   */
  public jumpToPosition(position: Position): void {
    this._currentPosition = position;
    
    // TODO: Find the time corresponding to this position
    // and seek the audio player to that time
    
    this.emitPositionUpdate(position, this._currentTime);
  }
  
  // ========================================================================
  // Part Management
  // ========================================================================
  
  /**
   * Get active parts.
   */
  public getActiveParts(): Set<string> {
    return new Set(this._activeParts);
  }
  
  /**
   * Check if a part is active.
   */
  public isPartActive(partId: string): boolean {
    return this._activeParts.has(partId);
  }

  /**
   * Get active note events from the source map.
   */
  public getActiveNoteEvents(): SourceMapEvent[] {
    return [...this._activeNoteEvents];
  }

  // ========================================================================
  // Register Tracking
  // ========================================================================
  
  /**
   * Add a register write event.
   */
  public addRegisterWrite(chip: string, addr: number, data: number): void {
    const event: TraceEvent = {
      type: 'register-write',
      chip,
      addr,
      data,
      time: this._currentTime,
      timestamp: new Date(),
    };
    
    this._recentEvents.push(event);
    
    // Limit event history
    if (this._recentEvents.length > this._maxEvents) {
      this._recentEvents.shift();
    }
    
    this.emitRegisterWrite(chip, addr, data, this._currentTime);
  }
  
  /**
   * Get recent events.
   */
  public getRecentEvents(): TraceEvent[] {
    return [...this._recentEvents];
  }
  
  // ========================================================================
  // Status
  // ========================================================================
  
  /**
   * Get current trace status.
   */
  public getStatus(): TraceStatus {
    return {
      isTracing: this._isTracing,
      isPaused: this._isPaused,
      currentTime: this._currentTime,
      currentPosition: this._currentPosition,
      activeParts: new Set(this._activeParts),
      recentEvents: [...this._recentEvents],
    };
  }
  
  /**
   * Check if currently tracing.
   */
  public isTracing(): boolean {
    return this._isTracing;
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
  public addEventListener(listener: TraceEventListener): void {
    this.listeners.push(listener);
  }
  
  /**
   * Remove an event listener.
   */
  public removeEventListener(listener: TraceEventListener): void {
    this.listeners = this.listeners.filter(l => l !== listener);
  }
  
  /**
   * Clear all event listeners.
   */
  public clearEventListeners(): void {
    this.listeners = [];
  }
  
  private emitTraceStart(): void {
    this.listeners.forEach(l => l.onTraceStart?.());
  }
  
  private emitTraceStop(): void {
    this.listeners.forEach(l => l.onTraceStop?.());
  }
  
  private emitTracePause(): void {
    this.listeners.forEach(l => l.onTracePause?.());
  }
  
  private emitTraceResume(): void {
    this.listeners.forEach(l => l.onTraceResume?.());
  }
  
  private emitPositionUpdate(position: Position, time: number): void {
    this.listeners.forEach(l => l.onPositionUpdate?.(position, time));
  }
  
  private emitPartEvent(partIndex: number, event: string): void {
    this.listeners.forEach(l => l.onPartEvent?.(partIndex, event));
  }
  
  private emitRegisterWrite(chip: string, addr: number, data: number, time: number): void {
    this.listeners.forEach(l => l.onRegisterWrite?.(chip, addr, data, time));
  }
  
// Singleton Export
// ============================================================================


  private emitTraceError(error: Error): void {
    this.listeners.forEach(l => l.onTraceError?.(error));
  }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const traceService = TraceService.getInstance();
