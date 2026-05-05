/**
 * Web Worker for WASM Compilation
 *
 * This worker runs mml2vgm WASM compilation in a background thread
 * to prevent blocking the main UI thread.
 *
 * This file is bundled by Vite with vite-plugin-wasm support.
 */

console.log('[Worker] Worker script started loading');

// Import the WASM wrapper
import { initializeWasm, compileMml, extractResultData } from './wasmWrapper';

// Worker state
let isInitialized = false;

// Message types
interface InitMessage {
  type: 'INIT';
  requestId: string;
}

interface CompileMessage {
  type: 'COMPILE';
  requestId: string;
  mml: string;
  options: any;
}

interface CancelMessage {
  type: 'CANCEL';
  requestId: string;
}

interface TerminateMessage {
  type: 'TERMINATE';
}

interface ReadyMessage {
  type: 'READY';
  requestId: string;
}

interface ProgressMessage {
  type: 'PROGRESS';
  requestId: string;
  progress: number;
  message: string;
}

interface ResultMessage {
  type: 'RESULT';
  requestId: string;
  result: any;
}

interface ErrorMessage {
  type: 'ERROR';
  requestId: string;
  error: string;
}

type WorkerMessage = InitMessage | CompileMessage | CancelMessage | TerminateMessage;
type MainThreadMessage = ReadyMessage | ProgressMessage | ResultMessage | ErrorMessage;

/**
 * Initialize the WASM module
 */
async function initWasm(): Promise<void> {
  if (isInitialized) return;

  try {
    console.log('[Worker] Starting WASM initialization...');
    await initializeWasm();
    console.log('[Worker] WASM module loaded successfully');
    isInitialized = true;
  } catch (error) {
    console.error('[Worker] Failed to initialize WASM:', error);
    console.error('[Worker] Error stack:', error?.stack);
    console.error('[Worker] Error type:', error?.constructor?.name);
    throw error;
  }
}

/**
 * Handle a compile request
 */
async function handleCompile(requestId: string, mml: string, options: any): Promise<void> {
  try {
    console.log(`[Worker] Starting compile for request: ${requestId}`);

    // Post progress update
    postMessageToMain({
      type: 'PROGRESS',
      requestId,
      progress: 10,
      message: 'Starting compilation...'
    });

    // Convert options to JSON string
    const optionsJson = JSON.stringify(options);
    console.log(`[Worker] Options JSON: ${optionsJson.substring(0, 100)}...`);

    // Call the WASM compile function
    // Wrap in Promise to allow timeout
    console.log(`[Worker] Calling compileMml...`);
    const compilationPromise = Promise.resolve().then(() => {
      console.log(`[Worker] Starting synchronous compile_mml call...`);
      const startTime = performance.now();
      try {
        const result: any = compileMml(mml, optionsJson);
        const endTime = performance.now();
        console.log(`[Worker] compile_mml returned after ${endTime - startTime}ms`);
        return result;
      } catch (e) {
        console.error(`[Worker] compile_mml threw error:`, e);
        throw e;
      }
    });

    // Add a timeout to compilation
    const timeoutPromise = new Promise((_, reject) => {
      setTimeout(() => reject(new Error('Compilation timeout (60s)')), 60000);
    });

    const result: any = await Promise.race([compilationPromise, timeoutPromise]);
    console.log(`[Worker] Compilation completed successfully`);

    // Extract data from result using the wrapper
    const resultData = extractResultData(result);
    console.log(`[Worker] Result data extraction completed`);
    console.log(`[Worker] Result data length:`, resultData.data.length);
    console.log(`[Worker] Result part_count:`, resultData.partCount);
    console.log(`[Worker] Result command_count:`, resultData.commandCount);

    // Extract data from result
    const compileResult: any = {
      data: resultData.data,
      errors: result.errors || [],
      warnings: result.warnings || [],
      info: {
        part_count: resultData.partCount,
        command_count: resultData.commandCount,
        duration_samples: resultData.durationSamples,
        duration_seconds: resultData.durationSeconds,
        chips_used: resultData.chipsUsed ? JSON.parse(resultData.chipsUsed) : [],
        format_version: result.format_version || '',
      },
    };

    // Post result
    console.log(`[Worker] Posting RESULT for request: ${requestId}`);
    postMessageToMain({
      type: 'RESULT',
      requestId,
      result: compileResult
    });
    console.log(`[Worker] Result posted successfully for request: ${requestId}`);

  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error);
    console.error(`[Worker] Compilation error:`, errorMsg);
    console.error(`[Worker] Error details:`, error);
    console.log(`[Worker] Posting ERROR message for request: ${requestId}`);
    postMessageToMain({
      type: 'ERROR',
      requestId,
      error: errorMsg
    });
  }
}

/**
 * Post message to main thread
 */
function postMessageToMain(message: MainThreadMessage): void {
  (self as any).postMessage(message);
}

/**
 * Main message handler
 */
self.onmessage = async (e: MessageEvent) => {
  const message: WorkerMessage = e.data;
  console.log(`[Worker] onmessage called with type: ${message.type}`);

  switch (message.type) {
    case 'INIT':
      console.log(`[Worker] Processing INIT message: ${message.requestId}`);
      try {
        console.log('[Worker] About to call initWasm()');
        await initWasm();
        console.log('[Worker] initWasm() completed successfully');
        postMessageToMain({
          type: 'READY',
          requestId: message.requestId
        });
        console.log('[Worker] READY message posted');
      } catch (error) {
        console.error('[Worker] INIT handler caught error:', error);
        postMessageToMain({
          type: 'ERROR',
          requestId: message.requestId,
          error: error instanceof Error ? error.message : String(error)
        });
      }
      break;

    case 'COMPILE':
      console.log(`[Worker] Received COMPILE request: ${message.requestId}`);
      console.log(`[Worker] MML length: ${message.mml.length}, options keys: ${Object.keys(message.options).join(',')}`);
      try {
        await handleCompile(message.requestId, message.mml, message.options);
        console.log(`[Worker] COMPILE request ${message.requestId} completed`);
      } catch (err) {
        console.error(`[Worker] COMPILE request ${message.requestId} failed:`, err);
      }
      break;

    case 'CANCEL':
      // Cancellation is handled by the main thread's request tracking
      // For now, we just acknowledge
      postMessageToMain({
        type: 'PROGRESS',
        requestId: message.requestId,
        progress: 0,
        message: 'Cancelled'
      });
      break;

    case 'TERMINATE':
      // Clean up and close the worker
      isInitialized = false;
      self.close();
      break;
  }
};

// Handle errors globally
self.onerror = (error: ErrorEvent) => {
  console.error('[Worker] Global error:', error);
  console.error('[Worker] Error message:', error.message);
  console.error('[Worker] Error filename:', error.filename);
  console.error('[Worker] Error lineno:', error.lineno);
  console.error('[Worker] Error colno:', error.colno);
};

console.log('[Worker] Worker initialization complete, ready to receive messages');
