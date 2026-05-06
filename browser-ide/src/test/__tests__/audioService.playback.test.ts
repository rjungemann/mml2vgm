import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { audioService } from '@/services/audioService';

const { mockWasmService } = vi.hoisted(() => ({
  mockWasmService: {
    createChipPlayer: vi.fn(async () => 'chip-player-1'),
    addChipToPlayer: vi.fn(async () => undefined),
    writeChipRegister: vi.fn(async () => undefined),
    generateSamples: vi.fn(async () => new Float32Array(4096 * 2)),
    compile: vi.fn(async () => ({
      data: new Uint8Array(),
      partCount: 0,
      commandCount: 0,
      duration: 0,
      durationSeconds: 0,
      chipsUsed: [],
      warnings: [],
    })),
    stopVgm: vi.fn(async () => undefined),
    destroyVgmPlayer: vi.fn(() => undefined),
    destroyChipPlayer: vi.fn(() => undefined),
  },
}));

vi.mock('@/services/wasmService', () => ({
  wasmService: mockWasmService,
}));

class MockAudioWorkletNode {
  public port = {
    onmessage: null as ((event: any) => void) | null,
    onmessageerror: null as ((event: any) => void) | null,
    postMessage: vi.fn(),
  };

  connect() {
    return this;
  }

  disconnect() {
    return this;
  }
}

class MockAudioContext {
  public state: AudioContextState = 'running';
  public currentTime = 0;
  public destination = {} as AudioNode;
  public audioWorklet = {
    addModule: vi.fn(async () => undefined),
  };

  resume = vi.fn(async () => {
    this.state = 'running';
  });

  close = vi.fn(async () => undefined);

  createScriptProcessor = vi.fn(() => ({
    onaudioprocess: null,
    connect: vi.fn(),
    disconnect: vi.fn(),
  }));
}

const createMinimalAudibleVgm = (): Uint8Array => {
  const data = new Uint8Array(0x50);
  data[0] = 0x56; // 'V'
  data[1] = 0x67; // 'g'
  data[2] = 0x6d; // 'm'
  data[3] = 0x20; // ' '
  // data offset = 0 -> stream starts at 0x40.

  let offset = 0x40;
  data[offset++] = 0x50; // SN76489 write
  data[offset++] = 0x90; // channel 0 volume = 0 (loud)
  data[offset++] = 0x61; // wait n samples
  data[offset++] = 0x10;
  data[offset++] = 0x00;
  data[offset++] = 0x50; // SN76489 write
  data[offset++] = 0x80; // tone latch
  data[offset++] = 0x66; // end of data

  return data;
};

const ARPEGGIO_SAMPLE_PATH = path.resolve(process.cwd(), 'public/samples/arpeggio.gwi');

describe('AudioService playback duration regression', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();

    Object.defineProperty(window, 'AudioContext', {
      configurable: true,
      writable: true,
      value: MockAudioContext,
    });

    Object.defineProperty(window, 'webkitAudioContext', {
      configurable: true,
      writable: true,
      value: MockAudioContext,
    });

    Object.defineProperty(window, 'AudioWorklet', {
      configurable: true,
      writable: true,
      value: function MockAudioWorklet() {},
    });

    Object.defineProperty(window, 'AudioWorkletNode', {
      configurable: true,
      writable: true,
      value: MockAudioWorkletNode,
    });

    Object.defineProperty(URL, 'createObjectURL', {
      configurable: true,
      writable: true,
      value: vi.fn(() => 'blob:test-audio-worklet'),
    });

    Object.defineProperty(URL, 'revokeObjectURL', {
      configurable: true,
      writable: true,
      value: vi.fn(),
    });
  });

  afterEach(() => {
    audioService.stop();
    audioService.destroy();
    vi.useRealTimers();
  });

  it('keeps arpeggio-like VGM audible for longer than an initial burst (>0.5s)', async () => {
    const audibleSample = new Float32Array(4096 * 2);
    audibleSample.fill(0.125);
    mockWasmService.generateSamples.mockImplementation(async () => audibleSample);

    await audioService.playVGM(createMinimalAudibleVgm(), {
      chips: ['SN76489', 'YM2608'],
      volume: 1.0,
    });

    // bufferSize defaults to 4096 at 44.1kHz: ~92.9ms/buffer.
    // More than 0.5s means at least 6 buffers.
    for (let i = 0; i < 8; i++) {
      await vi.runOnlyPendingTimersAsync();
      await Promise.resolve();
    }

    const debug = audioService.getRuntimeDebugInfo();
    expect(debug.isPlaying).toBe(true);
    expect(debug.generatedBufferCount).toBeGreaterThanOrEqual(6);
    expect(debug.lastPeak).toBeGreaterThan(0.01);
    expect(debug.silentBufferCount).toBe(0);
    expect(debug.appliedWriteCount).toBeGreaterThan(0);
  });

  it('compiles literal arpeggio.gwi and stays audible for >0.5s', async () => {
    const arpeggioSource = await readFile(ARPEGGIO_SAMPLE_PATH, 'utf8');
    const audibleSample = new Float32Array(4096 * 2);
    audibleSample.fill(0.125);

    const compiledVgm = createMinimalAudibleVgm();
    mockWasmService.compile.mockResolvedValue({
      data: compiledVgm,
      partCount: 3,
      commandCount: 78,
      duration: 0,
      durationSeconds: 1.2,
      chipsUsed: ['YM2608', 'SN76489'],
      warnings: [],
    });
    mockWasmService.generateSamples.mockImplementation(async () => audibleSample);

    await audioService.playMML(arpeggioSource, ['YM2608', 'SN76489']);

    // Keep advancing generation windows long enough to exceed 0.5 seconds.
    for (let i = 0; i < 8; i++) {
      await vi.runOnlyPendingTimersAsync();
      await Promise.resolve();
    }

    const debug = audioService.getRuntimeDebugInfo();
    expect(mockWasmService.compile).toHaveBeenCalledTimes(1);
    expect(mockWasmService.compile).toHaveBeenCalledWith(arpeggioSource, {
      format: 'vgm',
      target_chips: ['YM2608', 'SN76489'],
    });
    expect(debug.generatedBufferCount).toBeGreaterThanOrEqual(6);
    expect(debug.lastPeak).toBeGreaterThan(0.01);
    expect(debug.silentBufferCount).toBe(0);
    expect(debug.appliedWriteCount).toBeGreaterThan(0);
    expect(debug.isPlaying).toBe(true);
  });
});
