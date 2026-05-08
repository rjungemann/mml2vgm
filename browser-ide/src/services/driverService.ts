/**
 * Driver Service
 *
 * Manages external MML driver support for the browser IDE.
 * This service provides access to external drivers (mucom, PMD, MoonDriver, M98, Muap)
 * via WASM bindings.
 */

import type { MMLLanguage, SoundChip, CompileOptions, CompileResult } from '@/types';
import { wasmService } from './wasmService';

// ============================================================================
// Types
// ============================================================================

/** Driver information */
export interface DriverInfo {
    id: string;
    displayName: string;
    extensions: string[];
    description: string;
    version: string;
    targetPlatform: string;
    isLoaded: boolean;
}

/** Format detection result */
export interface DriverDetectionResult {
    driverId: string | null;
    confidence: number; // 0-100
}

/** Compile result from external driver */
export interface ExternalCompileResult extends CompileResult {
    driverId: string;
    partCount: number;
    commandCount: number;
    durationSamples: number;
    durationSeconds: number;
    chipsUsed: SoundChip[];
}

/** Diagnostic from validation */
export interface ExternalDiagnostic {
    message: string;
    severity: 'error' | 'warning' | 'info' | 'hint';
    line: number;
    column: number;
    length: number;
}

/** Token from tokenization */
export interface ExternalToken {
    type: string;
    value: string;
    line: number;
    column: number;
    length: number;
}

/** A single autocompletion suggestion for the editor */
export interface CompletionSuggestion {
    label: string;
    insertText: string;
    detail?: string;
    documentation?: string;
    kind: 'keyword' | 'snippet' | 'value';
}

// ============================================================================
// Per-driver completion data
// ============================================================================

const GWI_COMPLETIONS: CompletionSuggestion[] = [
    // Chip part directives
    { label: 'PartYM2612', insertText: 'PartYM2612', detail: 'Define YM2612 (FM) part', kind: 'keyword' },
    { label: 'PartSN76489', insertText: 'PartSN76489', detail: 'Define SN76489 (PSG) part', kind: 'keyword' },
    { label: 'PartYM2151', insertText: 'PartYM2151', detail: 'Define YM2151 (OPM) part', kind: 'keyword' },
    { label: 'PartYM2608', insertText: 'PartYM2608', detail: 'Define YM2608 (OPNA) part', kind: 'keyword' },
    { label: 'PartYM2610', insertText: 'PartYM2610', detail: 'Define YM2610 (OPNB) part', kind: 'keyword' },
    { label: 'PartYM2413', insertText: 'PartYM2413', detail: 'Define YM2413 (OPLL) part', kind: 'keyword' },
    { label: 'PartAY8910', insertText: 'PartAY8910', detail: 'Define AY-3-8910 (PSG) part', kind: 'keyword' },
    { label: 'PartNESAPU', insertText: 'PartNESAPU', detail: 'Define NES APU part', kind: 'keyword' },
    { label: 'PartDMG', insertText: 'PartDMG', detail: 'Define DMG (Game Boy) part', kind: 'keyword' },
    // Tone/envelope blocks
    { label: '@', insertText: '@ ', detail: 'Tone number select', kind: 'keyword' },
    { label: "'@ M", insertText: "'@ M\n  ALG=0 FB=0\n  ; operator data here\n.", detail: "FM tone macro block", kind: 'snippet' },
    { label: "'@ E", insertText: "'@ E\n  AL=0 AR=15 DR=0 SR=0 RR=5 SL=0 TL=0\n.", detail: "Envelope macro", kind: 'snippet' },
    { label: "'@ P", insertText: "'@ P\n  ; PSG envelope data\n.", detail: "PSG envelope macro", kind: 'snippet' },
    { label: "'@ A", insertText: "'@ A\n  ; arpeggio data\n.", detail: "Arpeggio macro", kind: 'snippet' },
    // Loop/flow
    { label: '/[', insertText: '/[', detail: 'Loop start', kind: 'keyword' },
    { label: '/]', insertText: '/]', detail: 'Loop end', kind: 'keyword' },
    // Note/octave/volume commands
    { label: 'o', insertText: 'o4', detail: 'Set octave (o4)', kind: 'snippet' },
    { label: 'v', insertText: 'v100', detail: 'Set volume (v100)', kind: 'snippet' },
    { label: 't', insertText: 't120', detail: 'Set tempo (t120)', kind: 'snippet' },
    { label: 'q', insertText: 'q8', detail: 'Set quantize (q8)', kind: 'snippet' },
    { label: 'l', insertText: 'l8', detail: 'Set default note length (l8)', kind: 'snippet' },
    { label: 'r', insertText: 'r4', detail: 'Rest', kind: 'keyword' },
    { label: '>', insertText: '>', detail: 'Octave up', kind: 'keyword' },
    { label: '<', insertText: '<', detail: 'Octave down', kind: 'keyword' },
    // Part names
    { label: 'A', insertText: 'A', detail: 'Part A', kind: 'value' },
    { label: 'B', insertText: 'B', detail: 'Part B', kind: 'value' },
    { label: 'C', insertText: 'C', detail: 'Part C', kind: 'value' },
    { label: 'D', insertText: 'D', detail: 'Part D', kind: 'value' },
    { label: 'E', insertText: 'E', detail: 'Part E', kind: 'value' },
    { label: 'F', insertText: 'F', detail: 'Part F', kind: 'value' },
];

const M98_COMPLETIONS: CompletionSuggestion[] = [
    { label: 'M98', insertText: 'M98', detail: 'M98 music header', kind: 'keyword' },
    { label: 'OPNA', insertText: 'OPNA', detail: 'OPNA chip directive', kind: 'keyword' },
    { label: '#TEMPO', insertText: '#TEMPO ', detail: 'Set tempo', kind: 'keyword' },
    { label: '#OCTAVE', insertText: '#OCTAVE ', detail: 'Set octave', kind: 'keyword' },
    { label: '#VOLUME', insertText: '#VOLUME ', detail: 'Set volume', kind: 'keyword' },
    { label: '#CHANNEL', insertText: '#CHANNEL ', detail: 'Set channel', kind: 'keyword' },
    { label: 'o', insertText: 'o4', detail: 'Set octave', kind: 'snippet' },
    { label: 'v', insertText: 'v100', detail: 'Set volume', kind: 'snippet' },
    { label: 't', insertText: 't120', detail: 'Set tempo', kind: 'snippet' },
    { label: 'r', insertText: 'r4', detail: 'Rest', kind: 'keyword' },
    { label: '@', insertText: '@ ', detail: 'Voice select', kind: 'keyword' },
];

const MUCOM_COMPLETIONS: CompletionSuggestion[] = [
    { label: '#MUCOM88', insertText: '#MUCOM88', detail: 'MUCOM88 header', kind: 'keyword' },
    { label: '#VOICE', insertText: '#VOICE\n  ; FM voice data here\n.', detail: 'FM voice definition block', kind: 'snippet' },
    { label: '#TEMPO', insertText: '#TEMPO ', detail: 'Set tempo', kind: 'keyword' },
    { label: '#OCTAVE', insertText: '#OCTAVE ', detail: 'Set octave', kind: 'keyword' },
    { label: '#W', insertText: '#W ', detail: 'Wait command', kind: 'keyword' },
    { label: '#INCLUDE', insertText: "#INCLUDE '", detail: 'Include file', kind: 'keyword' },
    { label: 'A', insertText: 'A', detail: 'Channel A (FM)', kind: 'value' },
    { label: 'B', insertText: 'B', detail: 'Channel B (FM)', kind: 'value' },
    { label: 'C', insertText: 'C', detail: 'Channel C (FM)', kind: 'value' },
    { label: 'G', insertText: 'G', detail: 'Channel G (SSG)', kind: 'value' },
    { label: 'H', insertText: 'H', detail: 'Channel H (SSG)', kind: 'value' },
    { label: 'I', insertText: 'I', detail: 'Channel I (SSG)', kind: 'value' },
    { label: 'o', insertText: 'o4', detail: 'Set octave', kind: 'snippet' },
    { label: 'v', insertText: 'v100', detail: 'Set volume', kind: 'snippet' },
    { label: 't', insertText: 't120', detail: 'Set tempo', kind: 'snippet' },
    { label: 'r', insertText: 'r4', detail: 'Rest', kind: 'keyword' },
    { label: '@', insertText: '@0', detail: 'Voice number select', kind: 'keyword' },
];

const MOONDRIVER_COMPLETIONS: CompletionSuggestion[] = [
    { label: '#MD', insertText: '#MD', detail: 'MoonDriver header', kind: 'keyword' },
    { label: '#OPN2', insertText: '#OPN2', detail: 'Use YM2612 (OPN2) chip', kind: 'keyword' },
    { label: '#OPNA', insertText: '#OPNA', detail: 'Use YM2608 (OPNA) chip', kind: 'keyword' },
    { label: '#OPN3', insertText: '#OPN3', detail: 'Use YMF288 (OPN3) chip', kind: 'keyword' },
    { label: '#INCLUDE', insertText: "#INCLUDE '", detail: 'Include file', kind: 'keyword' },
    { label: '#TEMPO', insertText: '#TEMPO ', detail: 'Set tempo', kind: 'keyword' },
    { label: '#TRACK', insertText: '#TRACK ', detail: 'Track definition', kind: 'keyword' },
    { label: 'o', insertText: 'o4', detail: 'Set octave', kind: 'snippet' },
    { label: 'v', insertText: 'v100', detail: 'Set volume', kind: 'snippet' },
    { label: 't', insertText: 't120', detail: 'Set tempo', kind: 'snippet' },
    { label: 'r', insertText: 'r4', detail: 'Rest', kind: 'keyword' },
    { label: '@', insertText: '@0', detail: 'Voice select', kind: 'keyword' },
];

const PMD_COMPLETIONS: CompletionSuggestion[] = [
    { label: '@MUSIC', insertText: '@MUSIC', detail: 'Music data section', kind: 'keyword' },
    { label: '@PPZ', insertText: '@PPZ', detail: 'PPZ/ADPCM channel section', kind: 'keyword' },
    { label: '@RHYTHM', insertText: '@RHYTHM', detail: 'Rhythm track section', kind: 'keyword' },
    { label: '@ADPCM', insertText: '@ADPCM', detail: 'ADPCM section', kind: 'keyword' },
    { label: '@@', insertText: '@@', detail: 'End of part marker', kind: 'keyword' },
    { label: '#TEMPO', insertText: '#TEMPO ', detail: 'Set tempo', kind: 'keyword' },
    { label: '#OCTAVE', insertText: '#OCTAVE ', detail: 'Set octave direction', kind: 'keyword' },
    { label: '#TITLE', insertText: "#TITLE '", detail: 'Song title metadata', kind: 'keyword' },
    { label: '#MEMO', insertText: "#MEMO '", detail: 'Comment metadata', kind: 'keyword' },
    { label: 'BD', insertText: 'BD', detail: 'Bass drum rhythm step', kind: 'value' },
    { label: 'SD', insertText: 'SD', detail: 'Snare drum rhythm step', kind: 'value' },
    { label: 'TOM', insertText: 'TOM', detail: 'Tom-tom rhythm step', kind: 'value' },
    { label: 'HH', insertText: 'HH', detail: 'Hi-hat rhythm step', kind: 'value' },
    { label: 'CYM', insertText: 'CYM', detail: 'Cymbal rhythm step', kind: 'value' },
    { label: 'RIM', insertText: 'RIM', detail: 'Rim shot rhythm step', kind: 'value' },
    { label: 'o', insertText: 'o4', detail: 'Set octave', kind: 'snippet' },
    { label: 'v', insertText: 'v100', detail: 'Set volume', kind: 'snippet' },
    { label: 't', insertText: 't120', detail: 'Set tempo', kind: 'snippet' },
    { label: 'r', insertText: 'r4', detail: 'Rest', kind: 'keyword' },
    { label: '@', insertText: '@0', detail: 'Voice select', kind: 'keyword' },
];

const MUAP_COMPLETIONS: CompletionSuggestion[] = [
    { label: '@FM', insertText: '@FM', detail: 'FM synthesis section', kind: 'keyword' },
    { label: '@SSG', insertText: '@SSG', detail: 'SSG (PSG) section', kind: 'keyword' },
    { label: '@RHYTHM', insertText: '@RHYTHM', detail: 'Rhythm section', kind: 'keyword' },
    { label: '@ADPCM', insertText: '@ADPCM', detail: 'ADPCM section', kind: 'keyword' },
    { label: '@OPNA', insertText: '@OPNA', detail: 'OPNA chip section', kind: 'keyword' },
    { label: '#TEMPO', insertText: '#TEMPO ', detail: 'Set tempo', kind: 'keyword' },
    { label: '#OCTAVE', insertText: '#OCTAVE ', detail: 'Set octave', kind: 'keyword' },
    { label: 'o', insertText: 'o4', detail: 'Set octave', kind: 'snippet' },
    { label: 'v', insertText: 'v100', detail: 'Set volume', kind: 'snippet' },
    { label: 't', insertText: 't120', detail: 'Set tempo', kind: 'snippet' },
    { label: 'r', insertText: 'r4', detail: 'Rest', kind: 'keyword' },
    { label: '@', insertText: '@0', detail: 'Voice select', kind: 'keyword' },
];

const DRIVER_COMPLETIONS: Record<string, CompletionSuggestion[]> = {
    gwi: GWI_COMPLETIONS,
    mml: GWI_COMPLETIONS,
    m98: M98_COMPLETIONS,
    muc: MUCOM_COMPLETIONS,
    mucom: MUCOM_COMPLETIONS,
    mdl: MOONDRIVER_COMPLETIONS,
    moondriver: MOONDRIVER_COMPLETIONS,
    mus: PMD_COMPLETIONS,
    pmd: PMD_COMPLETIONS,
    muap: MUAP_COMPLETIONS,
};

// ============================================================================
// Driver Service Class
// ============================================================================

/**
 * Driver Service
 *
 * Provides access to external MML drivers for compilation, validation, and tokenization.
 * Drivers are loaded lazily and cached for performance.
 */
export class DriverService {
    private static instance: DriverService | null = null;

    private wasmRegistry: any = null;
    private loadedDrivers: Map<string, any> = new Map();
    private loadingPromises: Map<string, Promise<any>> = new Map();
    private isInitialized: boolean = false;

    // Event listeners for driver loading
    private listeners: Array<(event: { type: string; driverId?: string; error?: string }) => void> = [];

    // ========================================================================
    // Singleton
    // ========================================================================

    public static getInstance(): DriverService {
        if (!DriverService.instance) {
            DriverService.instance = new DriverService();
        }
        return DriverService.instance;
    }

    private constructor() {}

    // ========================================================================
    // Initialization
    // ========================================================================

    /**
     * Initialize the driver service by loading the WASM driver registry.
     */
    public async initialize(): Promise<void> {
        if (this.isInitialized) return;

        try {
            // Ensure WASM is loaded
            await wasmService.initialize();

            // Get the driver registry from WASM
            const wasm = await wasmService.getWasmModule();
            if (wasm && wasm.DriverRegistry) {
                this.wasmRegistry = await wasm.DriverRegistry.new();
                this.isInitialized = true;
            } else {
                console.warn('DriverRegistry not available in WASM module');
                this.isInitialized = true; // Mark as initialized but without registry
            }
        } catch (error) {
            console.error('Failed to initialize DriverService:', error);
            this.isInitialized = true; // Mark as initialized to prevent retries
            throw error;
        }
    }

    // ========================================================================
    // Driver Management
    // ========================================================================

    /**
     * List all available drivers.
     */
    public async listDrivers(): Promise<DriverInfo[]> {
        await this.initialize();

        if (!this.wasmRegistry) {
            return [];
        }

        try {
            const driversJson = this.wasmRegistry.list_drivers();
            const drivers: DriverInfo[] = JSON.parse(driversJson);
            return drivers.map(d => ({
                ...d,
                isLoaded: this.loadedDrivers.has(d.id),
            }));
        } catch (error) {
            console.error('Failed to list drivers:', error);
            return [];
        }
    }

    /**
     * Check if a driver is available.
     */
    public async hasDriver(id: string): Promise<boolean> {
        await this.initialize();

        if (!this.wasmRegistry) {
            return false;
        }

        try {
            return this.wasmRegistry.has_driver(id);
        } catch (error) {
            console.error(`Failed to check driver ${id}:`, error);
            return false;
        }
    }

    /**
     * Check if a driver is loaded.
     */
    public isDriverLoaded(id: string): boolean {
        return this.loadedDrivers.has(id);
    }

    // ========================================================================
    // Format Detection
    // ========================================================================

    /**
     * Detect the driver for a file based on content and/or filename.
     */
    public async detectFormat(
        content: string,
        filename: string = ''
    ): Promise<DriverDetectionResult> {
        await this.initialize();

        if (!this.wasmRegistry) {
            return { driverId: null, confidence: 0 };
        }

        try {
            const resultJson = this.wasmRegistry.detect_format(content, filename || null);
            const result = JSON.parse(resultJson);
            return {
                driverId: result.driverId || null,
                confidence: result.confidence || 0,
            };
        } catch (error) {
            console.error('Failed to detect format:', error);
            return { driverId: null, confidence: 0 };
        }
    }

    /**
     * Get the driver for a file extension.
     */
    public async getDriverByExtension(extension: string): Promise<DriverInfo | null> {
        await this.initialize();

        if (!this.wasmRegistry) {
            return null;
        }

        try {
            const ext = extension.startsWith('.') ? extension : `.${extension}`;
            const driverJson = this.wasmRegistry.get_driver_by_extension(ext);
            if (!driverJson || driverJson === 'null') {
                return null;
            }
            const driver = JSON.parse(driverJson);
            return {
                ...driver,
                isLoaded: this.loadedDrivers.has(driver.id),
            };
        } catch (error) {
            console.error(`Failed to get driver by extension ${extension}:`, error);
            return null;
        }
    }

    /**
     * Get the appropriate driver ID for a file.
     * First tries extension, then content detection, then falls back to default.
     */
    public async getDriverForFile(filename: string, content?: string): Promise<string | null> {
        // Try extension first
        const ext = filename.split('.').pop()?.toLowerCase();
        if (ext) {
            const driverByExt = await this.getDriverByExtension(ext);
            if (driverByExt) {
                return driverByExt.id;
            }
        }

        // Try content detection
        if (content) {
            const detection = await this.detectFormat(content, filename);
            if (detection.driverId && detection.confidence >= 30) {
                return detection.driverId;
            }
        }

        return null;
    }

    // ========================================================================
    // Compilation
    // ========================================================================

    /**
     * Compile MML content using a specific driver.
     */
    public async compile(
        content: string,
        driverId: string,
        options: Partial<CompileOptions> = {}
    ): Promise<ExternalCompileResult> {
        await this.initialize();

        if (!this.wasmRegistry) {
            throw new Error('Driver service not initialized');
        }

        try {
            // Create WASM compile options
            const wasmOptions = this.wasmRegistry.JsDriverCompileOptions?.new?.();
            if (!wasmOptions) {
                throw new Error(`Driver ${driverId} compile options not available`);
            }

            // Set options
            if (options.format) {
                wasmOptions.set_output_format(options.format);
            }
            if (options.target_chips) {
                // For now, we'll use the default chips for the driver
                // TODO: Pass target chips through
            }

            // Use the existing WASM compile for GWI, M98, Mucom, MoonDriver, PMD, and Muap
            // For now, all use the same underlying compiler with appropriate chips
            if (driverId === 'gwi' || driverId === 'm98' || driverId === 'mucom' || driverId === 'moondriver' || driverId === 'pmd' || driverId === 'muap') {
                // Use the existing WASM compile
                const targetChips = driverId === 'm98' 
                    ? ['YM2608', 'SN76489'] 
                    : driverId === 'mucom' 
                        ? ['YM2612', 'SN76489'] 
                        : driverId === 'moondriver' || driverId === 'pmd' || driverId === 'muap'
                            ? ['YM2608', 'SN76489']
                            : undefined;
                
                const result = await wasmService.compileMML(content, {
                    format: options.format || 'vgm',
                    target_chips: targetChips,
                });

                return {
                    ...result,
                    driverId,
                    partCount: result.part_count || 0,
                    commandCount: result.command_count || 0,
                    durationSamples: result.duration_samples || 0,
                    durationSeconds: result.duration_seconds || 0,
                    chipsUsed: result.chips_used || [],
                };
            } else {
                // For external drivers, use the driver_compile WASM function
                // This will fail until the drivers are implemented
                throw new Error(`Driver ${driverId} is not yet implemented in Rust`);
            }
        } catch (error: any) {
            console.error(`Compilation failed for driver ${driverId}:`, error);
            return {
                data: undefined,
                errors: [{
                    message: error.message || String(error),
                    line: 0,
                    column: 0,
                    length: 0,
                }],
                warnings: [],
                driverId,
                partCount: 0,
                commandCount: 0,
                durationSamples: 0,
                durationSeconds: 0,
                chipsUsed: [],
            };
        }
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /**
     * Validate MML content using a specific driver.
     */
    public async validate(
        content: string,
        driverId: string
    ): Promise<ExternalDiagnostic[]> {
        await this.initialize();

        if (!this.wasmRegistry) {
            throw new Error('Driver service not initialized');
        }

        try {
            if (driverId === 'gwi' || driverId === 'm98' || driverId === 'mucom' || driverId === 'moondriver' || driverId === 'pmd' || driverId === 'muap') {
                // Use the existing WASM validate for GWI/M98/Mucom/MoonDriver/PMD/Muap
                const isValid = await wasmService.validateMML(content);
                return isValid ? [] : [{
                    message: 'Validation failed',
                    severity: 'error',
                    line: 0,
                    column: 0,
                    length: 0,
                }];
            } else {
                // For external drivers, we would use driver_validate
                // For now, return empty (no validation)
                return [];
            }
        } catch (error: any) {
            console.error(`Validation failed for driver ${driverId}:`, error);
            return [{
                message: error.message || String(error),
                severity: 'error',
                line: 0,
                column: 0,
                length: 0,
            }];
        }
    }

    // ========================================================================
    // Tokenization
    // ========================================================================

    /**
     * Tokenize MML content using a specific driver for syntax highlighting.
     */
    public async tokenize(
        content: string,
        driverId: string
    ): Promise<ExternalToken[]> {
        await this.initialize();

        if (!this.wasmRegistry) {
            throw new Error('Driver service not initialized');
        }

        try {
            if (driverId === 'gwi' || driverId === 'm98' || driverId === 'mucom' || driverId === 'moondriver' || driverId === 'pmd' || driverId === 'muap') {
                // Use the existing WASM tokenize for GWI/M98/Mucom/MoonDriver/PMD/Muap
                const tokensJson = await wasmService.tokenize(content);
                const tokens: ExternalToken[] = JSON.parse(tokensJson);
                return tokens;
            } else {
                // For external drivers, we would use driver_tokenize
                // For now, return empty
                return [];
            }
        } catch (error: any) {
            console.error(`Tokenization failed for driver ${driverId}:`, error);
            return [];
        }
    }

    // ========================================================================
    // Event Handling
    // ========================================================================

    /**
     * Subscribe to driver service events.
     */
    public subscribe(listener: (event: { type: string; driverId?: string; error?: string }) => void): () => void {
        this.listeners.push(listener);
        return () => {
            const index = this.listeners.indexOf(listener);
            if (index !== -1) {
                this.listeners.splice(index, 1);
            }
        };
    }

    /**
     * Emit an event to all subscribers.
     */
    private emitEvent(event: { type: string; driverId?: string; error?: string }): void {
        for (const listener of this.listeners) {
            try {
                listener(event);
            } catch (error) {
                console.error('Error in driver service listener:', error);
            }
        }
    }

    // ========================================================================
    // Completions
    // ========================================================================

    /**
     * Completion item shape (subset of Monaco's CompletionItem, without range).
     */

    /**
     * Return format-specific completion suggestions for a driver.
     *
     * The `prefix` is the word currently being typed (may be empty).
     * Returns a list of {label, insertText, detail, documentation} objects
     * that MonacoEditor can turn into Monaco CompletionItems.
     */
    public getCompletions(driverId: string, prefix: string): CompletionSuggestion[] {
        const all = DRIVER_COMPLETIONS[driverId] ?? DRIVER_COMPLETIONS['gwi'];
        if (!prefix) return all;
        const lower = prefix.toLowerCase();
        return all.filter(s => s.label.toLowerCase().startsWith(lower));
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /**
     * Get the best driver for a language ID.
     */
    public async getDriverForLanguage(language: MMLLanguage): Promise<string | null> {
        // Map language IDs to driver IDs
        const languageToDriver: Record<MMLLanguage, string> = {
            gwi: 'gwi',
            mml: 'gwi', // Generic MML falls back to GWI for now
            muc: 'mucom', // MUC files use mucom driver
            mdl: 'moondriver', // MDL is MoonDriver
            mus: 'pmd',
            m98: 'm98',
            muap: 'muap',
        };

        const driverId = languageToDriver[language];
        if (driverId && await this.hasDriver(driverId)) {
            return driverId;
        }

        return null;
    }

    /**
     * Get driver ID for format detection from extension.
     */
    public async getDriverForExtension(extension: string): Promise<string | null> {
        const driver = await this.getDriverByExtension(extension);
        return driver?.id || null;
    }
}

// ============================================================================
// Singleton Export
// ============================================================================

// Initialize the singleton instance
export const driverService = DriverService.getInstance();
