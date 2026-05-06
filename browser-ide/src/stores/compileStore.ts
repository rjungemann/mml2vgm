/**
 * Compile Store
 * 
 * Manages compilation state, queue, and results.
 */

import { create } from 'zustand';
import type { CompileOptions, CompileError } from '@/types';
import { WorkerManager, getWorkerManager, resetWorkerManager, configureWorkerManager, preWarmWorkers } from '@/services/workerService';
import { wasmService } from '@/services/wasmService';

// ============================================================================
// Types
// ============================================================================

/** Compilation status */
export type CompileStatus = 'idle' | 'queued' | 'compiling' | 'success' | 'error';

export type CompilePath = 'worker' | 'fallback' | 'unknown';

export interface CompileTimingBreakdown {
    queueMs: number;
    documentLookupMs: number;
    compileMs: number;
    postProcessMs: number;
    totalMs: number;
    path: CompilePath;
}

export interface CompileTimingEntry {
    timestamp: Date;
    status: 'success' | 'error' | 'cancelled';
    summary: string;
    documentId: string | null;
    timing: CompileTimingBreakdown;
}

/** A compile request in the queue */
interface CompileRequest {
    id: string;
    documentId: string;
    options: CompileOptions;
    timestamp: Date;
    status: CompileStatus;
    resolve: (result: StoreCompileResult) => void;
    reject: (error: Error) => void;
}

/** Result of a compilation (store-specific) */
export interface StoreCompileResult {
    documentId: string;
    data?: Uint8Array;
    errors: CompileError[];
    warnings: CompileError[];
    duration: number; // in milliseconds
    timestamp: Date;
    // Metadata from compilation
    partCount: number;
    commandCount: number;
    durationSamples: number;
    durationSeconds: number;
    chipsUsed: string[];
    timing?: CompileTimingBreakdown;
}

interface CompileState {
    // Current compilation status
    status: CompileStatus;

    // Current document being compiled
    currentDocumentId: string | null;

    // Compilation results
    results: Map<string, StoreCompileResult>;

    // Compile queue
    queue: CompileRequest[];

    // Last compilation time
    lastCompileTime: Date | null;

    // Progress (0-100)
    progress: number;

    // Current progress message
    progressMessage: string;

    // Last compile timing details for UI diagnostics
    lastCompileTiming: CompileTimingBreakdown | null;

    // Human-readable last compile timing summary
    lastCompileTimingSummary: string;

    // Recent compile timings (newest first)
    compileTimingHistory: CompileTimingEntry[];

    // Web Worker settings
    useWebWorkers: boolean;
    workerManager: WorkerManager | null;
}

interface CompileActions {
    // Enqueue a compile request
    compile: (documentId: string, options: CompileOptions) => Promise<StoreCompileResult>;

    // Cancel current compilation
    cancel: () => void;

    // Clear results
    clearResults: () => void;

    // Clear result for a specific document
    clearResult: (documentId: string) => void;

    // Get result for a document
    getResult: (documentId: string) => StoreCompileResult | undefined;

    // Get status
    getStatus: () => CompileStatus;

    // Check if compiling
    isCompiling: () => boolean;

    // Check if queue has items
    hasQueue: () => boolean;

    // Update progress
    setProgress: (progress: number, message?: string) => void;

    // Initialize WorkerManager
    initWorkerManager: () => Promise<void>;

    // Internal: Process the compile queue (called automatically)
    processQueue: () => Promise<void>;
}

type CompileStore = CompileState & CompileActions;

// ============================================================================
// Store Definition
// ============================================================================

const initialState: CompileState = {
    status: 'idle',
    currentDocumentId: null,
    results: new Map(),
    queue: [],
    lastCompileTime: null,
    progress: 0,
    progressMessage: '',
    lastCompileTiming: null,
    lastCompileTimingSummary: '',
    compileTimingHistory: [],
    useWebWorkers: WorkerManager.isSupported(),
    workerManager: null,
};

const normalizeMs = (value: number): number => Math.max(0, Math.round(value));
const MAX_TIMING_HISTORY = 20;

const pushTimingHistory = (history: CompileTimingEntry[], entry: CompileTimingEntry): CompileTimingEntry[] => {
    return [entry, ...history].slice(0, MAX_TIMING_HISTORY);
};

const formatTimingSummary = (timing: CompileTimingBreakdown): string => {
    return `${timing.path} total ${timing.totalMs}ms (queue ${timing.queueMs}ms, lookup ${timing.documentLookupMs}ms, compile ${timing.compileMs}ms, post ${timing.postProcessMs}ms)`;
};

export const useCompileStore = create<CompileStore>()(
    (set, get) => ({
        ...initialState,

        // ============================================================
        // Actions
        // ============================================================

        compile: async (documentId: string, options: CompileOptions) => {
            console.log('[compileStore] compile() called with documentId:', documentId);
            console.log('[compileStore] compile() options:', JSON.stringify(options, null, 2));

            // Initialize worker manager if needed
            const state = get();

            console.log(`[compileStore] useWebWorkers: ${state.useWebWorkers}, has workerManager: ${!!state.workerManager}`);

            // If we don't have a worker manager yet, initialize it
            if (state.useWebWorkers && !state.workerManager) {
                console.log('[compileStore] Initializing worker manager...');
                try {
                    await get().initWorkerManager();
                    console.log('[compileStore] Worker manager initialized successfully');
                } catch (err) {
                    console.error('[compileStore] Failed to initialize worker manager:', err);
                }
            }

            return new Promise<StoreCompileResult>((resolve, reject) => {
                const state = get();
                const requestId = `compile-${Date.now()}-${Math.random().toString(36).substr(2, 4)}`;

                console.log(`[compileStore] Creating compile request: ${requestId}`);

                const request: CompileRequest = {
                    id: requestId,
                    documentId,
                    options,
                    timestamp: new Date(),
                    status: 'queued',
                    resolve,
                    reject,
                };

                // Add to queue
                set({
                    queue: [...state.queue, request],
                    progress: 0,
                    progressMessage: 'Queued for compilation',
                });

                console.log(`[compileStore] Request added to queue, queue length: ${state.queue.length + 1}`);

                // Process queue
                get().processQueue();
            });
        },

        initWorkerManager: async () => {
            const state = get();
            if (!state.useWebWorkers || state.workerManager) return;
            try {
                console.log('[compileStore] Initializing WorkerManager...');

                // Get the existing pre-warmed manager (created during app init)
                const manager = await getWorkerManager();

                // Set fallback to wasmService.compile for graceful degradation
                manager.setFallbackCompile((mml: string, options: CompileOptions) => {
                    console.log('[compileStore] Using fallback to wasmService.compile');
                    return wasmService.compile(mml, options);
                });

                // Set progress callback
                manager.setProgressCallback((progress, message) => {
                    get().setProgress(progress, message);
                });

                set({ workerManager: manager });
                console.log('[compileStore] WorkerManager ready, status:', manager.getStatus());
            } catch (error) {
                console.warn('[compileStore] Failed to initialize WorkerManager:', error);
                set({ useWebWorkers: false });
            }
        },
        
        /**
         * Pre-warm workers on app load
         */
        preWarmWorkers: async () => {
            await get().initWorkerManager();
        },

        cancel: () => {
            const state = get();

            // Cancel current compilation
            if (state.status === 'compiling' && state.queue.length > 0) {
                const current = state.queue[0];
                current.status = 'error';
                current.reject(new Error('Compilation cancelled'));

                // Cancel active worker compile task if running in worker mode.
                if (state.workerManager) {
                    void state.workerManager.cancelActiveCompilation('Compilation cancelled');
                }

                set({
                    status: 'idle',
                    currentDocumentId: null,
                    queue: state.queue.slice(1),
                    progress: 0,
                    progressMessage: '',
                });
            }
        },

        clearResults: () => {
            set({ results: new Map() });
        },

        clearResult: (documentId: string) => {
            const state = get();
            const results = new Map(state.results);
            results.delete(documentId);
            set({ results });
        },

        getResult: (documentId: string) => {
            const state = get();
            return state.results.get(documentId);
        },

        getStatus: () => {
            const state = get();
            return state.status;
        },

        isCompiling: () => {
            const state = get();
            return state.status === 'compiling' || state.status === 'queued';
        },

        hasQueue: () => {
            const state = get();
            return state.queue.length > 0;
        },

        setProgress: (progress: number, message?: string) => {
            set({
                progress,
                progressMessage: message || '',
            });
        },

        // ============================================================
        // Internal Methods
        // ============================================================

        processQueue: async () => {
            const state = get();

            console.log('[compileStore] processQueue called');

            // Don't process if already compiling
            if (state.status === 'compiling') {
                console.log('[compileStore] Already compiling, returning');
                return;
            }

            // Don't process if queue is empty
            if (state.queue.length === 0) {
                console.log('[compileStore] Queue is empty');
                set({ status: 'idle', currentDocumentId: null });
                return;
            }

            console.log(`[compileStore] Queue length: ${state.queue.length}, useWebWorkers: ${state.useWebWorkers}, has workerManager: ${!!state.workerManager}`);

            // Get the next request
            const request = state.queue[0];
            const dequeueTimeMs = Date.now();
            const queueMs = normalizeMs(dequeueTimeMs - request.timestamp.getTime());
            const totalStartMs = Date.now();

            // Update status
            set({
                status: 'compiling',
                currentDocumentId: request.documentId,
                queue: state.queue.map((r, i) =>
                    i === 0 ? { ...r, status: 'compiling' } : r
                ),
                progress: 0,
                progressMessage: `Starting compilation (queued ${queueMs}ms)...`,
            });

            try {
                // Get the document from the document store
                const documentLookupStartMs = Date.now();
                const { useDocumentStore } = await import('@/stores/documentStore');
                const documentStore = useDocumentStore.getState();
                const doc = documentStore.getDocument(request.documentId);
                const documentLookupMs = normalizeMs(Date.now() - documentLookupStartMs);

                if (!doc) {
                    throw new Error(`Document not found: ${request.documentId}`);
                }

                // Update progress
                get().setProgress(10, `Document ready (${documentLookupMs}ms)`);

                // Try to use WorkerManager if available, otherwise fall back to wasmService
                const currentState = get();
                let result;
                const startTime = Date.now();
                let compilePath: CompilePath = 'unknown';

                console.log(`[compileStore] Deciding compilation path: useWebWorkers=${currentState.useWebWorkers}, hasWorkerManager=${!!currentState.workerManager}`);

                const compileStartMs = Date.now();
                if (currentState.useWebWorkers && currentState.workerManager) {
                    compilePath = 'worker';
                    get().setProgress(20, 'Compiling via worker...');
                    console.log('[compileStore] Using WorkerManager for compilation');
                    result = await currentState.workerManager.compile(doc.content, request.options);
                } else {
                    compilePath = 'fallback';
                    get().setProgress(20, 'Compiling on main thread fallback...');
                    console.log('[compileStore] Using wasmService fallback for compilation');
                    const { wasmService } = await import('@/services/wasmService');
                    result = await wasmService.compile(doc.content, request.options);
                }
                const duration = Date.now() - startTime;
                const compileMs = normalizeMs(Date.now() - compileStartMs);
                const postProcessStartMs = Date.now();

                get().setProgress(80, 'Processing compile output...');

                // Extract metadata from compile result
                const info = result.info || {
                    part_count: 0,
                    command_count: 0,
                    duration_samples: 0,
                    duration_seconds: 0,
                    chips_used: [],
                    format_version: '',
                };

                const dataLength = result.data?.length || 0;
                const partCount = info.part_count || 0;
                const commandCount = info.command_count || 0;
                const metadataChipsUsed = (info.chips_used || []).map(c => c as string);
                const optionChips = Array.isArray(request.options?.target_chips)
                    ? request.options.target_chips.map((chip) => String(chip).toUpperCase())
                    : [];
                const chipsUsed = metadataChipsUsed.length > 0 ? metadataChipsUsed : optionChips;
                const hasPlayableMetadata = partCount > 0 || commandCount > 0 || metadataChipsUsed.length > 0;

                if (dataLength === 0) {
                    throw new Error(
                        'Compilation produced empty binary output. Check MML source and compile options.'
                    );
                }

                const warnings: CompileError[] = [];
                if (!hasPlayableMetadata) {
                    warnings.push({
                        type: 'warning',
                        message: 'Compilation metadata is empty (parts/commands/chips). Proceeding with binary output and fallback chips.',
                        line: 1,
                        column: 1,
                        length: 1,
                        severity: 'warning',
                        code: 'EMPTY_METADATA',
                    });
                }

                // Create compile result with metadata
                const compileResult: StoreCompileResult = {
                    documentId: request.documentId,
                    data: result.data,
                    errors: [],
                    warnings,
                    duration,
                    timestamp: new Date(),
                    partCount,
                    commandCount,
                    durationSamples: info.duration_samples || 0,
                    durationSeconds: info.duration_seconds || 0,
                    chipsUsed,
                };

                // Store result
                const results = new Map(state.results);
                results.set(request.documentId, compileResult);

                console.log('[compileStore] Storing compilation result:', {
                    documentId: request.documentId,
                    duration: compileResult.duration,
                    dataLength: compileResult.data?.length || 0,
                    partCount: compileResult.partCount,
                    commandCount: compileResult.commandCount,
                    chipsUsed: compileResult.chipsUsed,
                });

                // Update document store with results
                documentStore.setCompileResults(
                    request.documentId,
                    true,
                    []
                );

                const postProcessMs = normalizeMs(Date.now() - postProcessStartMs);
                const totalMs = normalizeMs(Date.now() - totalStartMs);
                const timing: CompileTimingBreakdown = {
                    queueMs,
                    documentLookupMs,
                    compileMs,
                    postProcessMs,
                    totalMs,
                    path: compilePath,
                };
                const timingSummary = formatTimingSummary(timing);
                compileResult.timing = timing;
                const timingEntry: CompileTimingEntry = {
                    timestamp: new Date(),
                    status: 'success',
                    summary: timingSummary,
                    documentId: request.documentId,
                    timing,
                };

                // Remove from queue and update status
                set({
                    status: 'success',
                    currentDocumentId: null,
                    queue: state.queue.slice(1),
                    results,
                    lastCompileTime: new Date(),
                    progress: 100,
                    progressMessage: `Compilation complete (${timingSummary})`,
                    lastCompileTiming: timing,
                    lastCompileTimingSummary: timingSummary,
                    compileTimingHistory: pushTimingHistory(get().compileTimingHistory, timingEntry),
                });

                // Resolve the promise
                request.resolve(compileResult);
                console.log('[compileStore] Resolved compilation promise');

                // Process next in queue
                get().processQueue();

            } catch (error) {
                const errorMessage = (error as Error).message || 'Unknown compilation error';
                const isCancelled = /cancel/i.test(errorMessage);
                const failureTiming: CompileTimingBreakdown = {
                    queueMs,
                    documentLookupMs: 0,
                    compileMs: 0,
                    postProcessMs: 0,
                    totalMs: normalizeMs(Date.now() - totalStartMs),
                    path: 'unknown',
                };
                const failureSummary = `failed in ${failureTiming.totalMs}ms (${errorMessage})`;
                const failureEntry: CompileTimingEntry = {
                    timestamp: new Date(),
                    status: isCancelled ? 'cancelled' : 'error',
                    summary: failureSummary,
                    documentId: request.documentId,
                    timing: failureTiming,
                };

                // Update status
                set({
                    status: isCancelled ? 'idle' : 'error',
                    currentDocumentId: null,
                    queue: state.queue.slice(1),
                    progress: 0,
                    progressMessage: '',
                    lastCompileTiming: failureTiming,
                    lastCompileTimingSummary: failureSummary,
                    compileTimingHistory: pushTimingHistory(get().compileTimingHistory, failureEntry),
                });

                // Update document store with error
                if (!isCancelled && request.documentId) {
                    const { useDocumentStore } = await import('@/stores/documentStore');
                    const documentStore = useDocumentStore.getState();
                    documentStore.setCompileResults(
                        request.documentId,
                        false,
                        [{ 
                            type: 'error',
                            message: errorMessage,
                            line: 0,
                            column: 0,
                            length: 0,
                            severity: 'error',
                        }],
                        undefined
                    );
                }

                // Reject the promise
                request.reject(error as Error);

                // Process next in queue
                get().processQueue();
            }
        },
    })
);

// ============================================================================
// Selectors
// ============================================================================

// Selector for current status
export const selectCompileStatus = (state: CompileStore) =>
    state.status;

// Selector for current document being compiled
export const selectCurrentDocumentId = (state: CompileStore) =>
    state.currentDocumentId;

// Selector for compile results
export const selectCompileResults = (state: CompileStore) =>
    state.results;

// Selector for result of a specific document
export const selectStoreCompileResult = (state: CompileStore, documentId: string) =>
    state.getResult(documentId);

// Selector for queue length
export const selectQueueLength = (state: CompileStore) =>
    state.queue.length;

// Selector for progress
export const selectProgress = (state: CompileStore) =>
    state.progress;

// Selector for progress message
export const selectProgressMessage = (state: CompileStore) =>
    state.progressMessage;

// Selector for is compiling
export const selectIsCompiling = (state: CompileStore) =>
    state.isCompiling();

// Selector for has queue
export const selectHasQueue = (state: CompileStore) =>
    state.hasQueue();
