/**
 * Trace Service
 * 
 * Manages real-time trace playback for MML debugging and visualization.
 * Tracks current playback position, active parts, register changes, etc.
 */

import { audioService } from './audioService';
import { wasmService } from './wasmService';
import type { Position, PartInfo, TraceEvent } from '@/types';
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
  } | null = null;
  
  // Part tracking
  private _activeParts: Set<string> = new Set();
  
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
  }): void {
    this._lastCompileResult = {
      data: compileResult.data,
      partCount: compileResult.partCount,
      duration: compileResult.duration,
      timingMap: compileResult.timingMap || new Map(),
    };
    
    console.log('[TraceService] Initialized with', compileResult.partCount, 'parts');
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
    
    // Update current position from timing map
    if (this._lastCompileResult?.timingMap) {
      this.updatePosition(time);
    }
    
    // Update active parts
    this.updateActiveParts(time);
    
    // Emit position update
    if (this._currentPosition) {
      this.emitPositionUpdate(this._currentPosition, time);
    }
  }
  
  /**
   * Update current position based on time.
   */
  private updatePosition(time: number): void {
    if (!this._lastCompileResult?.timingMap) return;
    
    const timingMap = this._lastCompileResult.timingMap;
    
    // Find the closest position before or at current time
    let closestTime = 0;
    let closestPosition: Position | null = null;
    
    for (const [t, pos] of timingMap) {
      if (t <= time && t > closestTime) {
        closestTime = t;
        closestPosition = pos;
      }
    }
    
    this._currentPosition = closestPosition;
  }
  
  /**
   * Update active parts based on time.
   */
  private updateActiveParts(time: number): void {
    const partCount = this._lastCompileResult?.partCount || 0;
    
    if (partCount === 0) {
      this._activeParts = new Set();
      return;
    }
    
    // Cycle through parts based on time
    // Each part is active for approximately 1 second (1000ms)
    // This is a simple simulation until we have real part timing data
    const partsPerSecond = 1; // How many parts to cycle through per second
    const activePartIndex = Math.floor((time / 1000) * partsPerSecond) % partCount;
    
    // Build set of active part IDs
    const activeParts = new Set<string>();
    activeParts.add(`part-${activePartIndex}`);
    
    // Also include adjacent parts for more visual interest
    if (partCount > 1) {
      const prevIndex = (activePartIndex + partCount - 1) % partCount;
      const nextIndex = (activePartIndex + 1) % partCount;
      activeParts.add(`part-${prevIndex}`);
      activeParts.add(`part-${nextIndex}`);
    }
    
    this._activeParts = activeParts;
    
    // Emit part events for active parts
    this._activeParts.forEach(partId => {
      const index = parseInt(partId.replace('part-', ''));
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
