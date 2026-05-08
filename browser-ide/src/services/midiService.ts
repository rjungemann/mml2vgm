/**
 * MIDI Service
 * 
 * Manages Web MIDI API integration for MIDI keyboard input and preview.
 * Handles MIDI device discovery, note input, and note preview.
 */

import type { SoundChip } from '@/types';

// ============================================================================
// Types
// ============================================================================

/** MIDI device information */
export interface MidiDeviceInfo {
    id: string;
    name: string | null;
    type: 'input' | 'output';
    manufacturer: string | null;
    version: string | null;
    state: 'connected' | 'disconnected';
}

/** MIDI note event */
export interface MidiNoteEvent {
    note: number; // 0-127 (MIDI note number)
    velocity: number; // 0-127
    channel: number; // 0-15
    type: 'noteOn' | 'noteOff' | 'controlChange';
    control?: number; // For control change
    value?: number; // For control change
    timestamp: number;
}

/** MIDI service state */
export interface MidiServiceState {
    isEnabled: boolean;
    isSupported: boolean;
    inputDevices: MidiDeviceInfo[];
    outputDevices: MidiDeviceInfo[];
    assignedPart: number | null;
    mode: 'preview' | 'input';
}

// ============================================================================
// MIDI Service Class
// ============================================================================

/**
 * Service for managing Web MIDI API integration.
 * 
 * This class provides:
 * - MIDI device discovery and management
 * - MIDI note input handling
 * - Note preview via WASM chip player
 * - MIDI to MML note conversion
 */
export class MidiService {
    private static instance: MidiService | null = null;
    
    // MIDI Access API
    private _midiAccess: MIDIAccess | null = null;
    
    // Device lists
    private _inputDevices: Map<string, MIDIInput> = new Map();
    private _outputDevices: Map<string, MIDIOutput> = new Map();
    
    // State
    private _isEnabled: boolean = false;
    private _assignedPart: number | null = null;
    private _mode: 'preview' | 'input' = 'preview';
    

    
    // Event listeners
    private _listeners: Array<(event: MidiNoteEvent) => void> = [];
    private _stateListeners: Array<(state: MidiServiceState) => void> = [];
    
    // Note mappings for MML
    private _noteNames = ['c', 'c+', 'd', 'd+', 'e', 'f', 'f+', 'g', 'g+', 'a', 'a+', 'b'];
    
    // ========================================================================
    // Singleton
    // ========================================================================
    
    public static getInstance(): MidiService {
        if (!MidiService.instance) {
            MidiService.instance = new MidiService();
        }
        return MidiService.instance;
    }
    
    private constructor() {
        // Private constructor for singleton
        this._isSupported = this.checkSupport();
    }
    
    // ========================================================================
    // Support Check
    // ========================================================================
    
    /**
     * Check if Web MIDI API is supported.
     */
    private checkSupport(): boolean {
        return 'requestMIDIAccess' in navigator;
    }
    
    private _isSupported: boolean = false;
    
    /**
     * Check if MIDI is supported.
     */
    public isSupported(): boolean {
        return this._isSupported;
    }
    
    // ========================================================================
    // Initialization
    // ========================================================================
    
    /**
     * Initialize MIDI service.
     */
    public async init(): Promise<boolean> {
        if (!this._isSupported) {
            console.warn('[MIDI Service] Web MIDI API not supported');
            return false;
        }
        
        if (this._midiAccess) {
            return true; // Already initialized
        }
        
        try {
            this._midiAccess = await navigator.requestMIDIAccess();
            this.setupDeviceListeners();
            this.refreshDevices();
            this._isEnabled = true;
            this.emitStateUpdate();
            console.log('[MIDI Service] Initialized successfully');
            return true;
        } catch (error) {
            console.error('[MIDI Service] Failed to initialize:', error);
            return false;
        }
    }
    
    /**
     * Setup listeners for device changes.
     */
    private setupDeviceListeners(): void {
        if (!this._midiAccess) return;
        
        this._midiAccess.onstatechange = (event: MIDIConnectionEvent) => {
            console.log('[MIDI Service] Device state change:', event);
            this.refreshDevices();
            this.emitStateUpdate();
        };
    }
    
    /**
     * Refresh device lists.
     */
    private refreshDevices(): void {
        if (!this._midiAccess) return;
        
        this._inputDevices.clear();
        this._outputDevices.clear();
        
        // Refresh inputs
        const inputs = Array.from(this._midiAccess.inputs.values());
        for (const input of inputs) {
            this._inputDevices.set(input.id, input);
            input.onmidimessage = (event) => this.handleMidiMessage(event);
        }
        
        // Refresh outputs
        const outputs = Array.from(this._midiAccess.outputs.values());
        for (const output of outputs) {
            this._outputDevices.set(output.id, output);
        }
    }
    
    // ========================================================================
    // Device Management
    // ========================================================================
    
    /**
     * Get list of input devices.
     */
    public getInputDevices(): MidiDeviceInfo[] {
        if (!this._midiAccess) return [];
        
        return Array.from(this._midiAccess.inputs.values()).map(input => ({
            id: input.id,
            name: input.name,
            type: 'input' as const,
            manufacturer: input.manufacturer,
            version: input.version,
            state: input.state === 'connected' ? 'connected' : 'disconnected',
        }));
    }
    
    /**
     * Get list of output devices.
     */
    public getOutputDevices(): MidiDeviceInfo[] {
        if (!this._midiAccess) return [];
        
        return Array.from(this._midiAccess.outputs.values()).map(output => ({
            id: output.id,
            name: output.name,
            type: 'output' as const,
            manufacturer: output.manufacturer,
            version: output.version,
            state: output.state === 'connected' ? 'connected' : 'disconnected',
        }));
    }
    
    /**
     * Get current state.
     */
    public getState(): MidiServiceState {
        return {
            isEnabled: this._isEnabled,
            isSupported: this._isSupported,
            inputDevices: this.getInputDevices(),
            outputDevices: this.getOutputDevices(),
            assignedPart: this._assignedPart,
            mode: this._mode,
        };
    }
    
    // ========================================================================
    // MIDI Message Handling
    // ========================================================================
    
    /**
     * Handle MIDI message from input device.
     */
    private handleMidiMessage(event: MIDIMessageEvent): void {
        const data = event.data as Uint8Array;
        const status = data[0];
        const data1 = data[1];
        const data2 = data[2];
        const command = status >> 4;
        const channel = status & 0x0F;
        
        let eventType: MidiNoteEvent['type'] = 'noteOn';
        let note: number | undefined = data1;
        let velocity: number | undefined = data2;
        let control: number | undefined;
        let value: number | undefined;
        
        switch (command) {
            case 0x08: // Note Off
                eventType = 'noteOff';
                note = data1;
                velocity = data2;
                break;
            case 0x09: // Note On
                if (data2 > 0) {
                    eventType = 'noteOn';
                } else {
                    eventType = 'noteOff';
                }
                note = data1;
                velocity = data2;
                break;
            case 0x0B: // Control Change
                eventType = 'controlChange';
                control = data1;
                value = data2;
                break;
            default:
                return; // Ignore other message types
        }
        
        if (note !== undefined && velocity !== undefined) {
            const midiEvent: MidiNoteEvent = {
                note,
                velocity,
                channel,
                type: eventType,
                timestamp: event.timeStamp || Date.now(),
            };
            
            this.handleNoteEvent(midiEvent);
            this.emitNoteEvent(midiEvent);
        } else if (control !== undefined && value !== undefined) {
            const midiEvent: MidiNoteEvent = {
                note: 0,
                velocity: 0,
                channel,
                type: eventType,
                control,
                value,
                timestamp: event.timeStamp || Date.now(),
            };
            
            this.emitNoteEvent(midiEvent);
        }
    }
    
    /**
     * Handle note event based on mode.
     */
    private handleNoteEvent(event: MidiNoteEvent): void {
        if (event.type !== 'noteOn' && event.type !== 'noteOff') return;
        
        if (this._mode === 'input' && this._assignedPart !== null) {
            // Insert note into editor
            if (event.type === 'noteOn' && event.velocity > 0) {
                const mmlNote = this.midiToMML(event.note);
                // This would trigger insertion at cursor position
                // For now, just log
                console.log('[MIDI Service] Input note:', mmlNote, 'for part', this._assignedPart);
            }
        } else if (this._mode === 'preview' && this._assignedPart !== null) {
            // Preview note via WASM
            this.previewNote(this._assignedPart, event.note, event.velocity);
        }
    }
    
    /**
     * Preview a note using the WASM chip player.
     */
    private previewNote(partIndex: number, midiNote: number, velocity: number): void {
        // This would send the note to the WASM chip player
        // For now, just log
        console.log('[MIDI Service] Preview note:', midiNote, 'velocity:', velocity, 'part:', partIndex);
    }
    
    // ========================================================================
    // MIDI to MML Conversion
    // ========================================================================
    
    /**
     * Convert MIDI note number to MML note name.
     * 
     * @param midiNote - MIDI note number (0-127)
     * @returns MML note string (e.g., 'c4', 'd#5')
     */
    public midiToMML(midiNote: number): string {
        const noteName = this._noteNames[midiNote % 12];
        const octave = Math.floor(midiNote / 12) - 1; // MIDI octave 0 = C-1, but MML typically uses octave 4 as middle C
        return `${noteName}${octave}`;
    }
    
    /**
     * Convert MML note to MIDI note number.
     * 
     * @param note - MML note string (e.g., 'c4', 'd#5')
     * @returns MIDI note number
     */
    public mmlToMidi(note: string): number {
        // Parse note: extract name and octave
        const match = note.match(/^([a-g][+#]?)(\d+)$/i);
        if (!match) {
            // Try without octave
            const simpleMatch = note.match(/^([a-g][+#]?)$/i);
            if (simpleMatch) {
                const noteName = simpleMatch[1].toLowerCase();
                const noteIndex = this._noteNames.indexOf(noteName as never);
                return noteIndex >= 0 ? noteIndex + 48 : 60; // Default to C4 (60)
            }
            return 60; // Default to C4
        }
        
        const noteName = match[1].toLowerCase();
        const octave = parseInt(match[2], 10);
        
        const noteIndex = this._noteNames.indexOf(noteName as never);
        if (noteIndex < 0) {
            return 60; // Default to C4
        }
        
        return noteIndex + (octave + 1) * 12;
    }
    
    // ========================================================================
    // Configuration
    // ========================================================================
    
    /**
     * Enable MIDI input.
     */
    public enable(): void {
        this._isEnabled = true;
        this.emitStateUpdate();
    }
    
    /**
     * Disable MIDI input.
     */
    public disable(): void {
        this._isEnabled = false;
        this.emitStateUpdate();
    }
    
    /**
     * Check if MIDI is enabled.
     */
    public isEnabled(): boolean {
        return this._isEnabled;
    }
    
    /**
     * Set the assigned part for MIDI input.
     * 
     * @param partIndex - Part index to assign, or null to clear
     */
    public setAssignedPart(partIndex: number | null): void {
        this._assignedPart = partIndex;
        this.emitStateUpdate();
    }
    
    /**
     * Get the assigned part.
     */
    public getAssignedPart(): number | null {
        return this._assignedPart;
    }
    
    /**
     * Set the MIDI mode (preview or input).
     * 
     * @param mode - 'preview' to preview notes, 'input' to insert into editor
     */
    public setMode(mode: 'preview' | 'input'): void {
        this._mode = mode;
        this.emitStateUpdate();
    }
    
    /**
     * Get the current mode.
     */
    public getMode(): 'preview' | 'input' {
        return this._mode;
    }
    
    /**
     * Set the chip and channel for preview.
     * 
     * @param _chip - Sound chip to use for preview
     * @param _channel - Channel on the chip
     */
    public setPreviewChannel(_chip: SoundChip, _channel: number): void {
        // For future implementation when connected to WASM chip player
    }
    
    // ========================================================================
    // Event Listeners
    // ========================================================================
    
    /**
     * Add a listener for MIDI note events.
     */
    public addListener(listener: (event: MidiNoteEvent) => void): void {
        this._listeners.push(listener);
    }
    
    /**
     * Remove a listener.
     */
    public removeListener(listener: (event: MidiNoteEvent) => void): void {
        this._listeners = this._listeners.filter(l => l !== listener);
    }
    
    /**
     * Add a listener for state changes.
     */
    public addStateListener(listener: (state: MidiServiceState) => void): void {
        this._stateListeners.push(listener);
    }
    
    /**
     * Remove a state listener.
     */
    public removeStateListener(listener: (state: MidiServiceState) => void): void {
        this._stateListeners = this._stateListeners.filter(l => l !== listener);
    }
    
    /**
     * Inject a note event from an external source (e.g. HID service) into the
     * existing listener chain, applying the same mode/preview logic as native
     * Web MIDI input.
     */
    public injectNoteEvent(event: MidiNoteEvent): void {
        this.handleNoteEvent(event);
        this.emitNoteEvent(event);
    }

    /**
     * Emit note event to all listeners.
     */
    private emitNoteEvent(event: MidiNoteEvent): void {
        this._listeners.forEach(listener => listener(event));
    }
    
    /**
     * Emit state update to all listeners.
     */
    private emitStateUpdate(): void {
        const state = this.getState();
        this._stateListeners.forEach(listener => listener(state));
    }
    
    // ========================================================================
    // Cleanup
    // ========================================================================
    
    /**
     * Clean up resources.
     */
    public cleanup(): void {
        this._listeners = [];
        this._stateListeners = [];
        
        if (this._midiAccess) {
            this._midiAccess.onstatechange = null;
            this._midiAccess = null;
        }
        
        this._inputDevices.clear();
        this._outputDevices.clear();
        this._isEnabled = false;
    }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const midiService = MidiService.getInstance();
