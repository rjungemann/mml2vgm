/**
 * Worker Service
 * 
 * Manages a pool of Web Workers for running WASM compilation
 * in the background, preventing UI blocking.
 * 
 * Features:
 * - Configurable worker pool size
 * - Pre-warming (workers initialized on creation)
 * - Graceful degradation (fallback to main thread)
 * - Request queuing and load balancing
 * 
 * Uses Vite-bundled worker file.
 */

import type { CompileOptions, CompileResult } from '@/types';

// Message types
interface WorkerInitMessage {
  type: 'INIT';
  requestId: string;
}

interface WorkerCompileMessage {
  type: 'COMPILE';
  requestId: string;
  mml: string;
  options: CompileOptions;
  // sample name → ArrayBuffer (underlying buffer transferred, not copied)
  samples?: Record<string, ArrayBuffer>;
}

interface WorkerCancelMessage {
  type: 'CANCEL';
  requestId: string;
}

interface WorkerTerminateMessage {
  type: 'TERMINATE';
}

interface MainReadyMessage {
  type: 'READY';
  requestId: string;
}

interface MainProgressMessage {
  type: 'PROGRESS';
  requestId: string;
  progress: number;
  message: string;
}

interface MainResultMessage {
  type: 'RESULT';
  requestId: string;
  result: CompileResult;
}

interface MainErrorMessage {
  type: 'ERROR';
  requestId: string;
  error: string;
}

type MainThreadMessage = MainReadyMessage | MainProgressMessage | MainResultMessage | MainErrorMessage;

/**
 * A queued compile request
 */
interface CompileRequest {
  requestId: string;
  mml: string;
  options: CompileOptions;
  samples?: Record<string, ArrayBuffer>;
  resolve: (result: CompileResult) => void;
  reject: (error: Error) => void;
  timestamp: number;
}

interface RequestTiming {
  createdAt: number;
  startedAt?: number;
}

/**
 * Worker with its state
 */
interface WorkerEntry {
  worker: Worker;
  isBusy: boolean;
  isInitialized: boolean;
  currentRequestId: string | null;
  activeTimeoutId: ReturnType<typeof setTimeout> | null;
}

/**
 * Worker Manager Configuration
 */
export interface WorkerManagerConfig {
  /** Maximum number of workers in the pool (default: 1) */
  maxWorkers?: number;
  /** Pre-warm workers on initialization (default: true) */
  preWarm?: boolean;
  /** Enable graceful degradation to main thread (default: true) */
  enableFallback?: boolean;
}

/**
 * Worker Manager
 * 
 * Manages a pool of workers and distributes compile requests
 */
export class WorkerManager {
  private static readonly COMPILE_TIMEOUT_MS = 15000;
  private static readonly WORKER_INIT_TIMEOUT_MS = 8000;
  private static readonly FALLBACK_TIMEOUT_MS = 20000;

  private workers: WorkerEntry[] = [];
  private queue: CompileRequest[] = [];
  private maxWorkers: number;
  private nextRequestId: number = 0;
  private isTerminated: boolean = false;
  private useFallback: boolean;
  
  // Map of requestId to request for tracking
  private activeRequests: Map<string, CompileRequest> = new Map();
  private requestTimings: Map<string, RequestTiming> = new Map();
  
  // Track pending initialization
  private pendingInits: Map<string, { resolve: () => void; reject: (e: Error) => void }> = new Map();
  
  // Track worker initialization failures
  private workerInitFailures: number = 0;
  
  // Progress callback
  private progressCallback?: (progress: number, message: string) => void;
  
  // Fallback compile function (for graceful degradation)
  private fallbackCompile?: (mml: string, options: CompileOptions) => Promise<CompileResult>;
  
  // Whether workers are currently working
  private workersActive: boolean = false;
  
  constructor(config: WorkerManagerConfig = {}) {
    this.maxWorkers = config.maxWorkers ?? 1;
    this.useFallback = config.enableFallback ?? true;
  }
  
  /**
   * Initialize the worker pool
   * If preWarm is true (default), workers are created and initialized immediately
   */
  async init(preWarm: boolean = true): Promise<void> {
    if (preWarm) {
      // Create and initialize all workers upfront
      for (let i = 0; i < this.maxWorkers; i++) {
        try {
          await this.createWorker();
        } catch (error) {
          this.workerInitFailures++;
          console.warn(`[WorkerManager] Worker ${i} failed to initialize:`, error);
        }
      }
      
      // Check if we have any working workers
      this.workersActive = this.workers.some(w => w.isInitialized);
      
      if (!this.workersActive && this.useFallback) {
        console.warn('[WorkerManager] No workers initialized, will use fallback');
      }
    }
  }
  
  /**
   * Set the fallback compile function for graceful degradation
   * This should be set to wasmService.compile() if available
   */
  setFallbackCompile(fn: (mml: string, options: CompileOptions) => Promise<CompileResult>): void {
    this.fallbackCompile = fn;
  }
  
  /**
   * Create a new worker using Vite's bundled worker file
   */
  private async createWorker(): Promise<WorkerEntry> {
    console.log(`[WorkerManager] Creating worker...`);

    // Use the standard Web Worker API with Vite's URL resolution
    // This approach uses import.meta.url to resolve the worker file location
    const workerUrl = new URL('../worker/compilerWorker.ts', import.meta.url);
    console.log(`[WorkerManager] Worker URL:`, workerUrl.toString());

    // Create a worker instance with module type for ES6 import support
    const worker = new Worker(workerUrl, { type: 'module' });
    console.log(`[WorkerManager] Worker instance created`);
    
    const entry: WorkerEntry = {
      worker,
      isBusy: false,
      isInitialized: false,
      currentRequestId: null,
      activeTimeoutId: null,
    };
    
    this.setupWorkerHandlers(entry);
    this.workers.push(entry);
    
    // Initialize the worker
    try {
      await this.initializeWorker(entry);
      console.log(`[WorkerManager] Worker initialized successfully`);
    } catch (error) {
      console.warn(`[WorkerManager] Worker initialization failed:`, error);
      throw error;
    }
    
    return entry;
  }
  
  /**
   * Set up message handlers for a worker
   */
  private setupWorkerHandlers(entry: WorkerEntry): void {
    entry.worker.onmessage = (e: MessageEvent) => {
      const message: MainThreadMessage = e.data;
      console.log(`[WorkerManager] Received message from worker: ${message.type} for request: ${message.requestId}`);
      
      switch (message.type) {
        case 'READY':
          entry.isInitialized = true;
          entry.isBusy = false;
          // Resolve any pending initialization
          const pending = this.pendingInits.get(entry.worker.toString());
          if (pending) {
            pending.resolve();
            this.pendingInits.delete(entry.worker.toString());
          }
          this.workersActive = true;
          this.processQueue();
          break;
          
        case 'PROGRESS':
          console.log(`[WorkerManager] Progress: ${message.progress}% - ${message.message}`);
          this.handleProgress(message.requestId, message.progress, message.message);
          break;
          
        case 'RESULT':
          console.log(`[WorkerManager] Received RESULT for request: ${message.requestId}`);
          this.clearWorkerTimeout(entry);
          entry.isBusy = false;
          entry.currentRequestId = null;
          this.handleResult(message.requestId, message.result);
          this.processQueue();
          break;
          
        case 'ERROR':
          console.error(`[WorkerManager] Received ERROR from worker: ${message.error} for request: ${message.requestId}`);
          this.clearWorkerTimeout(entry);
          // Check if this is an initialization error
          const initPending = this.pendingInits.get(entry.worker.toString());
          if (initPending) {
            // This is an initialization error
            initPending.reject(new Error(message.error));
            this.pendingInits.delete(entry.worker.toString());
          } else {
            // This is a compile error
            entry.isBusy = false;
            entry.currentRequestId = null;
            this.handleError(message.requestId, message.error);
          }
          this.processQueue();
          break;
      }
    };
    
    entry.worker.onerror = (error: ErrorEvent) => {
      console.error('[WorkerManager] Worker error:', error);
      this.clearWorkerTimeout(entry);
      entry.isBusy = false;
      entry.currentRequestId = null;
      this.processQueue();
    };
  }

  /**
   * Clear active compile timeout for a worker, if any.
   */
  private clearWorkerTimeout(entry: WorkerEntry): void {
    if (entry.activeTimeoutId) {
      clearTimeout(entry.activeTimeoutId);
      entry.activeTimeoutId = null;
    }
  }

  /**
   * Handle compile timeout from the main thread to recover from a stuck worker.
   */
  private async handleCompileTimeout(entry: WorkerEntry, requestId: string): Promise<void> {
    console.error(`[WorkerManager] Compile timed out after ${WorkerManager.COMPILE_TIMEOUT_MS}ms for request: ${requestId}`);

    this.clearWorkerTimeout(entry);
    const timedOutRequest = this.activeRequests.get(requestId);

    this.workers = this.workers.filter(w => w !== entry);

    try {
      entry.worker.terminate();
    } catch (e) {
      console.warn('[WorkerManager] Failed to terminate timed-out worker:', e);
    }

    entry.isBusy = false;
    entry.isInitialized = false;
    entry.currentRequestId = null;

    this.workersActive = this.workers.some(w => w.isInitialized);

    // Recreate worker in the background so timeout handling does not block fallback.
    if (!this.isTerminated) {
      void this.recreateWorkerAfterTimeout();
    }

    if (timedOutRequest && this.useFallback && this.fallbackCompile) {
      try {
        console.warn(`[WorkerManager] Falling back to main-thread compile for ${requestId} after worker timeout`);
        this.handleProgress(requestId, 20, 'Worker timeout; retrying on main thread fallback...');
        const fallbackResult = await this.runFallbackWithTimeout(timedOutRequest.mml, timedOutRequest.options, requestId);
        this.handleProgress(requestId, 90, 'Main-thread fallback compile completed');
        this.handleResult(requestId, fallbackResult);
      } catch (fallbackError) {
        const message = fallbackError instanceof Error ? fallbackError.message : String(fallbackError);
        this.handleError(requestId, `Worker timeout fallback failed: ${message}`, 'timeout');
      }
    } else {
      this.handleError(
        requestId,
        `Compilation timeout (${Math.floor(WorkerManager.COMPILE_TIMEOUT_MS / 1000)}s)`,
        'timeout'
      );
    }

    this.processQueue();
  }

  private async recreateWorkerAfterTimeout(): Promise<void> {
    try {
      await this.createWorker();
      this.workersActive = this.workers.some(w => w.isInitialized);
      console.log('[WorkerManager] Replacement worker initialized after timeout');
      this.processQueue();
    } catch (e) {
      this.workersActive = this.workers.some(w => w.isInitialized);
      console.warn('[WorkerManager] Failed to recreate worker after timeout:', e);
    }
  }

  private async runFallbackWithTimeout(
    mml: string,
    options: CompileOptions,
    requestId: string
  ): Promise<CompileResult> {
    if (!this.fallbackCompile) {
      throw new Error('No fallback compile function configured');
    }

    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(() => {
        reject(new Error(`Main-thread fallback timeout (${Math.floor(WorkerManager.FALLBACK_TIMEOUT_MS / 1000)}s)`));
      }, WorkerManager.FALLBACK_TIMEOUT_MS);
    });

    const compilePromise = this.fallbackCompile(mml, options);
    console.log(`[WorkerManager] Starting main-thread fallback compile for ${requestId}`);
    return Promise.race([compilePromise, timeoutPromise]);
  }
  
  /**
   * Initialize a worker
   */
  private async initializeWorker(entry: WorkerEntry): Promise<void> {
    const requestId = `init-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`;
    const timeout = WorkerManager.WORKER_INIT_TIMEOUT_MS;

    return new Promise((resolve, reject) => {
      const timeoutHandle = setTimeout(() => {
        this.pendingInits.delete(entry.worker.toString());
        console.error(`[WorkerManager] Worker initialization timeout after ${timeout}ms - worker may not have responded to INIT message`);
        reject(new Error(`Worker initialization timeout after ${timeout}ms`));
      }, timeout);

      const pending = {
        resolve: () => {
          clearTimeout(timeoutHandle);
          resolve();
        },
        reject: (error: Error) => {
          clearTimeout(timeoutHandle);
          reject(error);
        }
      };

      this.pendingInits.set(entry.worker.toString(), pending);

      entry.isBusy = true;
      entry.worker.postMessage({
        type: 'INIT',
        requestId,
      } as WorkerInitMessage);
    });
  }
  
  /**
   * Set progress callback for all compilations
   */
  setProgressCallback(callback?: (progress: number, message: string) => void): void {
    this.progressCallback = callback;
  }

  /**
   * Normalize browser-authored MML before compilation.
   * Some legacy/sample files use ';' full-line comments that can trigger parser stalls,
   * so strip those lines while preserving all musical content.
   */
  private preprocessMml(mml: string): string {
    return mml
      .replace(/\r\n/g, '\n')
      .split('\n')
      .filter((line) => !line.trimStart().startsWith(';'))
      .join('\n');
  }
  
  /**
   * Compile MML in a worker (or fallback to main thread).
   * Pass `samples` as a Map<name, Float32Array>; the underlying ArrayBuffers
   * are transferred to the worker so large PCM data is not copied.
   */
  async compile(
    mml: string,
    options: CompileOptions,
    samples?: Map<string, Float32Array>
  ): Promise<CompileResult> {
    if (this.isTerminated) {
      throw new Error('WorkerManager has been terminated');
    }
    
    const requestId = `compile-${this.nextRequestId++}`;
    const normalizedMml = this.preprocessMml(mml);
    const status = this.getStatus();
    console.log(`[WorkerManager] Compile requested. Status:`, status);

    // Convert Map<name, Float32Array> to Record<name, ArrayBuffer> for transfer
    let samplesRecord: Record<string, ArrayBuffer> | undefined;
    if (samples && samples.size > 0) {
      samplesRecord = {};
      samples.forEach((pcm, name) => {
        // slice() copies into a new ArrayBuffer (avoids SharedArrayBuffer type issues
        // and prevents detaching the caller's buffer on transfer)
        samplesRecord![name] = pcm.slice().buffer as ArrayBuffer;
      });
    }

    // If we have active workers, try to use them
    if (this.workersActive && this.workers.length > 0) {
      console.log(`[WorkerManager] Using worker for compilation`);
      return new Promise((resolve, reject) => {
        const request: CompileRequest = {
          requestId,
          mml: normalizedMml,
          options,
          samples: samplesRecord,
          resolve,
          reject,
          timestamp: Date.now(),
        };

        this.activeRequests.set(requestId, request);
        this.requestTimings.set(requestId, { createdAt: Date.now() });
        this.queue.push(request);

        this.processQueue();
      });
    }

    // Fallback to main thread if no workers are active
    if (this.useFallback && this.fallbackCompile) {
      console.log(`[WorkerManager] Using fallback to main thread (workersActive=${this.workersActive}, workers=${this.workers.length})`);
      return this.fallbackCompile(normalizedMml, options);
    }
    
    // No fallback available - throw error
    throw new Error('No workers available and no fallback configured');
  }

  /**
   * Cancel the currently active compile request, if any.
   * Returns true when an active request was cancelled.
   */
  async cancelActiveCompilation(reason: string = 'Compilation cancelled'): Promise<boolean> {
    const activeWorker = this.workers.find(w => w.isBusy && w.currentRequestId);
    if (!activeWorker || !activeWorker.currentRequestId) {
      return false;
    }

    const activeRequestId = activeWorker.currentRequestId;
    console.warn(`[WorkerManager] Cancelling active request: ${activeRequestId}`);

    this.clearWorkerTimeout(activeWorker);
    this.handleError(activeRequestId, reason, 'cancelled');

    this.workers = this.workers.filter(w => w !== activeWorker);

    try {
      activeWorker.worker.terminate();
    } catch (e) {
      console.warn('[WorkerManager] Failed to terminate worker during cancellation:', e);
    }

    activeWorker.isBusy = false;
    activeWorker.isInitialized = false;
    activeWorker.currentRequestId = null;

    if (!this.isTerminated) {
      try {
        await this.createWorker();
      } catch (e) {
        console.warn('[WorkerManager] Failed to recreate worker after cancellation:', e);
      }
    }

    this.workersActive = this.workers.some(w => w.isInitialized);
    this.processQueue();
    return true;
  }
  
  /**
   * Process the compile queue
   */
  private processQueue(): void {
    console.log(`[WorkerManager] processQueue called. Queue length: ${this.queue.length}, Available workers: ${this.workers.filter(w => !w.isBusy && w.isInitialized).length}`);

    if (this.queue.length === 0) {
      console.log(`[WorkerManager] Queue is empty, nothing to process`);
      return;
    }

    // Find an available worker
    const availableWorker = this.workers.find(w => !w.isBusy && w.isInitialized);

    if (!availableWorker) {
      // No available workers - if we have workers that aren't initialized yet,
      // we could create more, but for now we just wait
      console.log(`[WorkerManager] No available workers found (total workers: ${this.workers.length}, initialized: ${this.workers.filter(w => w.isInitialized).length})`);
      return;
    }

    // Get the next request
    const request = this.queue.shift()!;
    const timing = this.requestTimings.get(request.requestId);
    if (timing) {
      timing.startedAt = Date.now();
      this.requestTimings.set(request.requestId, timing);
    }

    console.log(`[WorkerManager] Processing request ${request.requestId} on worker`);
    console.log(`[WorkerManager] MML length: ${request.mml.length}, Options: ${JSON.stringify(request.options)}`);

    // Mark worker as busy
    availableWorker.isBusy = true;
    availableWorker.currentRequestId = request.requestId;
    this.clearWorkerTimeout(availableWorker);
    availableWorker.activeTimeoutId = setTimeout(() => {
      void this.handleCompileTimeout(availableWorker, request.requestId);
    }, WorkerManager.COMPILE_TIMEOUT_MS);

    // Send the compile request
    console.log(`[WorkerManager] Sending COMPILE message to worker for request ${request.requestId}`);
    try {
      const msg: WorkerCompileMessage = {
        type: 'COMPILE',
        requestId: request.requestId,
        mml: request.mml,
        options: request.options,
        samples: request.samples,
      };
      // Transfer ArrayBuffers so large PCM data is zero-copy
      const transferables = request.samples
        ? Object.values(request.samples).map((ab) => ab)
        : [];
      availableWorker.worker.postMessage(msg, transferables);
      console.log(`[WorkerManager] COMPILE message sent successfully`);
    } catch (e) {
      this.clearWorkerTimeout(availableWorker);
      console.error(`[WorkerManager] Failed to send COMPILE message:`, e);
    }
  }
  
  /**
   * Handle a successful compile result
   */
  private handleResult(requestId: string, result: CompileResult): void {
    const request = this.activeRequests.get(requestId);
    if (request) {
      this.logRequestSummary(requestId, 'success');
      request.resolve(result);
      this.activeRequests.delete(requestId);
      this.requestTimings.delete(requestId);
    }
  }
  
  /**
   * Handle a compile error
   */
  private handleError(requestId: string, error: string, outcome: 'error' | 'timeout' | 'cancelled' = 'error'): void {
    const request = this.activeRequests.get(requestId);
    if (request) {
      this.logRequestSummary(requestId, outcome, error);
      request.reject(new Error(error));
      this.activeRequests.delete(requestId);
      this.requestTimings.delete(requestId);
    }
  }

  /**
   * Emit a compact one-line summary for compile requests to help performance triage.
   */
  private logRequestSummary(
    requestId: string,
    outcome: 'success' | 'error' | 'timeout' | 'cancelled',
    detail?: string
  ): void {
    const timing = this.requestTimings.get(requestId);
    const now = Date.now();

    const queuedMs = timing?.startedAt && timing?.createdAt
      ? Math.max(0, timing.startedAt - timing.createdAt)
      : 0;
    const runMs = timing?.startedAt
      ? Math.max(0, now - timing.startedAt)
      : 0;
    const totalMs = timing?.createdAt
      ? Math.max(0, now - timing.createdAt)
      : 0;

    const detailSuffix = detail ? ` detail="${detail}"` : '';
    console.log(
      `[WorkerManager] CompileSummary id=${requestId} outcome=${outcome} queuedMs=${queuedMs} runMs=${runMs} totalMs=${totalMs}${detailSuffix}`
    );
  }
  
  /**
   * Handle a progress update from worker
   */
  private handleProgress(requestId: string, progress: number, message: string): void {
    if (this.progressCallback) {
      this.progressCallback(progress, message);
    }
  }
  
  /**
   * Terminate all workers and clean up
   */
  terminateAll(): void {
    this.isTerminated = true;
    this.workersActive = false;
    
    for (const workerEntry of this.workers) {
      this.clearWorkerTimeout(workerEntry);
      workerEntry.worker.postMessage({ type: 'TERMINATE' } as WorkerTerminateMessage);
      workerEntry.worker.terminate();
    }
    
    this.workers = [];
    this.queue = [];
    this.activeRequests.clear();
    this.workerInitFailures = 0;
  }
  
  /**
   * Add a new worker to the pool (for dynamic scaling)
   */
  async addWorker(): Promise<void> {
    if (this.workers.length >= this.maxWorkers) {
      throw new Error(`Maximum workers (${this.maxWorkers}) reached`);
    }
    
    this.maxWorkers++;
    await this.createWorker();
  }
  
  /**
   * Get the number of available workers
   */
  get availableWorkers(): number {
    return this.workers.filter(w => !w.isBusy && w.isInitialized).length;
  }
  
  /**
   * Get the number of busy workers
   */
  get busyWorkers(): number {
    return this.workers.filter(w => w.isBusy).length;
  }
  
  /**
   * Get the number of queued requests
   */
  get queueLength(): number {
    return this.queue.length;
  }
  
  /**
   * Get worker manager status
   */
  getStatus(): {
    workersTotal: number;
    workersInitialized: number;
    workersBusy: number;
    queueLength: number;
    workersActive: boolean;
    usingFallback: boolean;
  } {
    return {
      workersTotal: this.workers.length,
      workersInitialized: this.workers.filter(w => w.isInitialized).length,
      workersBusy: this.busyWorkers,
      queueLength: this.queueLength,
      workersActive: this.workersActive,
      usingFallback: !this.workersActive && this.useFallback,
    };
  }
  
  /**
   * Check if Web Workers are supported
   */
  static isSupported(): boolean {
    return 'Worker' in window;
  }
}

// Singleton instance
let workerManagerInstance: WorkerManager | null = null;

/**
 * Configuration for the global worker manager
 */
export interface GlobalWorkerConfig extends WorkerManagerConfig {
  /** Auto-initialize on first access (default: true) */
  autoInit?: boolean;
}

let globalConfig: GlobalWorkerConfig = {
  maxWorkers: 1,
  preWarm: true,
  enableFallback: true,
  autoInit: true,
};

/**
 * Configure the global worker manager (call this before using getWorkerManager)
 */
export function configureWorkerManager(config: GlobalWorkerConfig): void {
  globalConfig = { ...globalConfig, ...config };
  
  // Reset singleton to apply new config
  if (workerManagerInstance) {
    workerManagerInstance.terminateAll();
    workerManagerInstance = null;
  }
}

/**
 * Get the singleton WorkerManager instance
 * Auto-initializes if autoInit is true (default)
 */
export async function getWorkerManager(): Promise<WorkerManager> {
  if (!WorkerManager.isSupported()) {
    throw new Error('Web Workers are not supported in this browser');
  }
  
  if (!workerManagerInstance) {
    workerManagerInstance = new WorkerManager({
      maxWorkers: globalConfig.maxWorkers,
      preWarm: globalConfig.preWarm,
      enableFallback: globalConfig.enableFallback,
    });
    
    if (globalConfig.autoInit) {
      await workerManagerInstance.init(globalConfig.preWarm);
    }
  }
  
  return workerManagerInstance;
}

/**
 * Get the worker manager without auto-initialization
 * Use this if you want to manually control initialization
 */
export function getWorkerManagerSync(): WorkerManager {
  if (!workerManagerInstance) {
    workerManagerInstance = new WorkerManager({
      maxWorkers: globalConfig.maxWorkers,
      enableFallback: globalConfig.enableFallback,
    });
  }
  return workerManagerInstance;
}

/**
 * Reset the singleton instance (for testing or HMR)
 */
export function resetWorkerManager(): void {
  if (workerManagerInstance) {
    workerManagerInstance.terminateAll();
    workerManagerInstance = null;
  }
}

/**
 * Pre-warm the worker manager
 * Call this early in app initialization for better UX
 */
export async function preWarmWorkers(): Promise<void> {
  try {
    const manager = await getWorkerManager();
    if (!globalConfig.preWarm) {
      await manager.init(true);
    }
  } catch (error) {
    console.warn('[WorkerManager] Pre-warming failed:', error);
  }
}
