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

    // Import the wasm-pack JS glue module
    const imported = await import('mml2vgm-wasm');

    // wasm-pack generates a default async init() that fetches and instantiates
    // the .wasm binary. Without calling it, all WASM functions fail because the
    // internal `wasm` variable (which holds WebAssembly exports) is undefined.
    if (typeof imported.default === 'function') {
      console.log('[WasmWrapper] Calling WASM init() to load binary...');
      await imported.default();
      console.log('[WasmWrapper] WASM binary loaded successfully');
    }

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
export function compileMml(mml: string, optionsJson: string, requestId: string = 'unknown'): any {
  if (!wasmModule) {
    throw new Error('WASM module not initialized');
  }

  console.log(`[WasmWrapper][${requestId}] About to call compile_mml with:`, {
    mmlLength: mml.length,
    optionsLength: optionsJson.length,
  });

  if (!wasmModule.compile_mml) {
    throw new Error('compile_mml function not found in WASM module');
  }

  console.log(`[WasmWrapper][${requestId}] compile_mml function exists, calling it...`);
  console.log(`[WasmWrapper][${requestId}] MML preview:`, mml.substring(0, 100));
  console.log(`[WasmWrapper][${requestId}] Options preview:`, optionsJson.substring(0, 100));

  try {
    const startTime = performance.now();
    console.log(`[WasmWrapper][${requestId}] Calling compile_mml at`, new Date().toISOString());

    // Call the function with error handling
    let result;
    try {
      result = wasmModule.compile_mml(mml, optionsJson);
    } catch (wasmError) {
      console.error(`[WasmWrapper][${requestId}] WASM function threw exception:`, wasmError);
      throw new Error(`WASM compile_mml exception: ${wasmError}`);
    }

    const duration = performance.now() - startTime;
    console.log(`[WasmWrapper][${requestId}] compile_mml returned after ${duration.toFixed(2)}ms at ${new Date().toISOString()}`);
    console.log(`[WasmWrapper][${requestId}] Result type:`, typeof result);
    console.log(`[WasmWrapper][${requestId}] Result constructor:`, result?.constructor?.name);
    console.log(`[WasmWrapper][${requestId}] Result is null/undefined:`, result == null);

    if (!result) {
      throw new Error('WASM compile_mml returned null or undefined');
    }

    return result;
  } catch (error) {
    console.error(`[WasmWrapper][${requestId}] compile_mml threw an error:`, error);
    console.error(`[WasmWrapper][${requestId}] Error details:`, {
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
  /** JSON-encoded SourceMap (`{events: [...]}`). Empty string if the WASM
   *  build doesn't expose it or extraction fails. */
  sourceMapJson: string;
} {
  if (!result) {
    throw new Error('Result is null or undefined');
  }

  console.log('[WasmWrapper] Extracting result data...');
  console.log('[WasmWrapper] Result type:', typeof result);
  console.log('[WasmWrapper] Result constructor:', result.constructor?.name);

  const readValue = <T>(obj: any, key: string, fallback: T): T => {
    const value = obj?.[key];
    if (typeof value === 'function') {
      try {
        return value.call(obj) as T;
      } catch (e) {
        console.warn(`[WasmWrapper] ${key}() invocation failed:`, e);
        return fallback;
      }
    }
    return (value ?? fallback) as T;
  };

  // Call the getter methods on the result object
  let data: Uint8Array;
  let partCount: number;
  let commandCount: number;
  let durationSamples: bigint;
  let durationSeconds: number;
  let chipsUsed: string;

  try {
    console.log('[WasmWrapper] Reading result data...');
    const dataArray = readValue<any>(result, 'get_data', readValue<any>(result, 'data', []));
    data = dataArray instanceof Uint8Array ? dataArray : new Uint8Array(dataArray || []);
    console.log('[WasmWrapper] data read succeeded, length:', data.length);
  } catch (e) {
    console.error('[WasmWrapper] data read failed:', e);
    data = new Uint8Array([]);
  }

  try {
    console.log('[WasmWrapper] Reading part_count...');
    partCount = Number(readValue<any>(result, 'part_count', 0)) || 0;
    console.log('[WasmWrapper] part_count():', partCount);
  } catch (e) {
    console.error('[WasmWrapper] part_count() failed:', e);
    partCount = 0;
  }

  try {
    console.log('[WasmWrapper] Reading command_count...');
    commandCount = Number(readValue<any>(result, 'command_count', 0)) || 0;
    console.log('[WasmWrapper] command_count():', commandCount);
  } catch (e) {
    console.error('[WasmWrapper] command_count() failed:', e);
    commandCount = 0;
  }

  try {
    console.log('[WasmWrapper] Reading duration_samples...');
    durationSamples = readValue<any>(result, 'duration_samples', 0n) ?? 0n;
    console.log('[WasmWrapper] duration_samples():', durationSamples);
  } catch (e) {
    console.error('[WasmWrapper] duration_samples() failed:', e);
    durationSamples = 0n;
  }

  try {
    console.log('[WasmWrapper] Reading duration_seconds...');
    durationSeconds = Number(readValue<any>(result, 'duration_seconds', 0)) || 0;
    console.log('[WasmWrapper] duration_seconds():', durationSeconds);
  } catch (e) {
    console.error('[WasmWrapper] duration_seconds() failed:', e);
    durationSeconds = 0;
  }

  try {
    console.log('[WasmWrapper] Reading chips_used...');
    const rawChips = readValue<any>(result, 'chips_used', []);
    if (typeof rawChips === 'string') {
      chipsUsed = rawChips;
    } else if (Array.isArray(rawChips)) {
      chipsUsed = JSON.stringify(rawChips);
    } else {
      chipsUsed = '[]';
    }
    console.log('[WasmWrapper] chips_used():', chipsUsed.substring(0, 50));
  } catch (e) {
    console.error('[WasmWrapper] chips_used() failed:', e);
    chipsUsed = '[]';
  }

  // Source map: a JSON-encoded `{events: SourceMapEvent[]}` produced by the
  // Rust codegen. Old WASM builds didn't expose this getter; in that case
  // we drop to an empty string and the trace service treats the document as
  // sourcemap-less (highlighting just doesn't fire).
  let sourceMapJson: string;
  try {
    const raw = readValue<any>(result, 'source_map_json', '');
    sourceMapJson = typeof raw === 'string' ? raw : '';
    console.log('[WasmWrapper] source_map_json length:', sourceMapJson.length);
  } catch (e) {
    console.error('[WasmWrapper] source_map_json() failed:', e);
    sourceMapJson = '';
  }

  console.log('[WasmWrapper] Result extraction completed');

  return {
    data,
    partCount,
    commandCount,
    durationSamples,
    durationSeconds,
    chipsUsed,
    sourceMapJson,
  };
}

/**
 * Compile MML with pre-decoded PCM samples.
 *
 * If the WASM build exports `compile_with_samples`, samples are passed via a
 * js_sys::Map so large Float32Arrays are not JSON-serialised.  Otherwise falls
 * back to `compileMml` and surfaces a warning for each referenced sample that
 * could not be embedded.
 */
export function compileMmlWithSamples(
  mml: string,
  optionsJson: string,
  samples: Map<string, Float32Array>,
  requestId: string = 'unknown'
): any {
  if (!wasmModule) {
    throw new Error('WASM module not initialized');
  }

  // If the WASM export is available, use it
  if (typeof wasmModule.compile_with_samples === 'function') {
    console.log(`[WasmWrapper][${requestId}] Calling compile_with_samples with ${samples.size} sample(s)`);
    // Rust side expects { "filename.wav": [f32, f32, ...] }.
    // Float32Array does NOT JSON-stringify as an array by default — convert explicitly.
    const samplesObj: Record<string, number[]> = {};
    samples.forEach((pcm, name) => { samplesObj[name] = Array.from(pcm); });
    return wasmModule.compile_with_samples(mml, optionsJson, JSON.stringify(samplesObj));
  }

  // Fallback: compile without samples, inject warnings for missing PCM data
  console.warn(
    `[WasmWrapper][${requestId}] compile_with_samples not available — falling back to compile_mml. ` +
    `${samples.size} sample(s) will not be embedded.`
  );
  const result = compileMml(mml, optionsJson, requestId);

  // Attach missing-sample warnings so the UI can surface them
  if (samples.size > 0) {
    const missing = Array.from(samples.keys());
    if (!result.warnings) result.warnings = [];
    missing.forEach((name) => {
      result.warnings.push({
        type: 'warning',
        message: `PCM instruments require sample upload: "${name}" could not be embedded (WASM build does not yet support compile_with_samples).`,
        line: 0,
        column: 0,
        length: 0,
        severity: 'warning',
        code: 'MISSING_SAMPLE_SUPPORT',
      });
    });
  }

  return result;
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
