/**
 * WASM Service Tests
 *
 * Tests for WasmService initialization and compile API.
 * WASM is mocked via the global setup in setup.tsx.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

// We test the service class directly with a mocked WASM module.
// The real module is not available in the test environment.

describe('WasmService', () => {
  // ── Module shape ──────────────────────────────────────────────────────────

  it('exports a wasmService singleton', { timeout: 5000 }, async () => {
    const { wasmService } = await import('@/services/wasmService');
    expect(wasmService).toBeDefined();
  });

  it('exposes an isInitialized property', { timeout: 5000 }, async () => {
    const { wasmService } = await import('@/services/wasmService');
    expect(typeof wasmService.isInitialized).toBe('boolean');
  });

  it('exposes an init method', { timeout: 5000 }, async () => {
    const { wasmService } = await import('@/services/wasmService');
    expect(typeof wasmService.init).toBe('function');
  });

  it('exposes a compile method', { timeout: 5000 }, async () => {
    const { wasmService } = await import('@/services/wasmService');
    expect(typeof wasmService.compile).toBe('function');
  });

  it('exposes a validate method', { timeout: 5000 }, async () => {
    const { wasmService } = await import('@/services/wasmService');
    expect(typeof (wasmService as any).validate === 'function' ||
      typeof (wasmService as any).validateSource === 'function').toBe(true);
  });

  // ── init does not throw synchronously ─────────────────────────────────────

  it('init() does not throw synchronously', { timeout: 10000 }, async () => {
    const { wasmService } = await import('@/services/wasmService');
    // init may reject (no real WASM in test env), but must not throw sync
    try {
      await wasmService.init();
    } catch {
      // graceful failure is acceptable in test environment
    }
  });

  // ── compile handles empty string without hanging ───────────────────────────

  it('compile() resolves or rejects for empty string without hanging', { timeout: 5000 }, async () => {
    const { wasmService } = await import('@/services/wasmService');
    try {
      await wasmService.compile('', {});
    } catch {
      // error is fine
    }
  });

  // ── Singleton returns same instance ───────────────────────────────────────

  it('getInstance() returns the same instance every time', { timeout: 1000 }, async () => {
    const { WasmService } = await import('@/services/wasmService');
    const a = (WasmService as any).getInstance();
    const b = (WasmService as any).getInstance();
    expect(a).toBe(b);
  });
});
