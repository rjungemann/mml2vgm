/**
 * Script Service
 * 
 * Manages Python script integration via Pyodide for the browser IDE.
 * This provides the script execution capabilities similar to the .NET IDE's
 * IronPython integration.
 * 
 * This service is part of Phase 6 feature parity.
 */

import type { MMLLanguage, CompileOptions, SoundChip } from '@/types';

// ============================================================================
// Types
// ============================================================================

/** Script type */
export type ScriptType = 'python';

/** Script information */
export interface Script {
    id: string;
    name: string;
    content: string;
    type: ScriptType;
    path: string | null;
    isDirty: boolean;
    lastExecuted: Date | null;
    lastError: string | null;
}

/** Script execution context */
export interface ScriptContext {
    /** Current document content */
    documentContent: string;
    /** Current document language */
    documentLanguage: MMLLanguage;
    /** Compile options */
    compileOptions: CompileOptions;
    /** Current position in document (line, column) */
    position: { line: number; column: number };
    /** Selected text */
    selection: string | null;
}

/** Script execution result */
export interface ScriptResult {
    success: boolean;
    output: string | null;
    error: string | null;
    returnValue: unknown;
    executionTime: number;
}

/** Script function information */
export interface ScriptFunction {
    name: string;
    parameters: ScriptParameter[];
    description: string | null;
}

/** Script parameter information */
export interface ScriptParameter {
    name: string;
    type: string;
    description: string | null;
}

/** Script service state */
export interface ScriptServiceState {
    isEnabled: boolean;
    isSupported: boolean;
    isInitialized: boolean;
    scripts: Script[];
    currentScriptId: string | null;
    isExecuting: boolean;
    lastError: string | null;
}

/** Pyodide initialization options */
export interface PyodideOptions {
    /** Pyodide index URL for loading packages */
    indexURL?: string;
    /** Packages to load on initialization */
    packages?: string[];
}

// ============================================================================
// Default Pyodide Configuration
// ============================================================================

const DEFAULT_PYODIDE_INDEX_URL = 'https://cdn.jsdelivr.net/pyodide/v0.25.0/full/';
const DEFAULT_PACKAGES: string[] = [
    // Core packages for MML processing
    'numpy',
    // Add more packages as needed for specific scripts
];

// ============================================================================
// Script Service
// ============================================================================

/**
 * Script Service
 * 
 * Provides Python script execution via Pyodide for the browser IDE.
 * This enables users to write Python scripts that can:
 * - Process MML content
 * - Generate MML from algorithms
 * - Analyze compiled output
 * - Automate IDE tasks
 */
export class ScriptService {
    private static instance: ScriptService | null = null;
    
    // Pyodide instance
    private _pyodide: any | null = null;
    
    // Pyodide support check
    private _isSupported: boolean = true;
    
    // Initialization state
    private _isInitialized: boolean = false;
    private _initializationPromise: Promise<void> | null = null;
    
    // Scripts
    private _scripts: Map<string, Script> = new Map();
    private _currentScriptId: string | null = null;
    
    // State
    private _isEnabled: boolean = true;
    private _isExecuting: boolean = false;
    private _lastError: string | null = null;
    
    // Event listeners
    private _stateListeners: Array<(state: ScriptServiceState) => void> = [];
    
    // Pyodide options
    private _pyodideOptions: PyodideOptions = {
        indexURL: DEFAULT_PYODIDE_INDEX_URL,
        packages: DEFAULT_PACKAGES,
    };
    
    // ========================================================================
    // Singleton
    // ========================================================================
    
    public static getInstance(): ScriptService {
        if (!ScriptService.instance) {
            ScriptService.instance = new ScriptService();
        }
        return ScriptService.instance;
    }
    
    private constructor() {
        // Private constructor for singleton
        this._isSupported = this.checkPyodideSupport();
    }
    
    // ========================================================================
    // Pyodide Support Check
    // ========================================================================
    
    /**
     * Check if Pyodide is supported in the current environment.
     * Pyodide requires WebAssembly and may not be available in all browsers.
     */
    private checkPyodideSupport(): boolean {
        // Check for WebAssembly support
        if (!('WebAssembly' in window)) {
            return false;
        }
        
        // Check for fetch (required for loading Pyodide)
        if (!('fetch' in window)) {
            return false;
        }
        
        return true;
    }
    
    /**
     * Check if Pyodide is supported.
     */
    public get isSupported(): boolean {
        return this._isSupported;
    }
    
    // ========================================================================
    // Initialization
    // ========================================================================
    
    /**
     * Initialize Pyodide.
     * This loads the Pyodide runtime and required packages.
     */
    public async init(options?: PyodideOptions): Promise<void> {
        if (this._isInitialized) {
            return;
        }
        
        if (this._initializationPromise) {
            return this._initializationPromise;
        }
        
        if (!this._isSupported) {
            this._lastError = 'Pyodide is not supported in this environment';
            this.notifyListeners();
            return;
        }
        
        this._isExecuting = true;
        this._isEnabled = true;
        this.notifyListeners();
        
        // Merge options
        if (options) {
            this._pyodideOptions = {
                ...this._pyodideOptions,
                ...options,
            };
        }
        
        this._initializationPromise = (async () => {
            try {
                // Dynamically import pyodide
                // Note: In production, you'll need to include pyodide in your build
                // This can be done via CDN or by bundling pyodide with your app
                
                // Load pyodide from CDN
                const pyodideScript = document.createElement('script');
                pyodideScript.src = this._pyodideOptions.indexURL!
                    .replace(/\/pyodide\/.*$/, '/pyodide/pyodide.js');
                pyodideScript.type = 'text/javascript';
                pyodideScript.async = true;
                
                // Wait for the script to load
                await new Promise<void>((resolve, reject) => {
                    pyodideScript.onload = () => resolve();
                    pyodideScript.onerror = () => {
                        reject(new Error('Failed to load Pyodide script'));
                    };
                    document.head.appendChild(pyodideScript);
                });
                
                // @ts-expect-error - pyodide is a global variable
                this._pyodide = window.loadPyodide({
                    indexURL: this._pyodideOptions.indexURL,
                });
                
                // Wait for Pyodide to be ready
                await this._pyodide;
                
                // Load required packages
                for (const pkg of this._pyodideOptions.packages || []) {
                    try {
                        await this._pyodide.loadPackage(pkg);
                    } catch (e) {
                        console.warn(`Failed to load package ${pkg}:`, e);
                    }
                }
                
                this._isInitialized = true;
                this._isExecuting = false;
                this._lastError = null;
                this.notifyListeners();
                
                console.log('Pyodide initialized successfully');
            } catch (error) {
                this._lastError = `Failed to initialize Pyodide: ${error}`;
                this._isExecuting = false;
                this.notifyListeners();
                console.error('Pyodide initialization error:', error);
            }
        })();
        
        return this._initializationPromise;
    }
    
    /**
     * Check if Pyodide is initialized.
     */
    public get isInitialized(): boolean {
        return this._isInitialized;
    }
    
    /**
     * Get the Pyodide instance.
     */
    public get pyodide(): any | null {
        return this._pyodide;
    }
    
    // ========================================================================
    // Script Management
    // ========================================================================
    
    /**
     * Create a new script.
     */
    public createScript(name: string, content: string = '', path: string | null = null): Script {
        const id = this.generateId();
        const script: Script = {
            id,
            name,
            content,
            type: 'python',
            path,
            isDirty: false,
            lastExecuted: null,
            lastError: null,
        };
        
        this._scripts.set(id, script);
        this.notifyListeners();
        return script;
    }
    
    /**
     * Load a script from a file.
     */
    public async loadScript(file: File): Promise<Script> {
        const content = await file.text();
        const name = file.name;
        
        // Detect script type from extension
        const ext = name.split('.').pop()?.toLowerCase();
        const type: ScriptType = ext === 'py' ? 'python' : 'python';
        
        const script = this.createScript(name, content, null);
        return script;
    }
    
    /**
     * Save a script.
     */
    public saveScript(script: Script): void {
        const existing = this._scripts.get(script.id);
        if (existing) {
            this._scripts.set(script.id, {
                ...existing,
                ...script,
                isDirty: false,
            });
            this.notifyListeners();
        }
    }
    
    /**
     * Update script content.
     */
    public updateScriptContent(id: string, content: string): void {
        const script = this._scripts.get(id);
        if (script) {
            this._scripts.set(id, {
                ...script,
                content,
                isDirty: true,
            });
            this.notifyListeners();
        }
    }
    
    /**
     * Delete a script.
     */
    public deleteScript(id: string): void {
        this._scripts.delete(id);
        if (this._currentScriptId === id) {
            this._currentScriptId = null;
        }
        this.notifyListeners();
    }
    
    /**
     * Set the current script.
     */
    public setCurrentScript(id: string | null): void {
        this._currentScriptId = id;
        this.notifyListeners();
    }
    
    /**
     * Get the current script.
     */
    public getCurrentScript(): Script | null {
        if (!this._currentScriptId) return null;
        return this._scripts.get(this._currentScriptId) || null;
    }
    
    /**
     * Get a script by ID.
     */
    public getScript(id: string): Script | null {
        return this._scripts.get(id) || null;
    }
    
    /**
     * Get all scripts.
     */
    public getAllScripts(): Script[] {
        return Array.from(this._scripts.values());
    }
    
    // ========================================================================
    // Script Execution
    // ========================================================================
    
    /**
     * Execute a script.
     */
    public async executeScript(
        script: Script,
        context?: Partial<ScriptContext>
    ): Promise<ScriptResult> {
        if (!this._isEnabled || !this._isInitialized || !this._pyodide) {
            return {
                success: false,
                output: null,
                error: this._lastError || 'Script service is not ready',
                returnValue: null,
                executionTime: 0,
            };
        }
        
        const startTime = performance.now();
        this._isExecuting = true;
        script.lastError = null;
        this.notifyListeners();
        
        try {
            // Build the execution context
            const execContext: ScriptContext = {
                documentContent: '',
                documentLanguage: 'gwi',
                compileOptions: {},
                position: { line: 1, column: 1 },
                selection: null,
                ...context,
            };
            
            // Convert context to Python variables
            const pythonContext = this.buildPythonContext(execContext);
            
            // Execute the script
            const result = await this._pyodide.runPythonAsync(script.content, {
                globals: pythonContext,
            });
            
            script.lastExecuted = new Date();
            script.lastError = null;
            
            return {
                success: true,
                output: result?.toString() || null,
                error: null,
                returnValue: result,
                executionTime: performance.now() - startTime,
            };
        } catch (error) {
            const errorString = error instanceof Error ? error.message : String(error);
            script.lastError = errorString;
            return {
                success: false,
                output: null,
                error: errorString,
                returnValue: null,
                executionTime: performance.now() - startTime,
            };
        } finally {
            this._isExecuting = false;
            this.notifyListeners();
        }
    }
    
    /**
     * Execute a script function with arguments.
     */
    public async executeFunction(
        script: Script,
        functionName: string,
        args: unknown[] = []
    ): Promise<ScriptResult> {
        if (!this._isEnabled || !this._isInitialized || !this._pyodide) {
            return {
                success: false,
                output: null,
                error: this._lastError || 'Script service is not ready',
                returnValue: null,
                executionTime: 0,
            };
        }
        
        const startTime = performance.now();
        this._isExecuting = true;
        this.notifyListeners();
        
        try {
            // First, execute the script to define the function
            await this._pyodide.runPythonAsync(script.content);
            
            // Build the function call
            const argsString = args.map(arg => this.formatPythonValue(arg)).join(', ');
            const call = `${functionName}(${argsString})`;
            
            // Execute the function call
            const result = await this._pyodide.runPythonAsync(call);
            
            return {
                success: true,
                output: result?.toString() || null,
                error: null,
                returnValue: result,
                executionTime: performance.now() - startTime,
            };
        } catch (error) {
            const errorString = error instanceof Error ? error.message : String(error);
            return {
                success: false,
                output: null,
                error: errorString,
                returnValue: null,
                executionTime: performance.now() - startTime,
            };
        } finally {
            this._isExecuting = false;
            this.notifyListeners();
        }
    }
    
    /**
     * Build Python context from execution context.
     */
    private buildPythonContext(context: ScriptContext): Record<string, unknown> {
        return {
            // Document information
            document_content: context.documentContent,
            document_language: context.documentLanguage,
            
            // Compile options (convert to Python-compatible types)
            compile_options: this.convertToPython(context.compileOptions),
            
            // Position
            position_line: context.position.line,
            position_column: context.position.column,
            
            // Selection
            selection: context.selection,
            
            // Utility functions for scripts
            log: (message: string) => {
                console.log(`[Script] ${message}`);
                return message;
            },
            error: (message: string) => {
                console.error(`[Script Error] ${message}`);
                throw new Error(message);
            },
        };
    }
    
    /**
     * Convert JavaScript value to Python-compatible representation.
     */
    private convertToPython(value: unknown): unknown {
        if (value === null || value === undefined) {
            return null;
        }
        if (typeof value === 'boolean') {
            return value;
        }
        if (typeof value === 'number') {
            return value;
        }
        if (typeof value === 'string') {
            return value;
        }
        if (Array.isArray(value)) {
            return value.map(item => this.convertToPython(item));
        }
        if (typeof value === 'object') {
            const result: Record<string, unknown> = {};
            for (const [key, val] of Object.entries(value)) {
                result[key] = this.convertToPython(val);
            }
            return result;
        }
        return value;
    }
    
    /**
     * Format a value for use in Python code.
     */
    private formatPythonValue(value: unknown): string {
        if (value === null || value === undefined) {
            return 'None';
        }
        if (typeof value === 'boolean') {
            return value ? 'True' : 'False';
        }
        if (typeof value === 'number') {
            return value.toString();
        }
        if (typeof value === 'string') {
            // Escape quotes in string
            return `"${value.toString().replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
        }
        if (Array.isArray(value)) {
            return `[${value.map(v => this.formatPythonValue(v)).join(', ')}]`;
        }
        if (typeof value === 'object') {
            const entries = Object.entries(value as Record<string, unknown>);
            return `{${entries.map(([k, v]) => `${k}: ${this.formatPythonValue(v)}`).join(', ')}}`;
        }
        return JSON.stringify(value);
    }
    
    // ========================================================================
    // Script Analysis
    // ========================================================================
    
    /**
     * Extract function information from a Python script.
     */
    public async getScriptFunctions(script: Script): Promise<ScriptFunction[]> {
        if (!this._isInitialized || !this._pyodide) {
            return [];
        }
        
        try {
            // Parse the script to find function definitions
            // This is a simple regex-based approach - for more robust parsing,
            // we could use Pyodide's ast module
            const functions: ScriptFunction[] = [];
            
            // Simple regex to find function definitions
            const funcRegex = /def\s+(\w+)\s*\((.*?)\)\s*:/g;
            let match;
            
            while ((match = funcRegex.exec(script.content)) !== null) {
                const funcName = match[1];
                const paramsStr = match[2];
                
                // Parse parameters
                const params: ScriptParameter[] = [];
                if (paramsStr.trim()) {
                    const paramParts = paramsStr.split(',').map(p => p.trim());
                    for (const paramPart of paramParts) {
                        // Skip self parameter
                        if (paramPart === 'self') continue;
                        
                        // Simple parameter (name: type = default)
                        const paramMatch = paramPart.match(/^(\w+)(?::\s*(\w+))?(?:\\s*=.*?)?$/);
                        if (paramMatch) {
                            params.push({
                                name: paramMatch[1],
                                type: paramMatch[2] || 'any',
                                description: null,
                            });
                        }
                    }
                }
                
                functions.push({
                    name: funcName,
                    parameters: params,
                    description: null, // Would need docstring parsing
                });
            }
            
            return functions;
        } catch (error) {
            console.error('Failed to analyze script:', error);
            return [];
        }
    }
    
    // ========================================================================
    // Utility Methods
    // ========================================================================
    
    /**
     * Generate a unique ID.
     */
    private generateId(): string {
        return `script-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
    }
    
    // ========================================================================
    // State Management
    // ========================================================================
    
    /**
     * Get current state.
     */
    public getState(): ScriptServiceState {
        return {
            isEnabled: this._isEnabled,
            isSupported: this._isSupported,
            isInitialized: this._isInitialized,
            scripts: Array.from(this._scripts.values()),
            currentScriptId: this._currentScriptId,
            isExecuting: this._isExecuting,
            lastError: this._lastError,
        };
    }
    
    /**
     * Subscribe to state changes.
     */
    public subscribe(callback: (state: ScriptServiceState) => void): () => void {
        this._stateListeners.push(callback);
        // Send current state immediately
        callback(this.getState());
        
        return () => {
            const index = this._stateListeners.indexOf(callback);
            if (index >= 0) {
                this._stateListeners.splice(index, 1);
            }
        };
    }
    
    private notifyListeners(): void {
        this._stateListeners.forEach(callback => callback(this.getState()));
    }
    
    // ========================================================================
    // Enable/Disable
    // ========================================================================
    
    /**
     * Enable script execution.
     */
    public enable(): void {
        this._isEnabled = true;
        this.notifyListeners();
    }
    
    /**
     * Disable script execution.
     */
    public disable(): void {
        this._isEnabled = false;
        this.notifyListeners();
    }
    
    /**
     * Check if script execution is enabled.
     */
    public get isEnabled(): boolean {
        return this._isEnabled;
    }
    
    /**
     * Check if currently executing a script.
     */
    public get isExecuting(): boolean {
        return this._isExecuting;
    }
    
    /**
     * Get the last error.
     */
    public get lastError(): string | null {
        return this._lastError;
    }
}

// ============================================================================
// Default Script Templates
// ============================================================================

/**
 * Built-in Python script templates for common MML operations.
 */
export const SCRIPT_TEMPLATES = {
    /**
     * Template for a simple "Hello World" script.
     */
    helloWorld: {
        name: 'Hello World',
        content: `# Hello World Script
# This is a simple test script for the browser IDE

def main():
    log("Hello from Python!")
    return "Script executed successfully"

# Run main if this is the main module
if __name__ == "__main__":
    main()
`,
    },
    
    /**
     * Template for MML content analysis.
     */
    analyzeMML: {
        name: 'Analyze MML',
        content: `# MML Analysis Script
# Analyzes the current MML document

def count_notes(content: str) -> int:
    """Count the number of notes in the MML content."""
    import re
    # Match note patterns like c4, d+4, e-4, etc.
    note_pattern = r'[a-gA-G][+#-]?\\d*'
    notes = re.findall(note_pattern, content)
    return len(notes)

def count_parts(content: str) -> int:
    """Count the number of parts in the MML content."""
    import re
    # Match part commands like @0, @1, etc.
    part_pattern = r'@\\d+'
    parts = re.findall(part_pattern, content)
    return len(parts)

def analyze_document():
    """Analyze the current document."""
    notes = count_notes(document_content)
    parts = count_parts(document_content)
    
    log(f"Document Analysis:")
    log(f"  - Notes: {notes}")
    log(f"  - Parts: {parts}")
    log(f"  - Language: {document_language}")
    
    return {
        'notes': notes,
        'parts': parts,
        'language': document_language
    }

# Run analysis if this is the main module
if __name__ == "__main__":
    result = analyze_document()
    log(str(result))
`,
    },
    
    /**
     * Template for MML generation.
     */
    generateMML: {
        name: 'Generate MML',
        content: `# MML Generation Script
# Generates MML content programmatically

def generate_scale(root_note: str = 'c', octave: int = 4, length: str = '4') -> str:
    """Generate a simple scale."""
    notes = ['c', 'd', 'e', 'f', 'g', 'a', 'b']
    scale = []
    
    for note in notes:
        scale.append(f"{note}{octave}{length} ")
    
    return ''.join(scale)

def generate_chord(root: str, octave: int = 4, length: str = '4') -> str:
    """Generate a simple triad chord."""
    # Simple C major triad
    chord_notes = {
        'c': ['c', 'e', 'g'],
        'd': ['d', 'f+', 'a'],
        'e': ['e', 'g+', 'b'],
        'f': ['f', 'a', 'c'],
        'g': ['g', 'b', 'd'],
        'a': ['a', 'c+', 'e'],
        'b': ['b', 'd+', 'f+'],
    }
    
    notes = chord_notes.get(root.lower(), ['c', 'e', 'g'])
    return ''.join([f"{n}{octave}{length}" for n in notes])

# Example usage
if __name__ == "__main__":
    scale = generate_scale('c', 4, '8')
    log(f"Generated scale: {scale}")
    
    chord = generate_chord('c', 4, '4')
    log(f"Generated chord: {chord}")
`,
    },
    
    /**
     * Template for MML transformation.
     */
    transformMML: {
        name: 'Transform MML',
        content: `# MML Transformation Script
# Transforms MML content (e.g., transpose, change octave)

def transpose_note(note: str, semitones: int) -> str:
    """Transpose a single note by semitones."""
    notes = ['c', 'c+', 'd', 'd+', 'e', 'f', 'f+', 'g', 'g+', 'a', 'a+', 'b']
    
    # Parse note
    note_match = re.match(r'([a-gA-G][+#-]?)(\\d*)', note)
    if not note_match:
        return note  # Not a note, return as-is
    
    note_name = note_match.group(1).lower()
    octave = note_match.group(2)
    
    # Find note index
    try:
        idx = notes.index(note_name)
    except ValueError:
        return note  # Note not found, return as-is
    
    # Calculate new index
    new_idx = (idx + semitones) % 12
    new_note = notes[new_idx]
    
    # Handle octave change
    octave_change = semitones // 12
    if octave:
        new_octave = str(int(octave) + octave_change)
    else:
        new_octave = str(4 + octave_change)  # Default to octave 4
    
    return f"{new_note}{new_octave}"

def transpose_content(content: str, semitones: int) -> str:
    """Transpose all notes in the content by semitones."""
    import re
    
    # Find all notes
    def replacer(match):
        note = match.group(0)
        return transpose_note(note, semitones)
    
    # Match note patterns
    note_pattern = r'[a-gA-G][+#-]?\\d*'
    return re.sub(note_pattern, replacer, content)

# Example usage
if __name__ == "__main__":
    transposed = transpose_content(document_content, 2)  # Transpose up 2 semitones
    log("Transposed content:")
    log(transposed[:200] + "..." if len(transposed) > 200 else transposed)
`,
    },
};

// ============================================================================
// Exports
// ============================================================================

/**
 * Singleton instance of the ScriptService.
 * Use this for easy access throughout the application.
 */
export const scriptService = ScriptService.getInstance();

export default ScriptService;
