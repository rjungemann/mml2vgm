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
