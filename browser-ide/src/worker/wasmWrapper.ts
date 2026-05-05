/**
 * WASM Wrapper for Worker Context
 *
 * Provides a wrapper around mml2vgm-wasm to properly handle WASM initialization
 * and result data extraction in Web Worker context.
 */

let wasmModule: any = null;
let isInitialized = false;

/**
 * Initialize the WASM module in the worker context
 */
export async function initializeWasm(): Promise<void> {
  if (isInitialized) return;

  try {
    console.log('[WasmWrapper] Initializing WASM module...');

    // Import the WASM module - this triggers the glue code initialization
    const imported = await import('mml2vgm-wasm');

    // The imported module should have all the exported functions and classes
    wasmModule = imported;

    console.log('[WasmWrapper] WASM module initialized');
    const allKeys = Object.keys(imported);
    console.log('[WasmWrapper] Total exports:', allKeys.length);
    console.log('[WasmWrapper] All exports:', allKeys);
    console.log('[WasmWrapper] Has compile_mml:', 'compile_mml' in imported);

    isInitialized = true;
  } catch (error) {
    console.error('[WasmWrapper] Failed to initialize WASM:', error);
    throw error;
  }
}

/**
 * Compile MML source code using WASM
 */
export function compileMml(mml: string, optionsJson: string): any {
  if (!wasmModule) {
    throw new Error('WASM module not initialized');
  }

  console.log('[WasmWrapper] About to call compile_mml with:', {
    mmlLength: mml.length,
    optionsLength: optionsJson.length,
  });

  if (!wasmModule.compile_mml) {
    throw new Error('compile_mml function not found in WASM module');
  }

  console.log('[WasmWrapper] compile_mml function exists, calling it...');

  try {
    const startTime = performance.now();
    console.log('[WasmWrapper] Calling compile_mml at', new Date().toISOString());

    // Call the function
    const result = wasmModule.compile_mml(mml, optionsJson);

    const duration = performance.now() - startTime;
    console.log(`[WasmWrapper] compile_mml returned after ${duration.toFixed(2)}ms at ${new Date().toISOString()}`);
    console.log('[WasmWrapper] Result type:', typeof result);
    console.log('[WasmWrapper] Result constructor:', result?.constructor?.name);

    return result;
  } catch (error) {
    console.error('[WasmWrapper] compile_mml threw an error:', error);
    console.error('[WasmWrapper] Error details:', {
      message: (error as Error)?.message,
      stack: (error as Error)?.stack,
    });
    throw error;
  }
}

/**
 * Extract data from a compilation result
 *
 * The result object is a JsCompileResult WASM class instance.
 * This function safely extracts the data by calling the getter methods.
 */
export function extractResultData(result: any): {
  data: Uint8Array;
  partCount: number;
  commandCount: number;
  durationSamples: bigint;
  durationSeconds: number;
  chipsUsed: string;
} {
  if (!result) {
    throw new Error('Result is null or undefined');
  }

  console.log('[WasmWrapper] Extracting result data...');
  console.log('[WasmWrapper] Result type:', typeof result);
  console.log('[WasmWrapper] Result constructor:', result.constructor?.name);

  // Call the getter methods on the result object
  let data: Uint8Array;
  let partCount: number;
  let commandCount: number;
  let durationSamples: bigint;
  let durationSeconds: number;
  let chipsUsed: string;

  try {
    // These should be methods on the JsCompileResult instance
    console.log('[WasmWrapper] Calling result.get_data()...');
    const dataArray = result.get_data?.();
    data = new Uint8Array(dataArray || []);
    console.log('[WasmWrapper] get_data() succeeded, length:', data.length);
  } catch (e) {
    console.error('[WasmWrapper] get_data() failed:', e);
    console.log('[WasmWrapper] Attempting alternative access...');
    data = new Uint8Array([]);
  }

  try {
    console.log('[WasmWrapper] Calling result.part_count()...');
    partCount = result.part_count?.() ?? 0;
    console.log('[WasmWrapper] part_count():', partCount);
  } catch (e) {
    console.error('[WasmWrapper] part_count() failed:', e);
    partCount = 0;
  }

  try {
    console.log('[WasmWrapper] Calling result.command_count()...');
    commandCount = result.command_count?.() ?? 0;
    console.log('[WasmWrapper] command_count():', commandCount);
  } catch (e) {
    console.error('[WasmWrapper] command_count() failed:', e);
    commandCount = 0;
  }

  try {
    console.log('[WasmWrapper] Calling result.duration_samples()...');
    durationSamples = result.duration_samples?.() ?? 0n;
    console.log('[WasmWrapper] duration_samples():', durationSamples);
  } catch (e) {
    console.error('[WasmWrapper] duration_samples() failed:', e);
    durationSamples = 0n;
  }

  try {
    console.log('[WasmWrapper] Calling result.duration_seconds()...');
    durationSeconds = result.duration_seconds?.() ?? 0;
    console.log('[WasmWrapper] duration_seconds():', durationSeconds);
  } catch (e) {
    console.error('[WasmWrapper] duration_seconds() failed:', e);
    durationSeconds = 0;
  }

  try {
    console.log('[WasmWrapper] Calling result.chips_used()...');
    chipsUsed = result.chips_used?.() ?? '[]';
    console.log('[WasmWrapper] chips_used():', chipsUsed.substring(0, 50));
  } catch (e) {
    console.error('[WasmWrapper] chips_used() failed:', e);
    chipsUsed = '[]';
  }

  console.log('[WasmWrapper] Result extraction completed');

  return {
    data,
    partCount,
    commandCount,
    durationSamples,
    durationSeconds,
    chipsUsed,
  };
}

/**
 * Get the WASM module (for advanced usage)
 */
export function getWasmModule(): any {
  if (!wasmModule) {
    throw new Error('WASM module not initialized');
  }
  return wasmModule;
}
