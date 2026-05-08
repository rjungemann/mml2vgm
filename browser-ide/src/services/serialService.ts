/**
 * Serial Service
 *
 * WebSerial API integration for real sound chip hardware access (experimental).
 * Supports GIMIC modules (OPN2, OPNA, OPM, etc.) and generic serial adapters
 * via a pluggable protocol adapter.
 *
 * Browser support: Chrome 89+, Edge 89+. Not available in Firefox or Safari.
 * Requires a user gesture to call requestPort().
 */

import type { SoundChip } from '@/types';

// ============================================================================
// Types
// ============================================================================

/** Serial protocol variants supported by the adapter layer */
export type SerialProtocol = 'gimic' | 'scci-raw' | 'generic';

/** Options for opening a serial port */
export interface SerialConnectOptions {
  baudRate: number;
  protocol: SerialProtocol;
}

/** Information about the currently connected port */
export interface SerialPortDetails {
  usbVendorId?: number;
  usbProductId?: number;
  baudRate: number;
  protocol: SerialProtocol;
}

/** A register write command derived from a VGM stream */
export interface SerialRegisterWrite {
  timeSamples: number;
  chip: SoundChip;
  /** VGM port index (0 = primary bank, 1 = secondary bank) */
  port: number;
  addr: number;
  data: number;
}

/** Service state snapshot */
export interface SerialServiceState {
  isSupported: boolean;
  isConnected: boolean;
  isPlayingHardware: boolean;
  port: SerialPortDetails | null;
}

// ============================================================================
// GIMIC protocol constants
// ============================================================================

// Known USB identifiers for GIMIC and compatible adapters.
// The list is non-exhaustive; the port picker shows all serial devices.
const GIMIC_USB_FILTERS: SerialPortFilter[] = [
  { usbVendorId: 0x0403 }, // FTDI (GIMIC Gen1/Gen2)
  { usbVendorId: 0x16C0 }, // VOTI (GIMIC Gen3)
  { usbVendorId: 0x1FC9 }, // NXP (some clones)
];

const GIMIC_CMD_WRITE = 0x02;
const GIMIC_CMD_RESET = 0x10;

// Mapping from VGM chip type to GIMIC chip port byte.
// GIMIC only contains one chip per module, so the chip byte is always 0;
// the port byte (0 or 1) selects the register bank on dual-port chips.
const VGM_CHIP_TO_GIMIC_CHIP: Partial<Record<SoundChip, number>> = {
  YM2612: 0x00,
  YM2612X: 0x00,
  YM2608: 0x00,
  YM2203: 0x00,
  YM2151: 0x00,
  YM3812: 0x00,
  YMF262: 0x00,
  YM3526: 0x00,
  Y8950: 0x00,
  YM2413: 0x00,
  SN76489: 0x00,
  AY8910: 0x00,
};

// ============================================================================
// Serial Service
// ============================================================================

/**
 * Manages WebSerial API connections to real sound chip hardware.
 *
 * Usage flow:
 *   1. Check isSupported() — false on Firefox/Safari.
 *   2. Call requestPort() from a user gesture (button click) to show the OS
 *      port picker. The browser remembers granted ports across page reloads.
 *   3. Call connect(options) to open the port at the desired baud rate.
 *   4. Call playVgmCommands(commands, sampleRate) to stream VGM register writes
 *      to the hardware with correct inter-write timing.
 *   5. Call stopPlayback() and disconnect() to clean up.
 */
export class SerialService {
  private static instance: SerialService | null = null;

  private _port: SerialPort | null = null;
  private _writer: WritableStreamDefaultWriter<Uint8Array> | null = null;
  private _isConnected = false;
  private _isPlayingHardware = false;
  private _connectOptions: SerialConnectOptions = { baudRate: 38400, protocol: 'gimic' };

  private _stateListeners: Array<(state: SerialServiceState) => void> = [];

  // Playback scheduling
  private _playbackAbortController: AbortController | null = null;

  // ========================================================================
  // Singleton
  // ========================================================================

  public static getInstance(): SerialService {
    if (!SerialService.instance) {
      SerialService.instance = new SerialService();
    }
    return SerialService.instance;
  }

  private constructor() {}

  // ========================================================================
  // Feature detection
  // ========================================================================

  /** Returns true when the browser exposes navigator.serial (Chrome/Edge 89+). */
  public isSupported(): boolean {
    return typeof navigator !== 'undefined' && 'serial' in navigator;
  }

  // ========================================================================
  // Port management
  // ========================================================================

  /**
   * Show the OS port picker so the user can grant access to a serial device.
   * Must be called from a user gesture (click handler).
   * Returns true if a port was selected, false if the picker was dismissed.
   */
  public async requestPort(useGimicFilters = true): Promise<boolean> {
    if (!this.isSupported() || !navigator.serial) return false;

    try {
      const filters = useGimicFilters ? GIMIC_USB_FILTERS : [];
      this._port = await navigator.serial.requestPort({ filters });
      this.emitStateUpdate();
      return true;
    } catch (err) {
      // User dismissed the picker — not an error worth logging loudly.
      if ((err as DOMException).name !== 'NotFoundError') {
        console.error('[SerialService] requestPort error:', err);
      }
      return false;
    }
  }

  /**
   * Attempt to restore a previously granted port without a user gesture.
   * Returns true if at least one granted port is available.
   * The first granted port is selected; the caller can prompt for a new one
   * via requestPort() if the wrong device is restored.
   */
  public async tryRestorePort(): Promise<boolean> {
    if (!this.isSupported() || !navigator.serial) return false;

    try {
      const ports = await navigator.serial.getPorts();
      if (ports.length > 0) {
        this._port = ports[0];
        this.emitStateUpdate();
        console.log('[SerialService] Restored previously granted port');
        return true;
      }
    } catch (err) {
      console.warn('[SerialService] tryRestorePort error:', err);
    }
    return false;
  }

  /**
   * Open the selected port at the given baud rate and protocol.
   * Call requestPort() first if no port has been selected.
   */
  public async connect(options: SerialConnectOptions): Promise<boolean> {
    if (!this._port) {
      console.error('[SerialService] No port selected; call requestPort() first.');
      return false;
    }
    if (this._isConnected) {
      await this.disconnect();
    }

    this._connectOptions = options;

    try {
      await this._port.open({
        baudRate: options.baudRate,
        dataBits: 8,
        stopBits: 1,
        parity: 'none',
        flowControl: 'none',
      });

      if (!this._port.writable) {
        throw new Error('Port writable stream unavailable after open()');
      }
      this._writer = this._port.writable.getWriter();
      this._isConnected = true;
      this.emitStateUpdate();

      // Send a reset pulse so the hardware starts from a clean state.
      await this.resetHardware();

      console.log('[SerialService] Connected:', options);
      return true;
    } catch (err) {
      console.error('[SerialService] connect error:', err);
      this._isConnected = false;
      this.emitStateUpdate();
      return false;
    }
  }

  /** Close the active serial port and release the writer. */
  public async disconnect(): Promise<void> {
    this.stopPlayback();

    if (this._writer) {
      try {
        this._writer.releaseLock();
      } catch { /* ignore */ }
      this._writer = null;
    }

    if (this._port && this._isConnected) {
      try {
        await this._port.close();
      } catch (err) {
        console.warn('[SerialService] disconnect error:', err);
      }
    }

    this._isConnected = false;
    this.emitStateUpdate();
    console.log('[SerialService] Disconnected');
  }

  public isConnected(): boolean {
    return this._isConnected;
  }

  public getPortDetails(): SerialPortDetails | null {
    if (!this._port || !this._isConnected) return null;
    const info = this._port.getInfo();
    return {
      usbVendorId: info.usbVendorId,
      usbProductId: info.usbProductId,
      baudRate: this._connectOptions.baudRate,
      protocol: this._connectOptions.protocol,
    };
  }

  public getState(): SerialServiceState {
    return {
      isSupported: this.isSupported(),
      isConnected: this._isConnected,
      isPlayingHardware: this._isPlayingHardware,
      port: this.getPortDetails(),
    };
  }

  // ========================================================================
  // Hardware reset
  // ========================================================================

  /** Send a chip-reset command to all common chips on the connected hardware. */
  private async resetHardware(): Promise<void> {
    if (this._connectOptions.protocol !== 'gimic') return;

    // GIMIC reset for the primary chip (chip slot 0)
    await this.sendBytes(new Uint8Array([GIMIC_CMD_RESET, 0x00]));
    await this.delay(10);
  }

  // ========================================================================
  // Register write
  // ========================================================================

  /**
   * Write a single chip register to the hardware.
   * Encoding depends on the configured protocol.
   */
  public async writeRegister(chip: SoundChip, port: number, addr: number, data: number): Promise<void> {
    if (!this._isConnected || !this._writer) return;

    const bytes = this.encodeRegisterWrite(chip, port, addr, data);
    if (bytes) {
      await this.sendBytes(bytes);
    }
  }

  private encodeRegisterWrite(chip: SoundChip, port: number, addr: number, data: number): Uint8Array | null {
    switch (this._connectOptions.protocol) {
      case 'gimic':
        return this.encodeGimicWrite(chip, port, addr, data);
      case 'scci-raw':
        return this.encodeSCCIRawWrite(addr, data);
      case 'generic':
        return new Uint8Array([addr & 0xFF, data & 0xFF]);
      default:
        return null;
    }
  }

  /**
   * Encode a GIMIC USB-serial register write packet.
   *
   * Packet: [GIMIC_CMD_WRITE, chip_id, addr, data]
   *   - chip_id = 0 for primary chip on module; port selects register bank
   *     (for OPN2 port 1 addresses the second register file via addr | 0x100
   *      conceptually — GIMIC actually uses separate chip_id bytes for this)
   *
   * Reference: GIMIC G2 protocol documentation (scc_app source, 2019)
   */
  private encodeGimicWrite(chip: SoundChip, port: number, addr: number, _data: number): Uint8Array {
    const chipId = VGM_CHIP_TO_GIMIC_CHIP[chip] ?? 0x00;
    // For chips with secondary register banks (YM2612 port 1, YMF262 port 1),
    // the GIMIC firmware differentiates by adding 1 to the chip_id slot byte.
    const slotByte = chipId + (port > 0 ? 1 : 0);
    return new Uint8Array([GIMIC_CMD_WRITE, slotByte, addr & 0xFF, _data & 0xFF]);
  }

  /**
   * Encode an SCCI-raw 3-byte write (addr, data, 0x00 strobe).
   * Used by some homebrew SCCI-compatible serial adapters.
   */
  private encodeSCCIRawWrite(addr: number, data: number): Uint8Array {
    return new Uint8Array([addr & 0xFF, data & 0xFF, 0x00]);
  }

  // ========================================================================
  // VGM hardware playback
  // ========================================================================

  /**
   * Stream VGM register-write commands to connected hardware with real-time timing.
   *
   * The commands list should come from audioService.parseVgmCommands() or be
   * derived from a VGM file. Timestamps are in VGM sample units (44100 Hz by
   * default but parameterised via sampleRate).
   *
   * Timing is tracked using performance.now(); commands are flushed in ~4 ms
   * batches, which matches the minimum browser timer resolution. This gives
   * ±4 ms jitter — adequate for most sound hardware but may cause audible
   * timing artefacts on rhythm-critical content.
   */
  public playVgmCommands(commands: SerialRegisterWrite[], sampleRate = 44100): void {
    if (!this._isConnected) {
      console.warn('[SerialService] Cannot play: not connected');
      return;
    }

    this.stopPlayback();

    this._isPlayingHardware = true;
    this.emitStateUpdate();

    const abort = new AbortController();
    this._playbackAbortController = abort;

    const samplesPerMs = sampleRate / 1000;
    const startWall = performance.now();
    let cmdIndex = 0;

    const tick = async () => {
      if (abort.signal.aborted || cmdIndex >= commands.length) {
        if (!abort.signal.aborted) {
          this._isPlayingHardware = false;
          this.emitStateUpdate();
        }
        return;
      }

      const elapsedMs = performance.now() - startWall;
      const targetSample = elapsedMs * samplesPerMs;

      // Flush all commands whose timestamps have been reached.
      while (cmdIndex < commands.length && commands[cmdIndex].timeSamples <= targetSample) {
        const cmd = commands[cmdIndex++];
        await this.writeRegister(cmd.chip, cmd.port, cmd.addr, cmd.data);
        if (abort.signal.aborted) return;
      }

      setTimeout(tick, 4);
    };

    tick();
  }

  /** Halt in-progress hardware playback. Does not disconnect the port. */
  public stopPlayback(): void {
    if (this._playbackAbortController) {
      this._playbackAbortController.abort();
      this._playbackAbortController = null;
    }
    if (this._isPlayingHardware) {
      this._isPlayingHardware = false;
      this.emitStateUpdate();
    }
  }

  public isPlayingHardware(): boolean {
    return this._isPlayingHardware;
  }

  // ========================================================================
  // Internal helpers
  // ========================================================================

  private async sendBytes(bytes: Uint8Array): Promise<void> {
    if (!this._writer) return;
    try {
      await this._writer.write(bytes);
    } catch (err) {
      console.error('[SerialService] Write error:', err);
      this._isConnected = false;
      this.emitStateUpdate();
    }
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  // ========================================================================
  // Event listeners
  // ========================================================================

  public addStateListener(listener: (state: SerialServiceState) => void): void {
    this._stateListeners.push(listener);
  }

  public removeStateListener(listener: (state: SerialServiceState) => void): void {
    this._stateListeners = this._stateListeners.filter((l) => l !== listener);
  }

  private emitStateUpdate(): void {
    const state = this.getState();
    this._stateListeners.forEach((l) => l(state));
  }

  // ========================================================================
  // Cleanup
  // ========================================================================

  public async destroy(): Promise<void> {
    await this.disconnect();
    this._stateListeners = [];
  }
}

// ============================================================================
// Singleton export
// ============================================================================

export const serialService = SerialService.getInstance();
