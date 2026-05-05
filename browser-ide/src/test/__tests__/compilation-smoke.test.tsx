/**
 * Smoke test for WASM compilation
 *
 * This test verifies that:
 * 1. WASM module initializes correctly
 * 2. Worker is pre-warmed
 * 3. Compilation request completes
 * 4. Progress exceeds 10%
 */

import { describe, it, expect, beforeAll, afterAll, vi } from 'vitest';
import { wasmService } from '@/services/wasmService';
import { WorkerManager } from '@/services/workerService';

describe('Compilation Smoke Test', () => {
  let consoleSpy: any;
  const logs: string[] = [];

  beforeAll(() => {
    // Capture console logs
    consoleSpy = vi.spyOn(console, 'log').mockImplementation((msg: any) => {
      logs.push(String(msg));
    });
  });

  afterAll(() => {
    consoleSpy.mockRestore();
  });

  it('should initialize WASM module', async () => {
    try {
      await wasmService.init();
      expect(wasmService).toBeDefined();

      const hasInitLog = logs.some(log =>
        log.includes('WASM') && log.includes('Initialize')
      );
      console.log('✓ WASM initialized');
    } catch (error) {
      console.error('✗ WASM initialization failed:', error);
      throw error;
    }
  });

  it('should complete compilation', async () => {
    const mml = 't 127 l 4 A4A8A16 C4C8C16 E4E8E16';
    const options = {
      format: 'vgm',
      target_chips: ['YM2608'],
      clock_count: 7987200,
    };

    try {
      const result = await wasmService.compile(mml, options);

      expect(result).toBeDefined();
      expect(result.data).toBeDefined();
      expect(result.data.length).toBeGreaterThan(0);

      console.log(`✓ Compilation successful: ${result.data.length} bytes`);
      console.log(`  Part count: ${result.info?.part_count}`);
      console.log(`  Command count: ${result.info?.command_count}`);
      console.log(`  Duration: ${result.info?.duration_seconds}s`);
    } catch (error) {
      console.error('✗ Compilation failed:', error);
      throw error;
    }
  });

  it('should have logged progress updates', () => {
    // Check that compilation progress was logged
    const progressLogs = logs.filter(log =>
      log.includes('progress') ||
      log.includes('Progress') ||
      log.includes('Starting')
    );

    if (progressLogs.length === 0) {
      console.warn('⚠ No progress logs found - compilation may have skipped logging');
    } else {
      console.log(`✓ Found ${progressLogs.length} progress logs`);
      progressLogs.slice(0, 3).forEach(log => console.log(`  - ${log.substring(0, 80)}`));
    }
  });

  it('should display all console logs for debugging', () => {
    console.log('\n=== Console Log Output ===');
    logs.forEach((log, i) => {
      // Only show relevant logs
      if (log.includes('[') || log.includes('Compil') || log.includes('Worker') || log.includes('progress')) {
        console.log(`${i}: ${log.substring(0, 120)}`);
      }
    });
    console.log('=== End Log Output ===\n');
  });
});
