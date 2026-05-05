/**
 * Format Service
 * 
 * Manages multi-format MML support including format detection,
 * format-specific parsing, and compilation options.
 * 
 * This service provides the foundation for Phase 6 feature parity
 * with the .NET IDE's multi-format support.
 */

import type { MMLLanguage, SoundChip, CompileOptions, ChipInfo, FormatInfo } from '@/types';
import { wasmService } from './wasmService';

// ============================================================================
// Types
// ============================================================================

/** Format handler interface for different MML formats */
export interface FormatHandler {
    /** Unique identifier for this format */
    readonly id: MMLLanguage;
    
    /** Display name for this format */
    readonly displayName: string;
    
    /** File extensions associated with this format */
    readonly extensions: string[];
    
    /** Default file extension */
    readonly defaultExtension: string;
    
    /** Description of this format */
    readonly description: string;
    
    /** Target platform/chipset for this format */
    readonly targetPlatform: string;
    
    /** Default chips used by this format */
    readonly defaultChips: SoundChip[];
    
    /** Default output format for compilation */
    readonly defaultOutputFormat: 'vgm' | 'xgm' | 'xgm2' | 'zgm';
    
    /** Whether this format is natively supported (vs external driver) */
    readonly isNative: boolean;
    
    /** Whether external driver support is available in browser */
    readonly driverAvailable: boolean;
    
    /** Detect if content matches this format (content-based detection) */
    detectFromContent(content: string): number; // Returns confidence score 0-100
    
    /** Get compile options for this format */
    getCompileOptions(): Promise<CompileOptions>;
    
    /** Get syntax highlighting configuration for this format */
    getSyntaxConfig(): MonacoSyntaxConfig;
    
    /** Get token patterns for additional syntax highlighting */
    getTokenPatterns?(): TokenPattern[];
}

/** Monaco syntax configuration */
export interface MonacoSyntaxConfig {
    /** Language ID for Monaco */
    languageId: string;
    
    /** Token provider configuration */
    tokens: {
        // Token types for this format
        [tokenType: string]: {
            pattern: string | RegExp;
            type: string;
        };
    };
    
    /** Additional keywords */
    keywords?: string[];
    
    /** Additional operators */
    operators?: string[];
    
    /** Built-in commands */
    builtins?: string[];
}

/** Token pattern for custom syntax highlighting */
export interface TokenPattern {
    regex: RegExp;
    tokenType: string;
}

/** Format detection result */
export interface FormatDetectionResult {
    format: MMLLanguage;
    confidence: number; // 0-100
    detectedFrom: 'extension' | 'content' | 'default';
    handler: FormatHandler;
}

/** Format service state */
export interface FormatServiceState {
    supportedFormats: MMLLanguage[];
    currentFormat: MMLLanguage | null;
    isLoading: boolean;
    error: string | null;
}

// ============================================================================
// Format Handlers
// ============================================================================

/**
 * GWI Format Handler (mml2vgm native format)
 * This is the primary format supported by the Rust compiler.
 */
const gwiHandler: FormatHandler = {
    id: 'gwi',
    displayName: 'mml2vgm (GWI)',
    extensions: ['.gwi', '.txt'],
    defaultExtension: '.gwi',
    description: 'mml2vgm native MML format',
    targetPlatform: 'Multi-platform',
    defaultChips: ['YM2608', 'SN76489'],
    defaultOutputFormat: 'vgm',
    isNative: true,
    driverAvailable: true, // Native support via WASM
    
    detectFromContent(content: string): number {
        // Check for @ commands which are specific to mml2vgm format
        const atCommands = (content.match(/@\d+/g) || []).length;
        const totalCommands = (content.match(/[a-zA-Z]/g) || []).length;
        
        if (atCommands > 0 && totalCommands > 0) {
            return Math.min(100, 50 + (atCommands * 100 / Math.max(1, totalCommands)));
        }
        return 0;
    },
    
    async getCompileOptions(): Promise<CompileOptions> {
        const options = await wasmService.compileOptionsForFormat('vgm');
        return {
            ...options,
            target_chips: [...this.defaultChips],
        };
    },
    
    getSyntaxConfig(): MonacoSyntaxConfig {
        return {
            languageId: 'gwi',
            tokens: {
                // Part commands
                partCommand: {
                    pattern: /@\d+/,
                    type: 'keyword.part',
                },
                // Note commands
                note: {
                    pattern: /[a-g][+#-]?\d*/i,
                    type: 'string.note',
                },
                // Duration
                duration: {
                    pattern: /[lL]\d+/,
                    type: 'keyword.duration',
                },
                // Volume
                volume: {
                    pattern: /[vV]\d+/,
                    type: 'keyword.volume',
                },
                // Octave
                octave: {
                    pattern: /[oO]\d+/,
                    type: 'keyword.octave',
                },
                // Tempo
                tempo: {
                    pattern: /[tT]\d+/,
                    type: 'keyword.tempo',
                },
                // Length
                length: {
                    pattern: /[lL]\d+/,
                    type: 'keyword.length',
                },
                // Rest
                rest: {
                    pattern: /r\d*/i,
                    type: 'keyword.rest',
                },
                // Tie
                tie: {
                    pattern: /_/,
                    type: 'operator.tie',
                },
                // Octave up/down
                octaveUp: {
                    pattern: />/,
                    type: 'operator.octave',
                },
                octaveDown: {
                    pattern: /</,
                    type: 'operator.octave',
                },
            },
            keywords: ['@', 'v', 'o', 'l', 't', 'q', 'r', '>'], 
            operators: ['+', '-', '=', '*', '/', '_', '>', '<'],
            builtins: [],
        };
    },
};

/**
 * MUC Format Handler (mucom88 format)
 * This format is for Sega Mega Drive music.
 */
const mucHandler: FormatHandler = {
    id: 'muc',
    displayName: 'mucom88 (MUC)',
    extensions: ['.muc'],
    defaultExtension: '.muc',
    description: 'mucom88 MML format for Sega Mega Drive',
    targetPlatform: 'Sega Mega Drive (YM2612 + SN76489)',
    defaultChips: ['YM2612', 'SN76489'],
    defaultOutputFormat: 'vgm',
    isNative: false,
    driverAvailable: true, // Implemented in Rust
    
    detectFromContent(content: string): number {
        // Check for mucom88-specific patterns
        const hasHash = content.includes('#');
        const hasMucomDirective = content.toLowerCase().includes('#mucom');
        const hasYm2612 = content.toLowerCase().includes('ym2612');
        
        if (hasMucomDirective) return 95;
        if (hasYm2612) return 80;
        if (hasHash) return 30;
        return 0;
    },
    
    async getCompileOptions(): Promise<CompileOptions> {
        // For now, use VGM with YM2612 + SN76489
        const options = await wasmService.compileOptionsForFormat('vgm');
        return {
            ...options,
            target_chips: [...this.defaultChips],
        };
    },
    
    getSyntaxConfig(): MonacoSyntaxConfig {
        return {
            languageId: 'muc',
            tokens: {
                // mucom88 directives
                directive: {
                    pattern: /#\w+/,
                    type: 'keyword.directive',
                },
                // Comments
                comment: {
                    pattern: /;.*$/,
                    type: 'comment',
                },
                // Note names
                note: {
                    pattern: /[A-G][+#-]?\d*/,
                    type: 'string.note',
                },
                // Rest
                rest: {
                    pattern: /R\d*/i,
                    type: 'keyword.rest',
                },
            },
            keywords: ['#MUCOM', '#OPM', '#PSG', '#PCM', '#TEMPO', '#LOOP'],
            operators: ['+', '-', '=', '*', '/'],
            builtins: [],
        };
    },
    
    getTokenPatterns(): TokenPattern[] {
        return [
            { regex: /@\w+/g, tokenType: 'keyword.part' },
        ];
    },
};

/**
 * MML Format Handler (Generic MML)
 * This is a generic MML format handler.
 */
const mmlHandler: FormatHandler = {
    id: 'mml',
    displayName: 'Generic MML',
    extensions: ['.mml'],
    defaultExtension: '.mml',
    description: 'Generic Music Macro Language format',
    targetPlatform: 'Multi-platform',
    defaultChips: ['YM2608'],
    defaultOutputFormat: 'vgm',
    isNative: false,
    driverAvailable: false,
    
    detectFromContent(content: string): number {
        // Generic MML detection - look for common MML patterns
        const hasNotes = /[a-g]\d*/i.test(content);
        const hasOctave = /o\d+/i.test(content);
        const hasLength = /l\d+/i.test(content);
        const hasVolume = /v\d+/i.test(content);
        
        let score = 0;
        if (hasNotes) score += 30;
        if (hasOctave) score += 25;
        if (hasLength) score += 20;
        if (hasVolume) score += 25;
        
        return Math.min(100, score);
    },
    
    async getCompileOptions(): Promise<CompileOptions> {
        const options = await wasmService.compileOptionsForFormat('vgm');
        return {
            ...options,
            target_chips: [...this.defaultChips],
        };
    },
    
    getSyntaxConfig(): MonacoSyntaxConfig {
        return {
            languageId: 'mml',
            tokens: {
                note: {
                    pattern: /[a-g][+#-]?\d*/i,
                    type: 'string.note',
                },
                rest: {
                    pattern: /r\d*/i,
                    type: 'keyword.rest',
                },
                duration: {
                    pattern: /l\d+/i,
                    type: 'keyword.duration',
                },
                volume: {
                    pattern: /v\d+/i,
                    type: 'keyword.volume',
                },
                octave: {
                    pattern: /o\d+/i,
                    type: 'keyword.octave',
                },
                tempo: {
                    pattern: /t\d+/i,
                    type: 'keyword.tempo',
                },
            },
            keywords: ['o', 'v', 'l', 't', 'q', 'r'],
            operators: ['+', '-', '=', '*', '/', '_', '>', '<'],
            builtins: [],
        };
    },
};

/**
 * MDL Format Handler (MoonDriver format)
 * This format is used by MoonDriver for various platforms.
 * Note: .mdl extension is also used by PMD, but MoonDriver has higher priority for non-PMD content.
 */
const mdlHandler: FormatHandler = {
    id: 'mdl',
    displayName: 'MoonDriver (MDL)',
    extensions: ['.mdl'],
    defaultExtension: '.mdl',
    description: 'MoonDriver MML format',
    targetPlatform: 'Multi-platform (OPN2/OPNA/OPN3)',
    defaultChips: ['YM2608'],
    defaultOutputFormat: 'vgm',
    isNative: false,
    driverAvailable: true,
    
    detectFromContent(content: string): number {
        // Check for MoonDriver-specific patterns
        const hasMoonDriver = content.toLowerCase().includes('moondriver');
        const hasMd = content.toLowerCase().includes('#md');
        
        if (hasMoonDriver) return 90;
        if (hasMd) return 70;
        return 0;
    },
    
    async getCompileOptions(): Promise<CompileOptions> {
        const options = await wasmService.compileOptionsForFormat('vgm');
        return {
            ...options,
            target_chips: [...this.defaultChips],
        };
    },
    
    getSyntaxConfig(): MonacoSyntaxConfig {
        return {
            languageId: 'mdl',
            tokens: {
                directive: {
                    pattern: /#\w+/,
                    type: 'keyword.directive',
                },
                note: {
                    pattern: /[A-G][+#-]?\d*/i,
                    type: 'string.note',
                },
                rest: {
                    pattern: /R\d*/i,
                    type: 'keyword.rest',
                },
                comment: {
                    pattern: /\/\/.*$/,
                    type: 'comment',
                },
            },
            keywords: ['#MD', '#OPN2', '#OPNA', '#OPN3', '#TEMPO'],
            operators: ['+', '-', '=', '*', '/'],
            builtins: [],
        };
    },
};

/**
 * MUS Format Handler (PMD format)
 * This is the PMD format for NEC PC-9801.
 */
const musHandler: FormatHandler = {
    id: 'mus',
    displayName: 'PMD (MUS)',
    extensions: ['.mus'],
    defaultExtension: '.mus',
    description: 'PMD MML format for NEC PC-9801',
    targetPlatform: 'NEC PC-9801 (YM2203/YM2608)',
    defaultChips: ['YM2608'],
    defaultOutputFormat: 'vgm',
    isNative: false,
    driverAvailable: true,
    
    detectFromContent(content: string): number {
        // Check for PMD-specific patterns
        const hasPmd = content.toLowerCase().includes('pmd');
        const hasPpz = content.toLowerCase().includes('ppz');
        
        if (hasPmd || hasPpz) return 85;
        return 0;
    },
    
    async getCompileOptions(): Promise<CompileOptions> {
        const options = await wasmService.compileOptionsForFormat('vgm');
        return {
            ...options,
            target_chips: [...this.defaultChips],
        };
    },
    
    getSyntaxConfig(): MonacoSyntaxConfig {
        return {
            languageId: 'mus',
            tokens: {
                directive: {
                    pattern: /@\w+/,
                    type: 'keyword.directive',
                },
                note: {
                    pattern: /[A-G][+#-]?\d*/i,
                    type: 'string.note',
                },
                rest: {
                    pattern: /R\d*/i,
                    type: 'keyword.rest',
                },
                comment: {
                    pattern: /\*.*\*/,
                    type: 'comment',
                },
            },
            keywords: ['@MUSIC', '@PPZ', '@TEMPO', '@VOLUME'],
            operators: ['+', '-', '=', '*', '/'],
            builtins: [],
        };
    },
// Format Registry
// ============================================================================
};

/**
 * M98 Format Handler (M98 format for NEC PC-9801)
 * Simplified MML format for PC-9801.
 */
const m98Handler: FormatHandler = {
    id: 'm98',
    displayName: 'M98 (PC-9801)',
    extensions: ['.m98'],
    defaultExtension: '.m98',
    description: 'M98 MML format for NEC PC-9801',
    targetPlatform: 'NEC PC-9801 (YM2203/YM2608)',
    defaultChips: ['YM2608'],
    defaultOutputFormat: 'vgm',
    isNative: false,
    driverAvailable: true,

    detectFromContent(content: string): number {
        const lines = content.split('\n');
        const hasM98Directive = lines.some(line =>
            line.trim().toLowerCase().startsWith('m98')
        );
        const hasPc98Pattern = content.toLowerCase().includes('pc-98');

        if (hasM98Directive) return 95;
        if (hasPc98Pattern) return 70;
        return 0;
    },

    async getCompileOptions(): Promise<CompileOptions> {
        const options = await wasmService.compileOptionsForFormat('vgm');
        return {
            ...options,
            target_chips: [...this.defaultChips],
        };
    },

    getSyntaxConfig(): MonacoSyntaxConfig {
        return {
            languageId: 'm98',
            tokens: {
                directive: {
                    pattern: /#[A-Za-z]+/,
                    type: 'keyword.directive',
                },
                note: {
                    pattern: /[A-G][+#-]?\d*/i,
                    type: 'string.note',
                },
                rest: {
                    pattern: /R\d*/i,
                    type: 'keyword.rest',
                },
                comment: {
                    pattern: /;.*$/,
                    type: 'comment',
                },
            },
            keywords: ['M98', 'YM2203', 'YM2608', 'OPNA'],
            operators: ['+', '-', '=', '*', '/'],
            builtins: [],
        };
    },
};

/**
 * Muap Format Handler (Muap format for YM2608/OPNA)
 */
const muapHandler: FormatHandler = {
    id: 'muap',
    displayName: 'Muap (OPNA)',
    extensions: ['.muap'],
    defaultExtension: '.muap',
    description: 'Muap MML format for YM2608 (OPNA)',
    targetPlatform: 'YM2608 (OPNA) with FM, SSG, Rhythm, and ADPCM',
    defaultChips: ['YM2608'],
    defaultOutputFormat: 'vgm',
    isNative: false,
    driverAvailable: true,

    detectFromContent(content: string): number {
        const hasMuap = content.toLowerCase().includes('muap');
        const hasOpna = content.toLowerCase().includes('opna');
        const hasYm2608 = content.toLowerCase().includes('ym2608');

        if (hasMuap) return 90;
        if (hasOpna || hasYm2608) return 75;
        return 0;
    },

    async getCompileOptions(): Promise<CompileOptions> {
        const options = await wasmService.compileOptionsForFormat('vgm');
        return {
            ...options,
            target_chips: [...this.defaultChips],
        };
    },

    getSyntaxConfig(): MonacoSyntaxConfig {
        return {
            languageId: 'muap',
            tokens: {
                directive: {
                    pattern: /@\w+/,
                    type: 'keyword.directive',
                },
                note: {
                    pattern: /[A-G][+#-]?\d*/i,
                    type: 'string.note',
                },
                rest: {
                    pattern: /R\d*/i,
                    type: 'keyword.rest',
                },
                comment: {
                    pattern: /;.*$/,
                    type: 'comment',
                },
            },
            keywords: ['@FM', '@SSG', '@RHYTHM', '@ADPCM', '@OPNA'],
            operators: ['+', '-', '=', '*', '/'],
            builtins: [],
        };
    },
};

// ============================================================================
// Format Registry
// ========================================================================================================================================================
// Format Registry
// ============================================================================

/**
 * Registry of all supported format handlers.
 * This allows for dynamic registration and lookup of format handlers.
 */
class FormatRegistry {
    private handlers: Map<MMLLanguage, FormatHandler> = new Map();
    
    constructor() {
        // Register all built-in handlers
        this.registerHandler(gwiHandler);
        this.registerHandler(mucHandler);
        this.registerHandler(mmlHandler);
        this.registerHandler(mdlHandler);
        this.registerHandler(musHandler);
        this.registerHandler(m98Handler);
        this.registerHandler(muapHandler);
    }
    
    /**
     * Register a new format handler.
     */
    registerHandler(handler: FormatHandler): void {
        this.handlers.set(handler.id, handler);
    }
    
    /**
     * Get a handler by format ID.
     */
    getHandler(id: MMLLanguage): FormatHandler | undefined {
        return this.handlers.get(id);
    }
    
    /**
     * Get all registered handlers.
     */
    getAllHandlers(): FormatHandler[] {
        return Array.from(this.handlers.values());
    }
    
    /**
     * Get all supported format IDs.
     */
    getSupportedFormats(): MMLLanguage[] {
        return Array.from(this.handlers.keys());
    }
    
    /**
     * Detect format from file extension.
     */
    detectFromExtension(filename: string): FormatHandler | undefined {
        const ext = filename.split('.').pop()?.toLowerCase();
        if (!ext) return undefined;
        
        for (const handler of this.handlers.values()) {
            if (handler.extensions.some(e => e.toLowerCase() === `.${ext}` || e.toLowerCase() === ext)) {
                return handler;
            }
        }
        
        return undefined;
    }
    
    /**
     * Detect format from content (with fallback to extension).
     */
    detectFromContent(content: string, filename: string = ''): FormatDetectionResult {
        const extensionHandler = this.detectFromExtension(filename);
        
        // If we have a clear match from extension, return it with high confidence
        if (extensionHandler) {
            const contentConfidence = extensionHandler.detectFromContent(content);
            return {
                format: extensionHandler.id,
                confidence: Math.max(70, contentConfidence),
                detectedFrom: 'extension',
                handler: extensionHandler,
            };
        }
        
        // Try all handlers for content-based detection
        let bestMatch: FormatDetectionResult | null = null;
        
        for (const handler of this.handlers.values()) {
            const confidence = handler.detectFromContent(content);
            
            if (confidence > 0) {
                if (!bestMatch || confidence > bestMatch.confidence) {
                    bestMatch = {
                        format: handler.id,
                        confidence,
                        detectedFrom: 'content',
                        handler,
                    };
                }
            }
        }
        
        // If we found a match, return it
        if (bestMatch && bestMatch.confidence >= 30) {
            return bestMatch;
        }
        
        // Default to GWI format
        const defaultHandler = this.handlers.get('gwi') || this.handlers.values().next().value;
        return {
            format: defaultHandler.id,
            confidence: 0,
            detectedFrom: 'default',
            handler: defaultHandler,
        };
    }
}

// ============================================================================
// Format Service
// ============================================================================

/**
 * Format Service
 * 
 * Central service for managing MML format detection, parsing, and compilation.
 * This is the main entry point for multi-format support in the browser IDE.
 */
export class FormatService {
    private static instance: FormatService | null = null;
    
    private registry: FormatRegistry;
    private _state: FormatServiceState;
    
    // Event listeners
    private _listeners: Array<(state: FormatServiceState) => void> = [];
    
    // ========================================================================
    // Singleton
    // ========================================================================
    
    public static getInstance(): FormatService {
        if (!FormatService.instance) {
            FormatService.instance = new FormatService();
        }
        return FormatService.instance;
    }
    
    private constructor() {
        this.registry = new FormatRegistry();
        this._state = {
            supportedFormats: this.registry.getSupportedFormats(),
            currentFormat: null,
            isLoading: false,
            error: null,
        };
    }
    
    // ========================================================================
    // State Management
    // ========================================================================
    
    /**
     * Get current state.
     */
    public getState(): FormatServiceState {
        return { ...this._state };
    }
    
    /**
     * Subscribe to state changes.
     */
    public subscribe(callback: (state: FormatServiceState) => void): () => void {
        this._listeners.push(callback);
        // Send current state immediately
        callback(this._state);
        
        return () => {
            const index = this._listeners.indexOf(callback);
            if (index >= 0) {
                this._listeners.splice(index, 1);
            }
        };
    }
    
    private notifyListeners(): void {
        this._listeners.forEach(callback => callback(this._state));
    }
    
    // ========================================================================
    // Format Detection
    // ========================================================================
    
    /**
     * Detect format from file extension.
     */
    public detectFromExtension(filename: string): MMLLanguage | null {
        const handler = this.registry.detectFromExtension(filename);
        return handler?.id || null;
    }
    
    /**
     * Detect format from content and filename.
     * This is the primary method for format detection.
     */
    public detectFormat(content: string, filename: string = ''): FormatDetectionResult {
        return this.registry.detectFromContent(content, filename);
    }
    
    /**
     * Get handler for a specific format.
     */
    public getHandler(format: MMLLanguage): FormatHandler | undefined {
        return this.registry.getHandler(format);
    }
    
    /**
     * Get all supported formats.
     */
    public getSupportedFormats(): MMLLanguage[] {
        return this.registry.getSupportedFormats();
    }
    
    /**
     * Get all format handlers.
     */
    public getAllHandlers(): FormatHandler[] {
        return this.registry.getAllHandlers();
    }
    
    // ========================================================================
    // Format Information
    // ========================================================================
    
    /**
     * Get display name for a format.
     */
    public getDisplayName(format: MMLLanguage): string {
        const handler = this.registry.getHandler(format);
        return handler?.displayName || format;
    }
    
    /**
     * Get default extension for a format.
     */
    public getDefaultExtension(format: MMLLanguage): string {
        const handler = this.registry.getHandler(format);
        return handler?.defaultExtension || `.${format}`;
    }
    
    /**
     * Get description for a format.
     */
    public getDescription(format: MMLLanguage): string {
        const handler = this.registry.getHandler(format);
        return handler?.description || `MML format: ${format}`;
    }
    
    /**
     * Get default chips for a format.
     */
    public getDefaultChips(format: MMLLanguage): SoundChip[] {
        const handler = this.registry.getHandler(format);
        return handler?.defaultChips || ['YM2608', 'SN76489'];
    }
    
    /**
     * Get default output format for a format.
     */
    public getDefaultOutputFormat(format: MMLLanguage): 'vgm' | 'xgm' | 'xgm2' | 'zgm' {
        const handler = this.registry.getHandler(format);
        return handler?.defaultOutputFormat || 'vgm';
    }
    
    /**
     * Check if format is natively supported (can be compiled without external driver).
     */
    public isNativeFormat(format: MMLLanguage): boolean {
        const handler = this.registry.getHandler(format);
        return handler?.isNative || false;
    }
    
    /**
     * Check if external driver is available for this format.
     */
    public isDriverAvailable(format: MMLLanguage): boolean {
        const handler = this.registry.getHandler(format);
        return handler?.driverAvailable || false;
    }
    
    // ========================================================================
    // Compilation Support
    // ========================================================================
    
    /**
     * Get compile options for a specific format.
     */
    public async getCompileOptions(format: MMLLanguage): Promise<CompileOptions> {
        const handler = this.registry.getHandler(format);
        if (!handler) {
            // Fallback to default options
            return await wasmService.compileOptionsForFormat('vgm');
        }
        return handler.getCompileOptions();
    }
    
    /**
     * Get compile options with appropriate chips for the format.
     */
    public async getCompileOptionsWithChips(format: MMLLanguage, additionalChips?: SoundChip[]): Promise<CompileOptions> {
        const options = await this.getCompileOptions(format);
        const defaultChips = this.getDefaultChips(format);
        const chips = [...defaultChips];
        
        if (additionalChips) {
            for (const chip of additionalChips) {
                if (!chips.includes(chip)) {
                    chips.push(chip);
                }
            }
        }
        
        return {
            ...options,
            target_chips: chips,
        };
    }
    
    // ========================================================================
    // Syntax Highlighting Support
    // ========================================================================
    
    /**
     * Get syntax configuration for a format.
     */
    public getSyntaxConfig(format: MMLLanguage): MonacoSyntaxConfig | null {
        const handler = this.registry.getHandler(format);
        return handler?.getSyntaxConfig() || null;
    }
    
    /**
     * Get additional token patterns for a format.
     */
    public getTokenPatterns(format: MMLLanguage): TokenPattern[] {
        const handler = this.registry.getHandler(format);
        return handler?.getTokenPatterns?.() || [];
    }
    
    // ========================================================================
    // Utility Methods
    // ========================================================================
    
    /**
     * Get all extensions for all supported formats.
     */
    public getAllExtensions(): string[] {
        const extensions: Set<string> = new Set();
        for (const handler of this.registry.getAllHandlers()) {
            for (const ext of handler.extensions) {
                extensions.add(ext.toLowerCase());
            }
        }
        return Array.from(extensions);
    }
    
    /**
     * Check if a file extension is supported.
     */
    public isSupportedExtension(extension: string): boolean {
        const ext = extension.toLowerCase().startsWith('.') ? extension : `.${extension}`;
        return this.getAllExtensions().includes(ext);
    }
    
    /**
     * Get format info for display purposes.
     */
    public getFormatInfo(format: MMLLanguage): FormatInfoForDisplay {
        const handler = this.registry.getHandler(format);
        if (!handler) {
            return {
                id: format,
                displayName: format,
                description: `Unknown format: ${format}`,
                extensions: [`.${format}`],
                targetPlatform: 'Unknown',
                isNative: false,
                driverAvailable: false,
            };
        }
        
        return {
            id: handler.id,
            displayName: handler.displayName,
            description: handler.description,
            extensions: handler.extensions,
            targetPlatform: handler.targetPlatform,
            isNative: handler.isNative,
            driverAvailable: handler.driverAvailable,
        };
    }
}

// ============================================================================
// Export Types and Helper Functions
// ============================================================================

/** Format info for display purposes */
export interface FormatInfoForDisplay {
    id: MMLLanguage;
    displayName: string;
    description: string;
    extensions: string[];
    targetPlatform: string;
    isNative: boolean;
    driverAvailable: boolean;
}

/**
 * Singleton instance of the FormatService.
 * Use this for easy access throughout the application.
 */
export const formatService = FormatService.getInstance();

/**
 * Get format from file extension.
 * This is a standalone function for convenience.
 */
export function getFormatFromExtension(filename: string): MMLLanguage | null {
    return formatService.detectFromExtension(filename);
}

/**
 * Detect format from content and filename.
 * This is a standalone function for convenience.
 */
export function detectFormat(content: string, filename: string = ''): FormatDetectionResult {
    return formatService.detectFormat(content, filename);
}

export default FormatService;
