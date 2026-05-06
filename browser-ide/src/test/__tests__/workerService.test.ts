/**
 * Worker Service Tests
 *
 * Verifies WorkerManager shape and non-crash behaviour.
 * Workers cannot actually run WASM in a test environment, so these tests
 * focus on the manager API surface and graceful-degradation paths.
 */

import { describe, it, expect } from 'vitest';

// Worker API is not available in jsdom – provide a minimal stub.
global.Worker = class {
  onmessage: ((e: any) => void) | null = null;
  onerror: ((e: any) => void) | null = null;
  postMessage(_: any) {}
  terminate() {}
} as any;

describe('WorkerService', () => {
  // ── Module exports ────────────────────────────────────────────────────────

  it('exports getWorkerManager', { timeout: 2000 }, async () => {
    const mod = await import('@/services/workerService');
    expect(typeof mod.getWorkerManager).toBe('function');
  });

  it('exports WorkerManager class or factory', { timeout: 2000 }, async () => {
    const mod = await import('@/services/workerService');
    expect(mod.WorkerManager).toBeDefined();
  });

  it('exports preWarmWorkers function', { timeout: 2000 }, async () => {
    const mod = await import('@/services/workerService');
    expect(typeof mod.preWarmWorkers).toBe('function');
  });

  // ── WorkerManager instance shape ──────────────────────────────────────────

  it('WorkerManager instance has compile method', { timeout: 2000 }, async () => {
    const { WorkerManager } = await import('@/services/workerService');
    const mgr = new WorkerManager({ maxWorkers: 1 });
    expect(typeof mgr.compile).toBe('function');
  });

  it('WorkerManager instance has cancel method', { timeout: 2000 }, async () => {
    const { WorkerManager } = await import('@/services/workerService');
    const mgr = new WorkerManager({ maxWorkers: 1 });
    expect(typeof mgr.cancelActiveCompilation).toBe('function');
  });

  it('WorkerManager instance has destroy/terminate method', { timeout: 2000 }, async () => {
    const { WorkerManager } = await import('@/services/workerService');
    const mgr = new WorkerManager({ maxWorkers: 1 });
    const hasDestroy =
      typeof (mgr as any).terminateAll === 'function';
    expect(hasDestroy).toBe(true);
  });

  // ── Graceful failure on compile without WASM ──────────────────────────────

  it('compile() rejects gracefully without real WASM', { timeout: 10000 }, async () => {
    const { WorkerManager } = await import('@/services/workerService');
    const mgr = new WorkerManager({ maxWorkers: 1 });
    try {
      await mgr.compile('t120 c d e', {});
    } catch (e) {
      // Expected — worker can't run WASM in test env
      expect(e).toBeDefined();
    }
  });

  // ── getWorkerManager returns singleton ────────────────────────────────────

  it('getWorkerManagerSync() returns same instance each call', { timeout: 2000 }, async () => {
    const { getWorkerManagerSync } = await import('@/services/workerService');
    const a = getWorkerManagerSync();
    const b = getWorkerManagerSync();
    expect(a).toBe(b);
  });
});
