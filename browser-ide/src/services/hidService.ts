/**
 * HID Service
 *
 * WebHID API integration for MIDI controllers that present as USB HID devices
 * rather than USB MIDI class devices (experimental).
 *
 * Useful when:
 *  - A controller lacks a system MIDI class driver (common on cheap keyboards,
 *    DAW controllers, some game controllers with MIDI mode).
 *  - The Web MIDI API is unavailable (future: Safari, iOS via WebHID polyfill).
 *
 * Events are forwarded to midiService.injectNoteEvent() so the rest of the app
 * (MIDIKeyboardPanel, note input) requires no changes.
 *
 * Browser support: Chrome 89+, Edge 89+. Not available in Firefox or Safari.
 * Requires a user gesture to call requestDevice().
 */

import { midiService } from './midiService';
import type { MidiNoteEvent } from './midiService';

// ============================================================================
// Types
// ============================================================================

/**
 * How to interpret raw HID input reports as MIDI data.
 *
 *  - 'usb-midi-class': Expects 4-byte USB MIDI class packets
 *    [cable|CIN, status, data1, data2]. Most common for HID MIDI adapters.
 *  - 'raw-scan': Scans each byte for a valid MIDI status byte and extracts
 *    note events from the following two bytes. Handles unusual layouts.
 */
export type HIDReportFormat = 'usb-midi-class' | 'raw-scan';

export interface HIDConnectedDevice {
  vendorId: number;
  productId: number;
  productName: string;
}

export interface HIDServiceState {
  isSupported: boolean;
  isEnabled: boolean;
  connectedDevices: HIDConnectedDevice[];
  reportFormat: HIDReportFormat;
  reportId: number | null;
  byteOffset: number;
  lastRawReport: number[] | null;
}

// ============================================================================
// USB MIDI class Code Index Numbers (CIN) — USB MIDI spec table 4-1
// ============================================================================

const CIN_NOTE_OFF = 0x08;
const CIN_NOTE_ON = 0x09;
const CIN_CONTROL_CHANGE = 0x0B;

// ============================================================================
// HID Service
// ============================================================================

export class HIDService {
  private static instance: HIDService | null = null;

  private _devices: HIDDevice[] = [];
  private _isEnabled = false;
  private _reportFormat: HIDReportFormat = 'usb-midi-class';
  private _reportId: number | null = null; // null = accept all report IDs
  private _byteOffset = 0;
  private _lastRawReport: number[] | null = null;

  private _stateListeners: Array<(state: HIDServiceState) => void> = [];

  // ========================================================================
  // Singleton
  // ========================================================================

  public static getInstance(): HIDService {
    if (!HIDService.instance) {
      HIDService.instance = new HIDService();
    }
    return HIDService.instance;
  }

  private constructor() {
    // Listen for HID device disconnections at the navigator level.
    if (this.isSupported() && navigator.hid) {
      navigator.hid.ondisconnect = (ev: Event) => {
        const device = (ev as unknown as { device: HIDDevice }).device;
        if (device) {
          this._devices = this._devices.filter((d) => d !== device);
          this.emitStateUpdate();
          console.log('[HIDService] Device disconnected:', device.productName);
        }
      };
    }
  }

  // ========================================================================
  // Feature detection
  // ========================================================================

  /** Returns true when navigator.hid is available (Chrome/Edge 89+). */
  public isSupported(): boolean {
    return typeof navigator !== 'undefined' && 'hid' in navigator;
  }

  // ========================================================================
  // Device management
  // ========================================================================

  /**
   * Show the OS HID device picker so the user can grant access.
   * Must be called from a user gesture.
   *
   * Passing an empty filters array lets the user pick any HID device.
   * This is intentional: MIDI-over-HID controllers have wildly varying
   * vendor/product IDs and usage pages.
   *
   * Returns the list of newly granted devices (may be empty if dismissed).
   */
  public async requestDevice(): Promise<HIDDevice[]> {
    if (!this.isSupported() || !navigator.hid) return [];

    try {
      // Empty filter = show all HID devices so the user can find their controller.
      const granted = await navigator.hid.requestDevice({ filters: [] });
      for (const device of granted) {
        await this.openDevice(device);
      }
      return granted;
    } catch (err) {
      if ((err as DOMException).name !== 'NotFoundError') {
        console.error('[HIDService] requestDevice error:', err);
      }
      return [];
    }
  }

  /**
   * Attempt to reconnect previously granted devices without a user gesture.
   * Returns the number of devices successfully opened.
   */
  public async tryRestoreDevices(): Promise<number> {
    if (!this.isSupported() || !navigator.hid) return 0;

    let count = 0;
    try {
      const granted = await navigator.hid.getDevices();
      for (const device of granted) {
        if (!device.opened) {
          const ok = await this.openDevice(device);
          if (ok) count++;
        }
      }
    } catch (err) {
      console.warn('[HIDService] tryRestoreDevices error:', err);
    }
    return count;
  }

  /**
   * Open a specific HIDDevice and attach the input-report listener.
   */
  private async openDevice(device: HIDDevice): Promise<boolean> {
    if (device.opened) return true;

    try {
      await device.open();
      device.oninputreport = (ev: HIDInputReportEvent) => this.handleInputReport(ev);
      this._devices.push(device);
      this.emitStateUpdate();
      console.log('[HIDService] Opened device:', device.productName,
        `(${device.vendorId.toString(16)}:${device.productId.toString(16)})`);
      return true;
    } catch (err) {
      console.error('[HIDService] Failed to open device:', device.productName, err);
      return false;
    }
  }

  /** Disconnect (close) all open HID devices. */
  public async disconnectAll(): Promise<void> {
    for (const device of this._devices) {
      if (device.opened) {
        try {
          device.oninputreport = null;
          await device.close();
        } catch { /* ignore */ }
      }
    }
    this._devices = [];
    this._isEnabled = false;
    this.emitStateUpdate();
    console.log('[HIDService] All devices disconnected');
  }

  /** Disconnect and forget a single device (removes browser permission). */
  public async forgetDevice(vendorId: number, productId: number): Promise<void> {
    const device = this._devices.find(
      (d) => d.vendorId === vendorId && d.productId === productId
    );
    if (!device) return;

    device.oninputreport = null;
    if (device.opened) {
      try { await device.close(); } catch { /* ignore */ }
    }
    try { await device.forget(); } catch { /* ignore */ }

    this._devices = this._devices.filter((d) => d !== device);
    this.emitStateUpdate();
  }

  // ========================================================================
  // HID report parsing
  // ========================================================================

  /**
   * Called for every HID input report from any open device.
   * Decodes the report according to the configured format and forwards valid
   * MIDI note events to midiService.injectNoteEvent().
   */
  private handleInputReport(ev: HIDInputReportEvent): void {
    const data = new Uint8Array(ev.data.buffer);

    // Store raw bytes for debugging.
    this._lastRawReport = Array.from(data);

    // Skip if report ID filter is set and doesn't match.
    if (this._reportId !== null && ev.reportId !== this._reportId) return;

    const events = this._reportFormat === 'usb-midi-class'
      ? this.parseUsbMidiClassReport(data, this._byteOffset)
      : this.parseRawScanReport(data, this._byteOffset);

    for (const midiEvent of events) {
      midiService.injectNoteEvent(midiEvent);
    }
  }

  /**
   * Parse a USB MIDI class packet stream from a HID report.
   *
   * USB MIDI class packs events as 4-byte groups:
   *   Byte 0: (cable_number << 4) | code_index_number
   *   Byte 1: MIDI status byte
   *   Byte 2: MIDI data 1
   *   Byte 3: MIDI data 2
   *
   * Some devices prefix the stream with a report ID byte or padding.
   * The byteOffset parameter skips leading bytes.
   */
  private parseUsbMidiClassReport(data: Uint8Array, offset: number): MidiNoteEvent[] {
    const events: MidiNoteEvent[] = [];
    let i = offset;

    while (i + 3 < data.length) {
      const cin = data[i] & 0x0F;
      const status = data[i + 1];
      const data1 = data[i + 2];
      const data2 = data[i + 3];
      i += 4;

      // Skip all-zero padding packets.
      if (data[i - 4] === 0 && status === 0) continue;

      const event = this.cinToMidiEvent(cin, status, data1, data2);
      if (event) events.push(event);
    }

    return events;
  }

  /**
   * Scan a raw report for any valid MIDI status byte and extract note events.
   * Less precise than USB MIDI class parsing but handles unusual device layouts.
   */
  private parseRawScanReport(data: Uint8Array, offset: number): MidiNoteEvent[] {
    const events: MidiNoteEvent[] = [];

    for (let i = offset; i + 2 < data.length; i++) {
      const byte = data[i];
      if ((byte & 0x80) === 0) continue; // not a status byte

      const command = byte >> 4;
      const channel = byte & 0x0F;
      const data1 = data[i + 1];
      const data2 = i + 2 < data.length ? data[i + 2] : 0;

      if (command === 0x09 && data2 > 0) {
        // Note On
        events.push({
          note: data1,
          velocity: data2,
          channel,
          type: 'noteOn',
          timestamp: performance.now(),
        });
        i += 2; // skip data bytes
      } else if (command === 0x08 || (command === 0x09 && data2 === 0)) {
        // Note Off (explicit or Note On with velocity 0)
        events.push({
          note: data1,
          velocity: 0,
          channel,
          type: 'noteOff',
          timestamp: performance.now(),
        });
        i += 2;
      }
    }

    return events;
  }

  /**
   * Convert a USB MIDI class Code Index Number (CIN) and status byte into a
   * MidiNoteEvent, returning null for non-note events (CC, pitch bend, etc.).
   */
  private cinToMidiEvent(
    cin: number,
    status: number,
    data1: number,
    data2: number
  ): MidiNoteEvent | null {
    const channel = status & 0x0F;
    const ts = performance.now();

    switch (cin) {
      case CIN_NOTE_ON:
        if (data2 > 0) {
          return { note: data1, velocity: data2, channel, type: 'noteOn', timestamp: ts };
        }
        // Note On with velocity 0 = Note Off
        return { note: data1, velocity: 0, channel, type: 'noteOff', timestamp: ts };

      case CIN_NOTE_OFF:
        return { note: data1, velocity: data2, channel, type: 'noteOff', timestamp: ts };

      case CIN_CONTROL_CHANGE:
        return {
          note: 0,
          velocity: 0,
          channel,
          type: 'controlChange',
          control: data1,
          value: data2,
          timestamp: ts,
        };

      default:
        return null;
    }
  }

  // ========================================================================
  // Configuration
  // ========================================================================

  public enable(): void {
    this._isEnabled = true;
    this.emitStateUpdate();
  }

  public disable(): void {
    this._isEnabled = false;
    this.emitStateUpdate();
  }

  public isEnabled(): boolean {
    return this._isEnabled;
  }

  public setReportFormat(format: HIDReportFormat): void {
    this._reportFormat = format;
    this.emitStateUpdate();
  }

  public setReportId(id: number | null): void {
    this._reportId = id;
    this.emitStateUpdate();
  }

  public setByteOffset(offset: number): void {
    this._byteOffset = Math.max(0, offset);
    this.emitStateUpdate();
  }

  public getConnectedDevices(): HIDConnectedDevice[] {
    return this._devices.map((d) => ({
      vendorId: d.vendorId,
      productId: d.productId,
      productName: d.productName || `HID ${d.vendorId.toString(16)}:${d.productId.toString(16)}`,
    }));
  }

  public getState(): HIDServiceState {
    return {
      isSupported: this.isSupported(),
      isEnabled: this._isEnabled,
      connectedDevices: this.getConnectedDevices(),
      reportFormat: this._reportFormat,
      reportId: this._reportId,
      byteOffset: this._byteOffset,
      lastRawReport: this._lastRawReport,
    };
  }

  // ========================================================================
  // Event listeners
  // ========================================================================

  public addStateListener(listener: (state: HIDServiceState) => void): void {
    this._stateListeners.push(listener);
  }

  public removeStateListener(listener: (state: HIDServiceState) => void): void {
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
    await this.disconnectAll();
    this._stateListeners = [];
  }
}

// ============================================================================
// Singleton export
// ============================================================================

export const hidService = HIDService.getInstance();
