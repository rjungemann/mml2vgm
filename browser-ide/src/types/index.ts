// Main type definitions for the mml2vgm Browser IDE

// ============================================================================
// MML Types
// ============================================================================

/** Declared implementation maturity for a chip or format. */
export type SupportTier = 'full' | 'partial' | 'declared';

/** Supported output formats */
export type OutputFormat = 'vgm' | 'xgm' | 'xgm2' | 'zgm';

/** Supported sound chip types */
export type SoundChip = 
    | 'YM2612' | 'YM2612X' | 'YM2612X2'
    | 'SN76489' | 'SN76489X2'
    | 'YM2608' | 'YM2609' | 'YM2610B'
    | 'YM2151' | 'YM3526' | 'Y8950' | 'YM3812' | 'YMF262'
    | 'YM2413' | 'YM2203'
    | 'RF5C164' | 'SegaPCM' | 'HuC6280' | 'C140' | 'C352'
    | 'AY8910' | 'K051649' | 'K053260' | 'K054539' | 'QSound'
    | 'NES' | 'DMG' | 'VRC6' | 'POKEY' | 'MIDI' | 'CONDUCTOR';

/** Chip information */
export interface ChipInfo {
    name: string;
    variant: SoundChip;
    clockRate: number;
    isPsg: boolean;
    isFm: boolean;
    supportsPcm: boolean;
    supportTier: SupportTier;
    browserCompileDefault: boolean;
}

/** Format information */
export interface FormatInfo {
    name: OutputFormat;
    extension: string;
    supportTier: SupportTier;
}

// ============================================================================
// Token Types (for syntax highlighting)
// ============================================================================

/** Token types from the lexer */
export type TokenType = 
    | 'number' | 'string' | 'identifier'
    | 'note' | 'sharp' | 'flat' | 'rest' | 'duration'
    | 'dot' | 'tie' | 'octave_up' | 'octave_down'
    | 'octave_cmd' | 'volume_cmd' | 'tempo_cmd' | 'length_cmd'
    | 'part_cmd' | 'bar' | 'comment' | 'whitespace' | 'eof'
    | 'left_brace' | 'right_brace' | 'apostrophe' | 'equals'
    | 'comma' | 'left_bracket' | 'right_bracket'
    | 'left_paren' | 'right_paren';

/** A token from the lexer */
export interface Token {
    type: TokenType;
    value: string;
    line: number;
    column: number;
}

// ============================================================================
// Compile Types
// ============================================================================

/** Compilation status */
export type CompileStatus = 'idle' | 'queued' | 'compiling' | 'success' | 'error';

/** Compilation options */
export interface CompileOptions {
    format: OutputFormat;
    target_chips?: SoundChip[];
    verbose?: boolean;
    debug?: boolean;
    output_trace?: boolean;
    compression?: number; // 0-9
    encoding?: string;
    include_paths?: string[];
    clock_count?: number;
}

/** Compilation result */
export interface CompileResult {
    data?: Uint8Array;
    errors: ErrorContext[];
    warnings: ErrorContext[];
    info?: CompileInfo;
}

/** Compilation information */
export interface CompileInfo {
    part_count: number;
    command_count: number;
    duration_samples: number;
    duration_seconds: number;
    chips_used: SoundChip[];
    format_version: string;
}

/** Error context */
export interface ErrorContext {
    message: string;
    line: number;
    column: number;
    length: number;
}

// ============================================================================
// Document Types
// ============================================================================

/** Supported MML file types */
export type MMLLanguage = 'gwi' | 'mml' | 'muc' | 'mdl' | 'mus' | 'm98' | 'muap';

/** A document in the editor */
export interface Document {
    id: string;
    filename: string;
    content: string;
    language: MMLLanguage;
    encoding: string;
    isDirty: boolean;
    lastCompileTime: Date | null;
    lastCompileSuccess: boolean;
    lastCompileErrors: ErrorContext[];
    // File System Access API handle for save operations
    fileHandle?: FileSystemFileHandle;
}

/** Document state for Monaco Editor */
export interface EditorState {
    line: number;
    column: number;
    selectionStartLine: number;
    selectionStartColumn: number;
    selectionEndLine: number;
    selectionEndColumn: number;
}

// ============================================================================
// Player Types
// ============================================================================

/** Player state */
export type PlayerState = 'stopped' | 'playing' | 'paused';

/** Chip player handle (opaque) */
export interface ChipPlayerHandle {
    id: string;
}

/** VGM player handle (opaque) */
export interface VgmPlayerHandle {
    id: string;
}

/** VGM header information */
export interface VgmHeaderInfo {
    total_samples: number;
    loop_samples: number;
    sample_rate: number;
    version: number;
    sn76489_clock: number;
    ym2413_clock: number;
    ym2612_clock: number;
    ym2151_clock: number;
}

// ============================================================================
// Part/Channel Types
// ============================================================================

/** Part information from MML */
export interface PartInfo {
    index: number;
    name: string;
    chip: SoundChip;
    channel: number;
    volume: number;
    pan: number;
    isSolo: boolean;
    isMuted: boolean;
    isKbdAssigned: boolean;
}

/** Part with additional state and position information */
export interface PartWithState extends PartInfo {
    startPosition: Position;
    endPosition: Position;
}

/** Channel information */
export interface ChannelInfo {
    chip: SoundChip;
    channel: number;
    volume: number;
    pan: number;
    muted: boolean;
    solo: boolean;
}

// ============================================================================
// Audio Types
// ============================================================================

/** Audio settings */
export interface SettingsAudio {
    sampleRate: number;
    bufferSize: number;
    masterVolume: number;
    enableReverb: boolean;
    reverbLevel: number;
    loopCount: number;
    fadeOutDuration: number;
    playbackRate: number;
    loop: boolean;
}

/** Audio settings (alias) */
export type AudioSettings = SettingsAudio;

/** Mixer chip settings */
export interface MixerChipSettings {
    chip: SoundChip;
    volume: number;
    pan: number;
    mute: boolean;
    solo: boolean;
}

// ============================================================================
// UI Types
// ============================================================================

/** Panel types */
export type PanelType =
    | 'folder'
    | 'partCounter'
    | 'errorList'
    | 'log'
    | 'lyrics'
    | 'mixer'
    | 'midiKeyboard'
    | 'debug'
    | 'playback'
    | 'compileOptions'
    | 'info'
    | 'folderTree'
    | 'script'
    | 'runtime'
    | 'compilation'
    | 'waveform'
    | 'fmToneEditor'
    | 'envelopeEditor'
    | 'arpeggioEditor';

/** Panel position */
export type PanelPosition = 
    | 'left'
    | 'right'
    | 'bottom'
    | 'floating';

/** Panel configuration */
export interface PanelConfig {
    id: PanelType;
    position: PanelPosition;
    visible: boolean;
    width?: number;
    height?: number;
}

/** Panel state */
export interface PanelState {
    panel: PanelType;
    isActive: boolean;
    isFocused: boolean;
}

// ============================================================================
// Settings Types
// ============================================================================

/** Editor settings */
export interface EditorSettings {
    fontSize: number;
    fontFamily: string;
    wordWrap: boolean;
    showLineNumbers: boolean;
    tabSize: number;
    insertSpaces: boolean;
    theme: 'vs-dark' | 'vs' | 'hc-black';
    showMinimap: boolean;
    options: Record<string, any>;
}

/** Panel settings */
export interface PanelSettings {
    visiblePanels: PanelType[];
    rightSidebar: PanelType[];
    bottomPanel: PanelType;
    panelWidth: number;
    bottomPanelHeight: number;
}

/** Audio settings */
export interface SettingsAudio {
    sampleRate: number;
    bufferSize: number;
    masterVolume: number;
    enableReverb: boolean;
    reverbLevel: number;
    loopCount: number;
}

/** MIDI settings */
export interface MidiSettings {
    enabled: boolean;
    inputDevice: string | null;
    outputDevice: string | null;
    channel: number;
    velocityCurve: 'linear' | 'logarithmic' | 'exponential';
}

/** WebHID report decoding format */
export type HIDReportFormat = 'usb-midi-class' | 'raw-scan';

/** WebHID MIDI controller settings (experimental) */
export interface HIDSettings {
  /** Enable HID MIDI input */
  enabled: boolean;
  /** How to interpret raw HID input reports */
  reportFormat: HIDReportFormat;
  /** Report ID to listen to (null = accept all) */
  reportId: number | null;
  /** Byte offset into the report where MIDI data starts */
  byteOffset: number;
  /** Attempt to reconnect previously granted devices on startup */
  autoReconnect: boolean;
}

/** WebSerial protocol variant */
export type SerialProtocol = 'gimic' | 'scci-raw' | 'generic';

/** WebSerial hardware-access settings (experimental) */
export interface SerialSettings {
    /** Baud rate for the serial connection */
    baudRate: number;
    /** Protocol adapter to use when encoding register writes */
    protocol: SerialProtocol;
    /** Whether to auto-reconnect to the last granted port on startup */
    autoReconnect: boolean;
}

/** Key binding */
export interface KeyBinding {
    command: string;
    key: string;
    ctrlKey?: boolean;
    shiftKey?: boolean;
    altKey?: boolean;
    metaKey?: boolean;
}

/** Full IDE settings */
export interface IDESettings {
    // General
    theme: 'dark' | 'light' | 'system';
    language: string;
    
    // Editor
    editor: EditorSettings;
    
    // Compilation
    outputFormat: OutputFormat;
    defaultChip: SoundChip;
    clockRate: number;
    
    // Audio
    audio: SettingsAudio;
    
    // MIDI
    midiMode: string;
    midiChannel: number;

    // HID MIDI controllers (experimental)
    hid: HIDSettings;

    // Serial / Hardware (experimental)
    serial: SerialSettings;

    // UI
    panelVisibility: Record<PanelType, boolean>;
    panelPositions: Record<PanelType, PanelPosition>;
    
    // Other
    keyBindings: KeyBinding[];
    defaultPath: string;
    rememberLastPath: boolean;
    autoSave: boolean;
    autoSaveInterval: number;
}

// ============================================================================
// Error Types
// ============================================================================

/** Compile error type */
export type CompileErrorType = 
    | 'syntax'
    | 'semantic'
    | 'compile'
    | 'runtime'
    | 'warning'
    | 'error';

/** Compile error */
export interface CompileError {
    type: CompileErrorType;
    message: string;
    line: number;
    column: number;
    length: number;
    severity: 'error' | 'warning' | 'info';
    code?: string;
}

// ============================================================================
// Event Types
// ============================================================================

/** Document change event */
export interface DocumentChangeEvent {
    documentId: string;
    oldContent: string;
    newContent: string;
    timestamp: Date;
}

/** Compilation event */
export interface CompilationEvent {
    documentId: string;
    status: CompileStatus;
    timestamp: Date;
    duration?: number;
}

/** Playback event */
export interface PlaybackEvent {
    playerId: string;
    state: PlayerState;
    position: number; // in samples
    timestamp: Date;
}

// ============================================================================
// MIDI Types
// ============================================================================

/** MIDI note */
export interface MidiNote {
    note: number; // 0-127
    velocity: number; // 0-127
    channel: number; // 0-15
    duration: number; // in seconds
}

/** MIDI device info */
export interface MidiDeviceInfo {
    id: string;
    name: string;
    type: 'input' | 'output';
    manufacturer?: string;
    version?: string;
}

// ============================================================================
// File Types
// ============================================================================

/** File system entry */
export interface FileSystemEntry {
    id: string;
    name: string;
    path: string;
    type: 'file' | 'directory';
    size?: number;
    lastModified?: Date;
    children?: FileSystemEntry[];
}

/** File handle for browser File System Access API */
export interface FileHandle {
    name: string;
    kind: 'file';
    getFile(): Promise<File>;
}

// ============================================================================
// Lyrics Types
// ============================================================================

/** Lyric entry */
export interface LyricEntry {
    time: number; // in seconds
    text: string;
    duration?: number;
}

// ============================================================================
// Script Types
// ============================================================================

/** Script language */
export type ScriptLanguage = 'javascript' | 'lua';

/** Script information */
export interface ScriptInfo {
    name: string;
    language: ScriptLanguage;
    description: string;
    code: string;
}

// ============================================================================
// Trace Types
// ============================================================================

/** Position in a document (line and column) */
export interface Position {
    line: number;
    column: number;
}

/** Trace event types */
export type TraceEventType = 'register-write' | 'part-event' | 'note-on' | 'note-off' | 'command';

/** Trace event for debugging playback */
export interface TraceEvent {
    type: TraceEventType;
    chip?: string;
    addr?: number;
    data?: number;
    partIndex?: number;
    event?: string;
    note?: number;
    velocity?: number;
    command?: string;
    value?: number;
    time: number; // in milliseconds
    timestamp: Date;
}

