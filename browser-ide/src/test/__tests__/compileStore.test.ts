/**
 * Compile Store Tests
 *
 * Tests for the Zustand compile store — initial state and API surface.
 * Full compilation is tested in integration/smoke tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';

// Reset the module between tests so store state is fresh.
describe('compileStore', () => {
  beforeEach(() => {
    vi.resetModules?.();
  });

  // ── Initial state ─────────────────────────────────────────────────────────

  it('initial status is idle', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(state.status).toBe('idle');
  });

  it('initial progress is 0', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(state.progress).toBe(0);
  });

  it('initial currentDocumentId is null', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(state.currentDocumentId).toBeNull();
  });

  it('initial queue is empty', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(state.queue.length).toBe(0);
  });

  // ── Action surface ────────────────────────────────────────────────────────

  it('has compile action', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(typeof state.compile).toBe('function');
  });

  it('has cancel action', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(typeof state.cancel).toBe('function');
  });

  it('has clearResults action', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(typeof state.clearResults).toBe('function');
  });

  it('has setProgress action', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(typeof state.setProgress).toBe('function');
  });

  it('has getStatus action', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(typeof state.getStatus).toBe('function');
  });

  it('has isCompiling action', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    const state = useCompileStore.getState();
    expect(typeof state.isCompiling).toBe('function');
  });

  // ── setProgress updates state ─────────────────────────────────────────────

  it('setProgress(50) updates progress to 50', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    useCompileStore.getState().setProgress(50);
    expect(useCompileStore.getState().progress).toBe(50);
  });

  it('setProgress(100) updates progress to 100', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    useCompileStore.getState().setProgress(100);
    expect(useCompileStore.getState().progress).toBe(100);
  });

  // ── cancel on idle does not throw ─────────────────────────────────────────

  it('cancel() on idle store does not throw', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    expect(() => useCompileStore.getState().cancel()).not.toThrow();
  });

  // ── clearResults empties results map ─────────────────────────────────────

  it('clearResults() clears the results map', { timeout: 1000 }, async () => {
    const { useCompileStore } = await import('@/stores/compileStore');
    useCompileStore.getState().clearResults();
    expect(useCompileStore.getState().results.size).toBe(0);
  });
});

// Vitest doesn't inject `vi` globally inside the file unless explicitly imported.
import { vi } from 'vitest';
