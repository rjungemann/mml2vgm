/**
 * Settings Store
 * 
 * Manages IDE settings including editor preferences, audio settings, and UI state.
 */

import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import type {
    IDESettings,
    EditorSettings,
    AudioSettings,
    HIDSettings,
    SerialSettings,
    PanelType,
    PanelPosition,
    OutputFormat,
    SoundChip,
} from '@/types';

// ============================================================================
// Default Settings
// ============================================================================

const defaultEditorSettings: EditorSettings = {
    fontSize: 14,
    fontFamily: 'Consolas, "Courier New", monospace',
    wordWrap: false,
    showLineNumbers: true,
    showMinimap: true,
    tabSize: 4,
    insertSpaces: true,
    theme: 'vs-dark',
    options: {},
};

const defaultAudioSettings: AudioSettings = {
    sampleRate: 44100,
    bufferSize: 4096,
    masterVolume: 100,
    enableReverb: false,
    reverbLevel: 0.5,
    loopCount: 0,
    fadeOutDuration: 1000,
    playbackRate: 1.0,
    loop: false,
};

const defaultHIDSettings: HIDSettings = {
    enabled: false,
    reportFormat: 'usb-midi-class',
    reportId: null,
    byteOffset: 0,
    autoReconnect: false,
};

const defaultSerialSettings: SerialSettings = {
    baudRate: 38400,
    protocol: 'gimic',
    autoReconnect: false,
};

const defaultPanelVisibility: Record<PanelType, boolean> = {
    folder: true,
    folderTree: true,
    partCounter: true,
    errorList: true,
    log: true,
    lyrics: false,
    mixer: false,
    midiKeyboard: false,
    debug: false,
    playback: true,
    compileOptions: true,
    info: true,
    script: false,
    runtime: true,
    compilation: true,
    waveform: true,
    fmToneEditor: false,
    envelopeEditor: false,
    arpeggioEditor: false,
};

const defaultPanelPositions: Record<PanelType, PanelPosition> = {
    folder: 'left',
    folderTree: 'left',
    partCounter: 'bottom',
    errorList: 'bottom',
    log: 'bottom',
    lyrics: 'right',
    mixer: 'right',
    midiKeyboard: 'floating',
    script: 'right',
    debug: 'floating',
    playback: 'bottom',
    compileOptions: 'right',
    info: 'bottom',
    runtime: 'bottom',
    compilation: 'bottom',
    waveform: 'bottom',
    fmToneEditor: 'right',
    envelopeEditor: 'right',
    arpeggioEditor: 'right',
};

export const defaultSettings: IDESettings = {
    // General
    theme: 'dark',
    language: 'en',
    
    // Editor
    editor: defaultEditorSettings,
    
    // Compilation
    outputFormat: 'vgm',
    defaultChip: 'YM2612',
    // Clock count override (0 = auto/default from driver or MML)
    clockRate: 0,
    
    // Audio
    audio: defaultAudioSettings,
    
    // MIDI
    midiMode: 'preview',
    midiChannel: 0,

    // HID MIDI controllers (experimental)
    hid: defaultHIDSettings,

    // Serial / Hardware (experimental)
    serial: defaultSerialSettings,

    // UI
    panelVisibility: defaultPanelVisibility,
    panelPositions: defaultPanelPositions,
    
    // Other
    keyBindings: [],
    defaultPath: '/',
    rememberLastPath: true,
    autoSave: true,
    autoSaveInterval: 30,
};

// ============================================================================
// Types
// ============================================================================

interface SettingsState {
    settings: IDESettings;
}

interface SettingsActions {
    // Get current settings
    getSettings: () => IDESettings;
    
    // Update entire settings
    setSettings: (settings: IDESettings) => void;
    
    // Update partial settings
    updateSettings: (updates: Partial<IDESettings>) => void;
    
    // Reset to defaults
    resetSettings: () => void;
    
    // Editor settings
    updateEditorSettings: (updates: Partial<EditorSettings>) => void;
    setFontSize: (size: number) => void;
    setFontFamily: (family: string) => void;
    setWordWrap: (enabled: boolean) => void;
    setShowLineNumbers: (enabled: boolean) => void;
    setTabSize: (size: number) => void;
    setInsertSpaces: (enabled: boolean) => void;
    setEditorTheme: (theme: EditorSettings['theme']) => void;
    
    // Audio settings
    updateAudioSettings: (updates: Partial<AudioSettings>) => void;
    setSampleRate: (rate: number) => void;
    setBufferSize: (size: number) => void;
    setMasterVolume: (volume: number) => void;
    
    // Compilation settings
    setOutputFormat: (format: OutputFormat) => void;
    setDefaultChip: (chip: SoundChip) => void;
    setClockRate: (rate: number) => void;
    
    // MIDI settings
    setMidiMode: (mode: 'preview' | 'input') => void;
    setMidiChannel: (channel: number) => void;
    
    // HID settings
    updateHIDSettings: (updates: Partial<HIDSettings>) => void;

    // Serial settings
    updateSerialSettings: (updates: Partial<SerialSettings>) => void;

    // Panel visibility
    setPanelVisibility: (panel: PanelType, visible: boolean) => void;
    togglePanelVisibility: (panel: PanelType) => void;
    
    // Panel positions
    setPanelPosition: (panel: PanelType, position: PanelPosition) => void;
    
    // General settings
    setTheme: (theme: IDESettings['theme']) => void;
    setLanguage: (language: string) => void;
}

type SettingsStore = SettingsState & SettingsActions;

// ============================================================================
// Store Definition
// ============================================================================

export const useSettingsStore = create<SettingsStore>()(
    persist(
        (set, get) => ({
            settings: defaultSettings,
            
            // ============================================================
            // Getters
            // ============================================================
            
            getSettings: () => get().settings,
            
            // ============================================================
            // Actions
            // ============================================================
            
            setSettings: (settings: IDESettings) => {
                set({ settings });
            },
            
            updateSettings: (updates: Partial<IDESettings>) => {
                const current = get().settings;
                set({ settings: { ...current, ...updates } });
            },
            
            resetSettings: () => {
                set({ settings: { ...defaultSettings } });
            },
            
            // Editor settings
            updateEditorSettings: (updates: Partial<EditorSettings>) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        editor: { ...current.editor, ...updates },
                    },
                });
            },
            
            setFontSize: (size: number) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        editor: { ...current.editor, fontSize: size },
                    },
                });
            },
            
            setFontFamily: (family: string) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        editor: { ...current.editor, fontFamily: family },
                    },
                });
            },
            
            setWordWrap: (enabled: boolean) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        editor: { ...current.editor, wordWrap: enabled },
                    },
                });
            },
            
            setShowLineNumbers: (enabled: boolean) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        editor: { ...current.editor, showLineNumbers: enabled },
                    },
                });
            },
            
            setTabSize: (size: number) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        editor: { ...current.editor, tabSize: size },
                    },
                });
            },
            
            setInsertSpaces: (enabled: boolean) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        editor: { ...current.editor, insertSpaces: enabled },
                    },
                });
            },
            
            setEditorTheme: (theme: EditorSettings['theme']) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        editor: { ...current.editor, theme },
                    },
                });
            },
            
            // Audio settings
            updateAudioSettings: (updates: Partial<AudioSettings>) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        audio: { ...current.audio, ...updates },
                    },
                });
            },
            
            setSampleRate: (rate: number) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        audio: { ...current.audio, sampleRate: rate },
                    },
                });
            },
            
            setBufferSize: (size: number) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        audio: { ...current.audio, bufferSize: size },
                    },
                });
            },
            
            setMasterVolume: (volume: number) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        audio: { ...current.audio, masterVolume: volume },
                    },
                });
            },
            
            // Compilation settings
            setOutputFormat: (format: OutputFormat) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        outputFormat: format,
                    },
                });
            },
            
            setDefaultChip: (chip: SoundChip) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        defaultChip: chip,
                    },
                });
            },
            
            setClockRate: (rate: number) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        clockRate: rate,
                    },
                });
            },
            
            // MIDI settings
            setMidiMode: (mode: 'preview' | 'input') => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        midiMode: mode,
                    },
                });
            },
            
            setMidiChannel: (channel: number) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        midiChannel: channel,
                    },
                });
            },
            
            // HID settings
            updateHIDSettings: (updates: Partial<HIDSettings>) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        hid: { ...current.hid, ...updates },
                    },
                });
            },

            // Serial settings
            updateSerialSettings: (updates: Partial<SerialSettings>) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        serial: { ...current.serial, ...updates },
                    },
                });
            },

            // Panel visibility
            setPanelVisibility: (panel: PanelType, visible: boolean) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        panelVisibility: {
                            ...current.panelVisibility,
                            [panel]: visible,
                        },
                    },
                });
            },
            
            togglePanelVisibility: (panel: PanelType) => {
                const current = get().settings;
                const currentVisibility = current.panelVisibility[panel];
                set({
                    settings: {
                        ...current,
                        panelVisibility: {
                            ...current.panelVisibility,
                            [panel]: !currentVisibility,
                        },
                    },
                });
            },
            
            // Panel positions
            setPanelPosition: (panel: PanelType, position: PanelPosition) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        panelPositions: {
                            ...current.panelPositions,
                            [panel]: position,
                        },
                    },
                });
            },
            
            // General settings
            setTheme: (theme: IDESettings['theme']) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        theme,
                    },
                });
            },
            
            setLanguage: (language: string) => {
                const current = get().settings;
                set({
                    settings: {
                        ...current,
                        language,
                    },
                });
            },
        }),
        {
            name: 'mml2vgm-settings-store',
            storage: createJSONStorage(() => sessionStorage),
        }
    )
);

// ============================================================================
// Selectors
// ============================================================================

// Selector for current settings
export const selectSettings = (state: SettingsStore) => state.getSettings();

// Selector for editor settings
export const selectEditorSettings = (state: SettingsStore) => 
    state.getSettings().editor;

// Selector for audio settings
export const selectAudioSettings = (state: SettingsStore) => 
    state.getSettings().audio;

// Selector for output format
export const selectOutputFormat = (state: SettingsStore) => 
    state.getSettings().outputFormat;

// Selector for default chip
export const selectDefaultChip = (state: SettingsStore) => 
    state.getSettings().defaultChip;

// Selector for theme
export const selectTheme = (state: SettingsStore) => 
    state.getSettings().theme;

// Selector for panel visibility
export const selectPanelVisibility = (state: SettingsStore, panel: PanelType) => 
    state.getSettings().panelVisibility[panel];

// Selector for panel position
export const selectPanelPosition = (state: SettingsStore, panel: PanelType) => 
    state.getSettings().panelPositions[panel];

// Selector for all panel visibility
export const selectAllPanelVisibility = (state: SettingsStore) => 
    state.getSettings().panelVisibility;
