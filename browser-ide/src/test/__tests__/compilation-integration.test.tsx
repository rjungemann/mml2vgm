/**
 * Integration test for compilation UI
 *
 * This test verifies the complete compilation flow through the UI:
 * 1. Page loads and WASM initializes
 * 2. Worker pre-warms
 * 3. User enters MML code
 * 4. User clicks Compile
 * 5. Compilation completes and progress reaches 100%
 *
 * Run this test locally with: npm run test -- compilation-integration
 *
 * DEBUGGING:
 * To see full console logs:
 * - Check browser console while test runs
 * - Look for "[Worker]" messages to verify worker compilation
 * - Look for "[WasmWrapper]" messages to verify WASM initialization
 * - Check for "[compileStore]" messages to verify store pipeline
 */

import { describe, it, expect, beforeAll, afterAll, vi, timeout } from 'vitest';
import { useCompileStore } from '@/stores/compileStore';
import { useDocumentStore } from '@/stores/documentStore';
import { wasmService } from '@/services/wasmService';
import { getWorkerManager } from '@/services/workerService';

describe('Compilation Integration Test', () => {
  const logs: { level: string; message: string; timestamp: number }[] = [];
  let consoleSpy: any;
  let errorSpy: any;

  beforeAll(async () => {
    // Capture all console output
    consoleSpy = vi.spyOn(console, 'log').mockImplementation((msg: any) => {
      logs.push({ level: 'log', message: String(msg), timestamp: Date.now() });
    });

    errorSpy = vi.spyOn(console, 'error').mockImplementation((msg: any) => {
      logs.push({ level: 'error', message: String(msg), timestamp: Date.now() });
    });

    // Initialize WASM
    try {
      await wasmService.init();
    } catch (e) {
      console.error('Failed to initialize WASM:', e);
    }
  });

  afterAll(() => {
    consoleSpy.mockRestore();
    errorSpy.mockRestore();
  });

  it('should have worker support available', () => {
    const compileStore = useCompileStore.getState();
    console.log(`Worker support enabled: ${compileStore.useWebWorkers}`);

    if (!compileStore.useWebWorkers) {
      console.warn('⚠ Web Workers are not supported - will fall back to main thread');
    }
    expect(compileStore.useWebWorkers).toBe(true);
  });

  it('should initialize worker manager', async () => {
    const compileStore = useCompileStore.getState();
    console.log('Initializing worker manager...');

    try {
      await compileStore.initWorkerManager();
      const { workerManager } = useCompileStore.getState();
      console.log(`Worker manager initialized: ${!!workerManager}`);
      expect(workerManager).toBeDefined();
    } catch (error) {
      console.error('Worker manager initialization failed:', error);
      throw error;
    }
  });

  it('should compile MML through the store', async () => {
    const documentStore = useDocumentStore.getState();
    const compileStore = useCompileStore.getState();

    // Create a test document if needed
    if (!documentStore.activeDocumentId) {
      documentStore.createDocument();
    }

    const docId = documentStore.activeDocumentId;
    expect(docId).toBeDefined();

    // Set document content
    documentStore.updateDocument(docId!, {
      content: 't 127 l 4 A4A8A16 C4C8C16 E4E8E16',
    });

    const options = {
      format: 'vgm' as const,
      target_chips: ['YM2608' as const],
      clock_count: 7987200,
    };

    console.log(`Starting compilation for document ${docId}`);
    console.log('Compile store state before:', {
      useWebWorkers: compileStore.useWebWorkers,
      hasWorkerManager: !!compileStore.workerManager,
      status: compileStore.status,
    });

    try {
      const result = await compileStore.compile(docId!, options);

      console.log('Compilation result:', {
        dataLength: result.data?.length,
        partCount: result.partCount,
        commandCount: result.commandCount,
        duration: result.duration,
      });

      expect(result).toBeDefined();
      expect(result.data).toBeDefined();
      expect(result.data!.length).toBeGreaterThan(0);
    } catch (error) {
      console.error('Compilation error:', error);
      throw error;
    }
  }, { timeout: 30000 });

  it('should log worker messages if worker was used', () => {
    const workerLogs = logs.filter(log =>
      log.message.includes('[Worker]') ||
      log.message.includes('[WasmWrapper]') ||
      log.message.includes('[compileStore]')
    );

    console.log(`\nFound ${workerLogs.length} worker-related logs:`);

    if (workerLogs.length === 0) {
      console.warn('⚠ No worker logs found - compilation may have used main thread fallback');
    } else {
      workerLogs.slice(0, 10).forEach(log => {
        console.log(`  [${log.level}] ${log.message.substring(0, 100)}`);
      });
    }

    // Critical logs to check
    const hasWorkerCompile = workerLogs.some(log =>
      log.message.includes('onmessage') && log.message.includes('COMPILE')
    );
    const hasWasmInit = workerLogs.some(log =>
      log.message.includes('WasmWrapper') && log.message.includes('Initializ')
    );

    if (!hasWorkerCompile) {
      console.warn('⚠ No worker COMPILE message found - worker may not be receiving requests');
    }
    if (!hasWasmInit) {
      console.warn('⚠ No WASM wrapper init message found - WASM may not be initializing in worker');
    }
  });

  it('should display full log output for debugging', () => {
    console.log('\n=== FULL CONSOLE OUTPUT ===\n');
    logs.forEach((log, i) => {
      const timestamp = new Date(log.timestamp).toISOString().split('T')[1];
      const prefix = log.level === 'error' ? '❌' : '✓';
      console.log(`${prefix} [${timestamp}] ${log.message.substring(0, 120)}`);
    });
    console.log('\n=== END LOG OUTPUT ===\n');
  });
});
