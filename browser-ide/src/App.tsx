import React, { useEffect, useState, useCallback, useRef } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { wasmService } from '@/services/wasmService';
import { audioService, type AudioRuntimeDebugInfo } from '@/services/audioService';
import { traceService } from '@/services/traceService';
import { partService } from '@/services/partService';
import { formatService } from '@/services/formatService';
import { storageService, registerServiceWorker, setUpdateNotificationCallback } from '@/services/storageService';
import { sampleService } from '@/services/sampleService';
import { serialService } from '@/services/serialService';
import { hidService } from '@/services/hidService';
import { i18nService } from '@/services/i18nService';
import { useDocumentStore } from '@/stores/documentStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useCompileStore } from '@/stores/compileStore';
import type { ChipInfo, PanelType, Position, SourceMapEvent } from '@/types';
import MonacoEditor, { type MonacoEditorHandle } from '@/components/Editor/MonacoEditor';
import StatusBar from '@/components/StatusBar';
import MenuBar from '@/components/MenuBar';
import ErrorListPanel from '@/components/panels/ErrorListPanel';
import PartCounterPanel from '@/components/panels/PartCounterPanel';
import FolderTreePanel from '@/components/panels/FolderTreePanel';
import PlaybackPanel from '@/components/panels/PlaybackPanel';
import CompileOptionsPanel from '@/components/panels/CompileOptionsPanel';
import InfoPanel from '@/components/panels/InfoPanel';
import ScriptPanel from '@/components/panels/ScriptPanel';
import LyricsPanel from '@/components/panels/LyricsPanel';
import MixerPanel from '@/components/panels/MixerPanel';
import MIDIKeyboardPanel from '@/components/panels/MIDIKeyboardPanel';
import DebugPanel from '@/components/panels/DebugPanel';
import RuntimePanel from '@/components/panels/RuntimePanel';
import CompilationPanel from '@/components/panels/CompilationPanel';
import WaveformPanel from '@/components/panels/WaveformPanel';
import SamplesPanel from '@/components/panels/SamplesPanel';
import FmToneEditorPanel from '@/components/panels/FmToneEditorPanel';
import EnvelopeEditorPanel from '@/components/panels/EnvelopeEditorPanel';
import ArpeggioEditorPanel from '@/components/panels/ArpeggioEditorPanel';
import { TabBar } from '@/components/TabBar';
import BottomTabs, { type BottomTab } from '@/components/BottomTabs';
import { useSessionStorageState } from '@/utils/useSessionStorageState';
import AboutDialog from '@/components/dialogs/AboutDialog';
import KeyBindingsDialog from '@/components/dialogs/KeyBindingsDialog';
import AudioSettingsDialog from '@/components/dialogs/AudioSettingsDialog';
import AdvancedCompileOptionsDialog, { type AdvancedCompileOptions } from '@/components/dialogs/AdvancedCompileOptionsDialog';
import MidiSettingsDialog from '@/components/dialogs/MidiSettingsDialog';
import HIDSettingsDialog from '@/components/dialogs/HIDSettingsDialog';
import SerialSettingsDialog from '@/components/dialogs/SerialSettingsDialog';
import PreferencesDialog from '@/components/dialogs/PreferencesDialog';
import HelpDialog from '@/components/dialogs/HelpDialog';
import MmlReferenceDialog from '@/components/dialogs/MmlReferenceDialog';

export const App: React.FC = () => {
  const MIN_SIDEBAR_WIDTH = 180;
  const MAX_SIDEBAR_WIDTH = 640;
  const MIN_BOTTOM_PANE_HEIGHT = 120;
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [wasmError, setWasmError] = useState<string | null>(null);
  const [runtimeFeedback, setRuntimeFeedback] = useState<string | null>(null);
  const [defaultCompileOptions, setDefaultCompileOptions] = useState<any>(null);
  const [supportedChipInfo, setSupportedChipInfo] = useState<ChipInfo[]>([]);
  const [audioRuntimeDebug, setAudioRuntimeDebug] = useState<AudioRuntimeDebugInfo>(
    audioService.getRuntimeDebugInfo()
  );
  const [waveformSamples, setWaveformSamples] = useState<number[]>(() => Array.from(audioService.getWaveformSnapshot(512)));
  const [activeBottomTab, setActiveBottomTab] = useSessionStorageState<string>('mml2vgm:activeBottomTab', 'output');
  const [bottomPaneMinimized, setBottomPaneMinimized] = useSessionStorageState<boolean>('mml2vgm:bottomPaneMinimized', false);
  const [bottomPaneHeight, setBottomPaneHeight] = useSessionStorageState<number>('mml2vgm:bottomPaneHeight', 200);
  const [isSidebarVisible, setIsSidebarVisible] = useSessionStorageState<boolean>('mml2vgm:isSidebarVisible', true);
  const [sidebarWidth, setSidebarWidth] = useSessionStorageState<number>('mml2vgm:sidebarWidth', 250);
  const [updateAvailable, setUpdateAvailable] = useState<boolean>(false);
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);
  const [hasSelection, setHasSelection] = useState<boolean>(false);
  const [canUndo, setCanUndo] = useState<boolean>(false);
  const [canRedo, setCanRedo] = useState<boolean>(false);
  const [hasCompileResult, setHasCompileResult] = useState<boolean>(false);
  const [activeNoteEvents, setActiveNoteEvents] = useState<SourceMapEvent[]>([]);
  // Dialog visibility — one state variable to avoid 8 separate booleans
  const [openDialog, setOpenDialog] = useState<string | null>(null);
  const closeDialog = useCallback(() => setOpenDialog(null), []);

  // Advanced compile options state (persisted locally, passed to dialog)
  const [advancedCompileOptions, setAdvancedCompileOptions] = useState<AdvancedCompileOptions>({
    targetChips: [],
    gd3Title: '',
    gd3Game: '',
    gd3Author: '',
    gd3Date: '',
    strictMode: false,
  });

  const wasmInitialized = useRef(false);
  const sidebarContainerRef = useRef<HTMLDivElement | null>(null);
  const bottomPanelRef = useRef<HTMLDivElement | null>(null);
  const suppressSidebarToggleClickRef = useRef(false);
  const suppressBottomToggleClickRef = useRef(false);
  const editorRef = useRef<MonacoEditorHandle | null>(null);

  // Document store
  const { documents, activeDocumentId, createDocument, setActiveDocument, closeDocument, closeAllDocuments } = useDocumentStore(
    useShallow((state) => ({
      documents: state.documents,
      activeDocumentId: state.activeDocumentId,
      createDocument: state.createDocument,
      setActiveDocument: state.setActiveDocument,
      closeDocument: state.closeDocument,
      closeAllDocuments: state.closeAllDocuments,
    }))
  );

  // Settings store
  const { settings, setSettings, updateEditorSettings, updateAudioSettings, updateHIDSettings, updateSerialSettings, setOutputFormat, togglePanelVisibility, setPanelVisibility } = useSettingsStore(
    useShallow((state) => ({
      settings: state.settings,
      setSettings: state.setSettings,
      updateEditorSettings: state.updateEditorSettings,
      updateAudioSettings: state.updateAudioSettings,
      updateHIDSettings: state.updateHIDSettings,
      updateSerialSettings: state.updateSerialSettings,
      setOutputFormat: state.setOutputFormat,
      togglePanelVisibility: state.togglePanelVisibility,
      setPanelVisibility: state.setPanelVisibility,
    }))
  );

  // Compile store (moved before useEffect that uses getResult)
  const { compile, cancel, status, getResult, progress, progressMessage, lastCompileTimingSummary } = useCompileStore(
    useShallow((state) => ({
      compile: state.compile,
      cancel: state.cancel,
      status: state.status,
      getResult: state.getResult,
      progress: state.progress,
      progressMessage: state.progressMessage,
      lastCompileTimingSummary: state.lastCompileTimingSummary,
    }))
  );

  // Get active document
  const activeDocument = activeDocumentId ? documents.get(activeDocumentId) || null : null;

  // Update selection and undo/redo state when editor or document changes
  useEffect(() => {
    if (editorRef.current && activeDocumentId) {
      setHasSelection(editorRef.current.hasSelection());
      setCanUndo(editorRef.current.canUndo());
      setCanRedo(editorRef.current.canRedo());
    } else {
      setHasSelection(false);
      setCanUndo(false);
      setCanRedo(false);
    }
  }, [activeDocumentId]);

  // Update hasCompileResult when active document or compile results change
  useEffect(() => {
    if (activeDocumentId) {
      const result = getResult(activeDocumentId);
      setHasCompileResult(!!result?.data);
    } else {
      setHasCompileResult(false);
    }
  }, [activeDocumentId, getResult]);

  const announceRuntimeFeedback = useCallback((message: string | null) => {
    setRuntimeFeedback(message);
    const announcer = document.getElementById('aria-live-announcer');
    if (announcer) {
      announcer.textContent = message || '';
    }
  }, []);

  // Helper function to create a basic timing map for trace playback
  // Maps time in milliseconds to source position (line, column)
  const createTimingMap = (source: string, durationMs: number): Map<number, Position> => {
    const timingMap = new Map<number, Position>();
    const lines = source.split('\n');
    const totalLines = lines.length;
    
    if (totalLines === 0 || durationMs <= 0) {
      return timingMap;
    }
    
    // Create a simple linear mapping: each line is evenly spaced across the duration
    const msPerLine = durationMs / totalLines;
    
    for (let line = 0; line < totalLines; line++) {
      const timeMs = Math.round(line * msPerLine);
      timingMap.set(timeMs, { line: line + 1, column: 1 }); // +1 for 1-indexed lines
    }
    
    // Also add end marker
    timingMap.set(durationMs, { line: totalLines, column: lines[totalLines - 1]?.length || 1 });
    
    return timingMap;
  };

  const getBrowserTargetChips = useCallback(() => {
    const browserDefaultTargets = supportedChipInfo
      .filter((chip) => chip.browserCompileDefault)
      .map((chip) => chip.variant.toLowerCase());

    // Fall back to the known-safe pair until support metadata has loaded.
    return browserDefaultTargets.length > 0 ? browserDefaultTargets : ['ym2608', 'sn76489'];
  }, [supportedChipInfo]);

  // Auto-reconnect WebHID devices if enabled in settings.
  useEffect(() => {
    if (settings.hid?.autoReconnect && hidService.isSupported()) {
      hidService.setReportFormat(settings.hid.reportFormat);
      hidService.setReportId(settings.hid.reportId);
      hidService.setByteOffset(settings.hid.byteOffset);
      hidService.tryRestoreDevices().then((count) => {
        if (count > 0) hidService.enable();
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Run once on mount only

  // Auto-reconnect WebSerial port if enabled in settings.
  // This is intentionally fire-and-forget; a failure is non-fatal.
  useEffect(() => {
    if (settings.serial?.autoReconnect && serialService.isSupported()) {
      serialService.tryRestorePort().then((found) => {
        if (found) {
          serialService.connect({
            baudRate: settings.serial.baudRate,
            protocol: settings.serial.protocol,
          });
        }
      });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Run once on mount only

  // Add beforeunload listener for unsaved changes
  useEffect(() => {
    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      const hasDirty = useDocumentStore.getState().hasDirtyDocuments();
      if (hasDirty) {
        event.preventDefault();
        event.returnValue = 'You have unsaved changes. Are you sure you want to leave?';
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => window.removeEventListener('beforeunload', handleBeforeUnload);
  }, []);

  // Get compiled data for active document
  const activeCompileResult = activeDocumentId ? getResult(activeDocumentId) : undefined;
  const compiledData = activeCompileResult?.data;

  useEffect(() => {
    const refresh = () => {
      setAudioRuntimeDebug(audioService.getRuntimeDebugInfo());
      setWaveformSamples(Array.from(audioService.getWaveformSnapshot(512)));
    };

    refresh();
    const id = window.setInterval(refresh, 200);
    return () => window.clearInterval(id);
  }, []);

  // Initialize all services on mount
  useEffect(() => {
    const initializeServices = async () => {
      // Initialize WASM
      if (!wasmInitialized.current) {
        try {
          await wasmService.init();
          setIsWasmReady(true);
          wasmInitialized.current = true;
        } catch (error) {
          setWasmError(`Failed to initialize WASM: ${error}`);
          console.error('WASM initialization error:', error);
          return; // Stop if WASM fails
        }
      }
      
      // Initialize other services (lazy, non-blocking)
      Promise.all([
        storageService.init().catch(e => console.warn('Storage init failed:', e)),
        i18nService.init().catch(e => console.warn('i18n init failed:', e)),
        registerServiceWorker().catch(e => console.warn('SW registration failed:', e)),
        wasmService.getDefaultCompileOptions().then(opts => setDefaultCompileOptions(opts)).catch(e => console.warn('Failed to get default compile options:', e)),
        wasmService.getSupportedChips().then(chips => setSupportedChipInfo(chips)).catch(e => console.warn('Failed to get supported chips:', e)),
        // Pre-warm workers for better UX (compilation won't block UI)
        (async () => {
          try {
            const { preWarmWorkers } = await import('@/services/workerService');
            await preWarmWorkers();
            console.log('[Worker] Pre-warming complete');
          } catch (e) {
            console.warn('[Worker] Pre-warming failed (will use fallback):', e);
          }
        })(),
      ]).then(() => {
        console.log('Phase 7 services initialized');
      }).catch(e => console.warn('Phase 7 services initialization had an error:', e));
      
      // Setup service worker update notification callback
      setUpdateNotificationCallback((available: boolean, version?: string) => {
        setUpdateAvailable(available);
        setUpdateVersion(version || null);
      });
      
      // Restore persisted playback settings
      const { audio } = useSettingsStore.getState().settings;
      audioService.setPlaybackRate(audio.playbackRate ?? 1.0);
      audioService.setLoop(audio.loop ?? false);

      // Create initial document if none exists
      if (documents.size === 0) {
        createDocument();
      }
    };

    initializeServices();

    return () => {
      // Cleanup if needed
    };
  }, []);

  useEffect(() => {
    const listener = {
      onError: (error: Error) => {
        announceRuntimeFeedback(`Audio error: ${error.message}`);
      },
    };

    audioService.addEventListener(listener);

    return () => {
      audioService.removeEventListener(listener);
    };
  }, [announceRuntimeFeedback]);

  // Handle document creation
  const handleNewDocument = useCallback(() => {
    createDocument();
  }, [createDocument]);

  // Get document store setters for file operations
  const { updateDocumentContent, updateDocumentFilename, updateDocumentFileHandle, setDocumentDirty } = useDocumentStore(
    useShallow((state) => ({
      updateDocumentContent: state.updateDocumentContent,
      updateDocumentFilename: state.updateDocumentFilename,
      updateDocumentFileHandle: state.updateDocumentFileHandle,
      setDocumentDirty: state.setDocumentDirty,
    }))
  );

  // Handle file open - must call showOpenFilePicker synchronously for user gesture
  const handleOpenFile = useCallback(() => {
    // Check if API is supported
    if (!('showOpenFilePicker' in window)) {
      console.error('File System Access API not supported in this browser');
      return;
    }
    
    // Call showOpenFilePicker directly to maintain user gesture context
    (window as any).showOpenFilePicker({
      types: [
        {
          description: 'MML Files',
          accept: {
            'text/plain': ['.gwi', '.mml', '.muc', '.mdl', '.mus', '.txt'],
          },
        },
      ],
      multiple: false,
    }).then((handles: any[]) => {
      if (!handles || handles.length === 0) {
        return; // User cancelled
      }
      
      const handle = handles[0];
      return handle.getFile();
    }).then((file: File) => {
      if (!file) return;
      
      return file.text().then((content) => {
        // Detect format from filename
        const detectedFormat = formatService.detectFromExtension(file.name);
        const doc = createDocument(detectedFormat || 'gwi');
        updateDocumentContent(doc.id, content);
        updateDocumentFilename(doc.id, file.name);
        setActiveDocument(doc.id);
      });
    }).catch((error: any) => {
      console.error('Failed to open file:', error);
    });
  }, [createDocument, updateDocumentContent, updateDocumentFilename, setActiveDocument]);

  // Handle loading an example file
  const handleLoadExample = useCallback(async (filename: string) => {
    try {
      const response = await fetch(`/samples/${filename}`);
      if (!response.ok) {
        throw new Error(`Failed to load example: ${response.status}`);
      }
      const content = await response.text();
      
      // Detect format from filename
      const detectedFormat = formatService.detectFromExtension(filename);
      const doc = createDocument(detectedFormat || 'gwi');
      updateDocumentContent(doc.id, content);
      updateDocumentFilename(doc.id, filename);
      setActiveDocument(doc.id);
    } catch (error) {
      console.error('Failed to load example:', error);
    }
  }, [createDocument, updateDocumentContent, updateDocumentFilename, setActiveDocument]);

  // Handle document close (from tab)
  const handleCloseDocument = useCallback((id: string) => {
    closeDocument(id);
  }, [closeDocument]);

  // Handle close active document (from menu)
  const handleCloseActiveDocument = useCallback(() => {
    if (activeDocumentId) {
      closeDocument(activeDocumentId);
    }
  }, [activeDocumentId, closeDocument]);

  // Handle close all documents (from menu)
  const handleCloseAllDocuments = useCallback(() => {
    closeAllDocuments();
  }, [closeAllDocuments]);

  // Handle save as document
  const handleSaveAs = useCallback(async () => {
    if (!activeDocumentId || !activeDocument) return;
    
    const doc = documents.get(activeDocumentId);
    if (!doc) return;
    
    // Check if File System Access API is supported
    if ('showSaveFilePicker' in window) {
      try {
        const options: any = {
          suggestedName: doc.filename,
          types: [
            {
              description: 'MML Files',
              accept: {
                'text/plain': ['.gwi', '.mml', '.muc', '.mdl', '.mus', '.txt'],
              },
            },
          ],
        };
        
        const handle = await (window as any).showSaveFilePicker(options);
        if (!handle) return; // User cancelled
        
        const writable = await handle.createWritable();
        await writable.write(doc.content);
        await writable.close();
        
        // Update document with new filename and file handle
        updateDocumentFilename(activeDocumentId, handle.name);
        updateDocumentFileHandle(activeDocumentId, handle);
        setDocumentDirty(activeDocumentId, false);
        
        announceRuntimeFeedback(`Saved to ${handle.name}`);
      } catch (error) {
        console.error('Failed to save file:', error);
        announceRuntimeFeedback(`Save error: ${error}`);
      }
    } else {
      // Fallback for browsers without File System Access API (Firefox, Safari)
      // Use Blob + URL.createObjectURL download pattern
      try {
        const blob = new Blob([doc.content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = doc.filename;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
        
        setDocumentDirty(activeDocumentId, false);
        announceRuntimeFeedback(`Downloaded ${doc.filename}`);
      } catch (error) {
        console.error('Failed to download file:', error);
        announceRuntimeFeedback(`Download error: ${error}`);
      }
    }
  }, [activeDocumentId, activeDocument, documents, updateDocumentFilename, updateDocumentFileHandle, setDocumentDirty, announceRuntimeFeedback]);

  // Handle exit with unsaved changes guard
  const handleExit = useCallback(() => {
    const hasDirty = useDocumentStore.getState().hasDirtyDocuments();
    
    if (hasDirty) {
      // Show confirmation dialog
      const shouldExit = window.confirm(
        'You have unsaved changes. Are you sure you want to exit?\n\n' +
        'Click OK to exit without saving, or Cancel to stay.'
      );
      
      if (!shouldExit) {
        return; // User cancelled
      }
    }
    
    // Try to close the window (only works if window was opened by script)
    window.close();
    
    // Note: window.close() only works when the tab was opened by script.
    // On a user-opened tab it silently fails. The beforeunload handler
    // covers the case when the user closes the tab directly via the browser UI.
  }, []);

  // Handle save document
  const handleSave = useCallback(async () => {
    if (!activeDocumentId || !activeDocument) return;
    
    const doc = documents.get(activeDocumentId);
    if (!doc) return;
    
    // If document has a file handle, save to it directly
    if (doc.fileHandle) {
      try {
        const writable = await (doc.fileHandle as any).createWritable();
        await writable.write(doc.content);
        await writable.close();
        
        // Mark as clean
        setDocumentDirty(activeDocumentId, false);
        announceRuntimeFeedback(`Saved to ${doc.filename}`);
        return;
      } catch (error) {
        console.error('Failed to save via file handle:', error);
        announceRuntimeFeedback(`Save error: ${error}`);
      }
    }
    
    // Fall through to Save As
    handleSaveAs();
  }, [activeDocumentId, activeDocument, documents, setDocumentDirty, announceRuntimeFeedback, handleSaveAs]);

  // Handle tab selection
  const handleSelectTab = useCallback((id: string) => {
    setActiveDocument(id);
  }, [setActiveDocument]);

  // Handle compile (compile only)
  const handleCompile = useCallback(async () => {
    console.log('[App] handleCompile called');
    console.log('[App] activeDocument:', !!activeDocument, 'status:', status, 'defaultCompileOptions:', !!defaultCompileOptions);
    if (!activeDocument || status === 'compiling') {
      console.log('[App] handleCompile returning early - condition check failed');
      return;
    }

    try {
      announceRuntimeFeedback(null);
      const options: any = {
        ...(defaultCompileOptions || {}),
        format: 'vgm',
        target_chips: getBrowserTargetChips(),
        // 0 means auto/default clock count from driver/MML.
        clock_count: 0,
      };

      console.log('[App] Calling compile with activeDocumentId:', activeDocumentId);
      await compile(activeDocumentId!, options);
    } catch (error) {
      console.error('Compilation error:', error);
      const message = error instanceof Error ? error.message : String(error);
      announceRuntimeFeedback(`Compile error: ${message}`);
    }
  }, [activeDocument, activeDocumentId, compile, status, defaultCompileOptions, announceRuntimeFeedback, getBrowserTargetChips]);

  // Handle compile and play (F5 behavior)
  const handleCompileAndPlay = useCallback(async () => {
    if (!activeDocument || status === 'compiling') return;
    
    try {
      announceRuntimeFeedback(null);
      const options: any = {
        ...(defaultCompileOptions || {}),
        format: 'vgm',
        target_chips: getBrowserTargetChips(),
        // 0 means auto/default clock count from driver/MML.
        clock_count: 0,
      };
      
      // Compile
      await compile(activeDocumentId!, options);
      
      // After compilation, get the result and auto-play
      const result = getResult(activeDocumentId!);
      if (!result) {
        throw new Error('Compilation finished but no result was returned.');
      }

      const chipsUsed = result.chipsUsed && result.chipsUsed.length > 0
        ? result.chipsUsed
        : [];
      const dataLength = result.data?.length || 0;
      const hasPlayableOutput = dataLength > 0;

      if (!hasPlayableOutput) {
        const feedback = `Compile output is not playable (bytes=${dataLength}, parts=${result.partCount}, commands=${result.commandCount}, chips=${chipsUsed.length}).`;
        useDocumentStore.getState().setCompileResults(activeDocumentId!, false, [
          {
            type: 'compile',
            message: feedback,
            line: 1,
            column: 1,
            length: 1,
            severity: 'error',
          },
        ]);
        throw new Error(feedback);
      }

      const chipsToPlay = chipsUsed.length > 0 ? chipsUsed : ['YM2608', 'SN76489'];

      partService.parseFromCompileResult(
        result.partCount,
        chipsToPlay,
        activeDocumentId
      );

      // Create a basic timing map based on duration
      const durationMs = (result.durationSeconds || 0) * 1000;
      const timingMap = createTimingMap(
        activeDocument.content,
        durationMs
      );

      // Initialize trace service with compile result
      traceService.init({
        data: result.data!,
        partCount: result.partCount,
        duration: durationMs,
        timingMap,
        sourceMap: result.source_map,
      });

      // Start trace playback
      traceService.start();

      // Play via audio service with chips from compile result
      await audioService.playVGM(result.data!, {
        chips: chipsToPlay as any[],
        volume: audioService.getVolume(),
      });
    } catch (error) {
      console.error('Compile and play error:', error);
      const message = error instanceof Error ? error.message : String(error);
      announceRuntimeFeedback(`Play error: ${message}`);
    }
  }, [activeDocument, activeDocumentId, compile, status, getResult, createTimingMap, defaultCompileOptions, announceRuntimeFeedback, getBrowserTargetChips]);

  // Handle play/pause
  const handlePlay = useCallback(() => {
    if (audioService.isPlaying()) {
      audioService.pause();
    } else {
      audioService.resume();
    }
  }, []);

  // Handle stop
  const handleStop = useCallback(() => {
    audioService.stop();
  }, []);

  // Handle Find (Ctrl+F) - delegates to Monaco's built-in find
  const handleFind = useCallback(() => {
    editorRef.current?.triggerCommand('actions.find');
  }, []);

  // Handle Replace (Ctrl+H) - delegates to Monaco's built-in replace
  const handleReplace = useCallback(() => {
    editorRef.current?.triggerCommand('editor.action.startFindReplaceAction');
  }, []);

  // Handle Select All - delegates to Monaco's built-in select all
  const handleSelectAll = useCallback(() => {
    editorRef.current?.triggerCommand('editor.action.selectAll');
  }, []);

  // Handle Cut - delegates to Monaco's built-in cut
  const handleCut = useCallback(() => {
    editorRef.current?.triggerCommand('editor.action.clipboardCutAction');
  }, []);

  // Handle Copy - delegates to Monaco's built-in copy
  const handleCopy = useCallback(() => {
    editorRef.current?.triggerCommand('editor.action.clipboardCopyAction');
  }, []);

  // Handle Paste - delegates to Monaco's built-in paste
  const handlePaste = useCallback(() => {
    editorRef.current?.triggerCommand('editor.action.clipboardPasteAction');
  }, []);

  // Handle Delete - delegates to Monaco's built-in delete
  const handleDelete = useCallback(() => {
    editorRef.current?.triggerCommand('deleteRight');
  }, []);

  // Handle Undo - delegates to Monaco's built-in undo
  const handleUndo = useCallback(() => {
    editorRef.current?.triggerCommand('undo');
  }, []);

  // Handle Redo - delegates to Monaco's built-in redo
  const handleRedo = useCallback(() => {
    editorRef.current?.triggerCommand('redo');
  }, []);

  // Handle export binary (VGM/XGM/ZGM)
  const handleExportBinary = useCallback(async (format: string) => {
    if (!activeDocumentId) return;
    
    const doc = documents.get(activeDocumentId);
    if (!doc) return;
    
    // Check if we already have a compile result
    const result = getResult(activeDocumentId);
    
    if (!result || !result.data) {
      // No compiled output yet, trigger a compile first
      announceRuntimeFeedback('No compiled output. Compiling first...');
      
      try {
        const options: any = {
          ...(defaultCompileOptions || {}),
          format: format as any,
          target_chips: getBrowserTargetChips(),
          clock_count: 0,
        };
        
        await compile(activeDocumentId, options);
        
        // Wait a moment for the result to be available
        await new Promise(resolve => setTimeout(resolve, 100));
      } catch (error) {
        console.error('Compile error before export:', error);
        const message = error instanceof Error ? error.message : String(error);
        announceRuntimeFeedback(`Compile error: ${message}`);
        return;
      }
    }
    
    // Get the result again (in case we just compiled)
    const exportResult = getResult(activeDocumentId);
    if (!exportResult || !exportResult.data) {
      announceRuntimeFeedback('No compiled output available for export.');
      return;
    }
    
    // Create filename: replace extension with format
    const baseName = doc.filename.replace(/\.\w+$/, '');
    const fileName = `${baseName}.${format}`;
    
    // Create blob and download
    const blob = new Blob([exportResult.data], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    
    announceRuntimeFeedback(`Exported to ${fileName}`);
  }, [activeDocumentId, documents, getResult, defaultCompileOptions, compile, announceRuntimeFeedback, getBrowserTargetChips]);

  // Phase 5.2: Playback speed and loop
  const handleSetPlaybackRate = useCallback((rate: number) => {
    audioService.setPlaybackRate(rate);
    updateAudioSettings({ playbackRate: rate });
  }, [updateAudioSettings]);

  const handleToggleLoop = useCallback(() => {
    const newLoop = !audioService.isLooping();
    audioService.setLoop(newLoop);
    updateAudioSettings({ loop: newLoop });
  }, [updateAudioSettings]);

  // Dialog openers
  const handleOpenAbout = useCallback(() => setOpenDialog('about'), []);
  const handleOpenKeyBindings = useCallback(() => setOpenDialog('keyBindings'), []);
  const handleOpenAudioSettings = useCallback(() => setOpenDialog('audioSettings'), []);
  const handleOpenAdvancedCompileOptions = useCallback(() => setOpenDialog('advancedCompileOptions'), []);
  const handleOpenMidiSettings = useCallback(() => setOpenDialog('midiSettings'), []);
  const handleOpenHIDSettings = useCallback(() => setOpenDialog('hidSettings'), []);
  const handleOpenSerialSettings = useCallback(() => setOpenDialog('serialSettings'), []);
  const handleOpenPreferences = useCallback(() => setOpenDialog('preferences'), []);
  const handleOpenHelp = useCallback(() => setOpenDialog('help'), []);
  const handleOpenMmlReference = useCallback(() => setOpenDialog('mmlReference'), []);

  // Upload Samples via menu bar — switch to the Samples tab and open file picker
  const [sampleUploadTrigger, setSampleUploadTrigger] = useState(0);
  const handleUploadSamples = useCallback(() => {
    setActiveBottomTab('samples');
    if (bottomPaneMinimized) setBottomPaneMinimized(false);
    setSampleUploadTrigger((prev) => prev + 1);
  }, [setActiveBottomTab, bottomPaneMinimized, setBottomPaneMinimized]);

  // Drag .wav files directly onto the editor area
  const [isEditorDragOver, setIsEditorDragOver] = useState(false);
  const editorDragCounterRef = useRef(0);

  const handleEditorDragOver = useCallback((e: React.DragEvent) => {
    const hasWav = Array.from(e.dataTransfer.items).some(
      (item) => item.kind === 'file' && (item.type === 'audio/wav' || item.type === 'audio/wave')
    );
    if (!hasWav) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
  }, []);

  const handleEditorDragEnter = useCallback((e: React.DragEvent) => {
    const hasWav = Array.from(e.dataTransfer.items).some(
      (item) => item.kind === 'file' && (item.type === 'audio/wav' || item.type === 'audio/wave')
    );
    if (!hasWav) return;
    e.preventDefault();
    editorDragCounterRef.current += 1;
    setIsEditorDragOver(true);
  }, []);

  const handleEditorDragLeave = useCallback(() => {
    editorDragCounterRef.current -= 1;
    if (editorDragCounterRef.current <= 0) {
      editorDragCounterRef.current = 0;
      setIsEditorDragOver(false);
    }
  }, []);

  const handleEditorDrop = useCallback(async (e: React.DragEvent) => {
    editorDragCounterRef.current = 0;
    setIsEditorDragOver(false);
    const files = Array.from(e.dataTransfer.files).filter((f) =>
      f.name.toLowerCase().endsWith('.wav')
    );
    if (files.length === 0) return;
    e.preventDefault();
    if (!activeDocumentId) return;
    for (const file of files) {
      try {
        const buf = await file.arrayBuffer();
        await sampleService.put(activeDocumentId, file.name, buf);
      } catch (err) {
        console.error('[App] Failed to upload dropped sample:', err);
      }
    }
    // Switch to Samples tab so user can see the result
    setActiveBottomTab('samples');
    if (bottomPaneMinimized) setBottomPaneMinimized(false);
    setSampleUploadTrigger((prev) => prev); // nudge without opening picker
  }, [activeDocumentId, setActiveBottomTab, bottomPaneMinimized, setBottomPaneMinimized]);

  // Phase 3.1: Zoom controls (adjust Monaco font size via settings store)
  const handleZoomIn = useCallback(() => {
    const current = settings.editor.fontSize ?? 14;
    updateEditorSettings({ fontSize: Math.min(current + 2, 32) });
  }, [settings.editor.fontSize, updateEditorSettings]);

  const handleZoomOut = useCallback(() => {
    const current = settings.editor.fontSize ?? 14;
    updateEditorSettings({ fontSize: Math.max(current - 2, 8) });
  }, [settings.editor.fontSize, updateEditorSettings]);

  const handleZoomReset = useCallback(() => {
    updateEditorSettings({ fontSize: 14 });
  }, [updateEditorSettings]);

  // Phase 4.1: Output format selection
  const handleSetOutputFormat = useCallback((format: string) => {
    setOutputFormat(format as any);
  }, [setOutputFormat]);

  // Phase 4.2 / 3.2: Show or toggle a panel in the bottom tabs / sidebar
  const handleShowPanel = useCallback((panelId: string) => {
    // Map panel keys to bottom tab IDs where applicable
    const panelToTabId: Record<string, string> = {
      errorList: 'output',
      runtime: 'runtime',
      compilation: 'compilation',
      waveform: 'waveform',
      samples: 'samples',
    };
    const tabId = panelToTabId[panelId];
    if (tabId) {
      setActiveBottomTab(tabId);
      if (bottomPaneMinimized) setBottomPaneMinimized(false);
    } else {
      // For sidebar panels, make them visible and show the sidebar
      setPanelVisibility(panelId as any, true);
      setIsSidebarVisible(true);
    }
  }, [setActiveBottomTab, bottomPaneMinimized, setBottomPaneMinimized, setPanelVisibility, setIsSidebarVisible]);

  // Phase 3.2: Toggle panel visibility
  const handleTogglePanel = useCallback((panelId: string) => {
    togglePanelVisibility(panelId as any);
  }, [togglePanelVisibility]);

  // Handle theme toggle
  const handleToggleTheme = useCallback(() => {
    const currentTheme = settings.editor.theme;
    const newTheme = currentTheme === 'vs-dark' ? 'vs' : 'vs-dark';
    const newIdTheme: 'dark' | 'light' | 'system' = newTheme === 'vs-dark' ? 'dark' : 'light';
    
    setSettings({
      ...settings,
      editor: {
        ...settings.editor,
        theme: newTheme,
      },
      theme: newIdTheme,
    });
  }, [settings, setSettings]);

  const handleToggleSidebar = useCallback(() => {
    if (suppressSidebarToggleClickRef.current) {
      suppressSidebarToggleClickRef.current = false;
      return;
    }
    setIsSidebarVisible((prev) => !prev);
  }, [setIsSidebarVisible]);

  const handleSidebarResizeStart = useCallback((event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();

    const containerRight = sidebarContainerRef.current?.getBoundingClientRect().right ?? window.innerWidth;
    const startX = event.clientX;
    let moved = false;

    const onMouseMove = (moveEvent: MouseEvent) => {
      const dragDistance = Math.abs(moveEvent.clientX - startX);
      if (dragDistance > 3) {
        moved = true;
      }

      const nextWidth = Math.round(containerRight - moveEvent.clientX);
      const clamped = Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, nextWidth));
      setSidebarWidth(clamped);
    };

    const onMouseUp = () => {
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';

      if (moved) {
        suppressSidebarToggleClickRef.current = true;
      }
    };

    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  }, [setSidebarWidth]);

  const handleBottomPaneToggle = useCallback(() => {
    if (suppressBottomToggleClickRef.current) {
      suppressBottomToggleClickRef.current = false;
      return;
    }

    setBottomPaneMinimized((prev) => !prev);
  }, [setBottomPaneMinimized]);

  const handleBottomPaneResizeStart = useCallback((event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();

    if (bottomPaneMinimized) {
      setBottomPaneMinimized(false);
    }

    const panelBottom = bottomPanelRef.current?.getBoundingClientRect().bottom ?? window.innerHeight;
    const startY = event.clientY;
    let moved = false;

    const onMouseMove = (moveEvent: MouseEvent) => {
      const dragDistance = Math.abs(moveEvent.clientY - startY);
      if (dragDistance > 3) {
        moved = true;
      }

      const maxBottomPaneHeight = Math.max(MIN_BOTTOM_PANE_HEIGHT + 40, Math.floor(window.innerHeight * 0.7));
      const nextHeight = Math.round(panelBottom - moveEvent.clientY);
      const clamped = Math.max(MIN_BOTTOM_PANE_HEIGHT, Math.min(maxBottomPaneHeight, nextHeight));
      setBottomPaneHeight(clamped);
    };

    const onMouseUp = () => {
      document.removeEventListener('mousemove', onMouseMove);
      document.removeEventListener('mouseup', onMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';

      if (moved) {
        suppressBottomToggleClickRef.current = true;
      }
    };

    document.body.style.cursor = 'row-resize';
    document.body.style.userSelect = 'none';
    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
  }, [bottomPaneMinimized, setBottomPaneHeight, setBottomPaneMinimized]);

  // Trace state
  const [traceStatus, setTraceStatus] = useState(traceService.getStatus());

  // State for navigation and error highlighting
  const [navigatePosition, setNavigatePosition] = useState<Position | null>(null);

  // Handle error navigation
  const handleNavigateToError = useCallback((position: Position) => {
    setNavigatePosition(position);
    // Clear after a moment to allow the effect to trigger
    setTimeout(() => setNavigatePosition(null), 500);
  }, []);

  // Listen to trace service events
  useEffect(() => {
    const handleTraceUpdate = () => {
      setTraceStatus(traceService.getStatus());
      setActiveNoteEvents(traceService.getActiveNoteEvents());
    };

    const listener = {
      onTraceStart: handleTraceUpdate,
      onTraceStop: handleTraceUpdate,
      onTracePause: handleTraceUpdate,
      onTraceResume: handleTraceUpdate,
      onPositionUpdate: handleTraceUpdate,
      onPartEvent: handleTraceUpdate,
      onRegisterWrite: handleTraceUpdate,
    };

    traceService.addEventListener(listener);

    return () => {
      traceService.removeEventListener(listener);
    };
  }, []);

  // Get active document for panel props
  const activeDoc = activeDocument;
  
  // Render panel based on type
  const renderPanel = useCallback((panelType: PanelType) => {
    switch (panelType) {
      case 'errorList':
        return <ErrorListPanel onNavigateToPosition={handleNavigateToError} />;
      case 'partCounter':
        return <PartCounterPanel documentId={activeDocumentId || undefined} />;
      case 'folderTree':
        return <FolderTreePanel />;
      case 'playback':
        return <PlaybackPanel compiledData={compiledData} />;
      case 'compileOptions':
        return <CompileOptionsPanel />;
      case 'info':
        return <InfoPanel />;
      case 'lyrics':
        return <LyricsPanel />;
      case 'mixer':
        return <MixerPanel />;
      case 'midiKeyboard':
        return <MIDIKeyboardPanel />;
      case 'debug':
        return <DebugPanel />;
      case 'runtime':
        return <RuntimePanel audioRuntimeDebug={audioRuntimeDebug} />;
      case 'compilation':
        return <CompilationPanel />;
      case 'waveform':
        return <WaveformPanel waveformSamples={waveformSamples} />;
      case 'script':
        return (
          <ScriptPanel
            documentId={activeDocumentId || undefined}
            documentContent={activeDoc?.content || ''}
            documentLanguage={activeDoc?.language || 'gwi'}
          />
        );
      case 'fmToneEditor':
        return <FmToneEditorPanel />;
      case 'envelopeEditor':
        return <EnvelopeEditorPanel />;
      case 'arpeggioEditor':
        return <ArpeggioEditorPanel />;
      default:
        return null;
    }
  }, [compiledData, handleNavigateToError, activeDocumentId, activeDoc, audioRuntimeDebug, waveformSamples]);

  // Get panels for right sidebar (positioned right)
  const allPanelTypes: PanelType[] = [
    'folder', 'folderTree', 'partCounter', 'errorList', 'log', 'lyrics',
    'mixer', 'midiKeyboard', 'debug', 'playback', 'compileOptions', 'info', 'script',
    'runtime', 'compilation', 'waveform',
    'fmToneEditor', 'envelopeEditor', 'arpeggioEditor',
  ];
  
  const rightSidebarPanelTypes: PanelType[] = allPanelTypes.filter(
    (p) => settings.panelPositions[p] === 'right' && settings.panelVisibility[p]
  );

  // Loading state
  if (!isWasmReady && !wasmError) {
    return (
      <div className="app-container" style={{ justifyContent: 'center', alignItems: 'center' }}>
        <div style={{ textAlign: 'center' }}>
          <h2>Initializing WASM module...</h2>
          <p>Please wait while we load the compiler.</p>
        </div>
      </div>
    );
  }

  // Error state
  if (wasmError) {
    return (
      <div className="app-container" style={{ justifyContent: 'center', alignItems: 'center' }}>
        <div style={{ textAlign: 'center', color: 'red' }}>
          <h2>Error Loading WASM Module</h2>
          <p>{wasmError}</p>
          <p>Please ensure you have an internet connection and try refreshing the page.</p>
          <button onClick={() => window.location.reload()} style={{ marginTop: '16px' }}>
            Refresh Page
          </button>
        </div>
      </div>
    );
  }

  // Right sidebar panels
  const rightSidebarPanels = rightSidebarPanelTypes.map((p) => (
    <React.Fragment key={p}>{renderPanel(p)}</React.Fragment>
  ));

  // Bottom tabs
  const bottomTabs: BottomTab[] = [
    {
      id: 'output',
      label: 'Output',
      content: <ErrorListPanel onNavigateToPosition={handleNavigateToError} />,
    },
    {
      id: 'runtime',
      label: 'Runtime',
      content: <RuntimePanel audioRuntimeDebug={audioRuntimeDebug} />,
    },
    {
      id: 'compilation',
      label: 'Info',
      content: (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%', overflowY: 'auto' }}>
          <CompilationPanel />
          <InfoPanel />
        </div>
      ),
    },
    {
      id: 'waveform',
      label: 'Waveform',
      content: <WaveformPanel waveformSamples={waveformSamples} />,
    },
    {
      id: 'samples',
      label: 'Samples',
      content: <SamplesPanel uploadTrigger={sampleUploadTrigger} />,
    },
  ];

  return (
    <div className="app-container" data-theme={settings.theme}>
      {/* Skip link for keyboard accessibility (Phase 7) */}
      <a href="#editor-container" className="skip-link">
        Skip to editor
      </a>
      
      {/* ARIA live region for announcements */}
      <div role="status" aria-live="polite" className="aria-live" id="aria-live-announcer"></div>
      
      {/* Menu Bar */}
      <MenuBar
        onNewDocument={handleNewDocument}
        onOpenFile={handleOpenFile}
        onCloseDocument={handleCloseActiveDocument}
        onCloseAllDocuments={handleCloseAllDocuments}
        onToggleTheme={handleToggleTheme}
        onToggleSidebar={handleToggleSidebar}
        isSidebarVisible={isSidebarVisible}
        onCompile={handleCompile}
        onCompileAndPlay={handleCompileAndPlay}
        onPlay={handlePlay}
        onStop={handleStop}
        onLoadExample={handleLoadExample}
        hasActiveDocument={!!activeDocumentId}
        hasMultipleDocuments={documents.size > 1}
        isCompiling={status === 'compiling'}
        isPlaying={audioService.isPlaying()}
        // Edit menu
        onFind={handleFind}
        onReplace={handleReplace}
        onSelectAll={handleSelectAll}
        onCut={handleCut}
        onCopy={handleCopy}
        onPaste={handlePaste}
        onDelete={handleDelete}
        onUndo={handleUndo}
        onRedo={handleRedo}
        hasSelection={hasSelection}
        canUndo={canUndo}
        canRedo={canRedo}
        // File menu - Export
        onExportBinary={handleExportBinary}
        hasCompileResult={hasCompileResult}
        // File menu - Save
        onSave={handleSave}
        onSaveAs={handleSaveAs}
        onExit={handleExit}
        // Phase 3.1: Zoom
        onZoomIn={handleZoomIn}
        onZoomOut={handleZoomOut}
        onZoomReset={handleZoomReset}
        fontSize={settings.editor.fontSize}
        // Phase 4.1: Output format
        onSetOutputFormat={handleSetOutputFormat}
        outputFormat={settings.outputFormat}
        // Phase 4.2 / 3.2: Panel access and visibility
        onShowPanel={handleShowPanel}
        onTogglePanel={handleTogglePanel}
        panelVisibility={settings.panelVisibility}
        // Phase 5.2: Playback speed and loop
        onSetPlaybackRate={handleSetPlaybackRate}
        playbackRate={settings.audio.playbackRate}
        onToggleLoop={handleToggleLoop}
        isLooping={audioService.isLooping()}
        // Dialogs
        onOpenAbout={handleOpenAbout}
        onOpenKeyBindings={handleOpenKeyBindings}
        onOpenAudioSettings={handleOpenAudioSettings}
        onOpenAdvancedCompileOptions={handleOpenAdvancedCompileOptions}
        onOpenMidiSettings={handleOpenMidiSettings}
        onOpenHIDSettings={handleOpenHIDSettings}
        onOpenSerialSettings={handleOpenSerialSettings}
        onOpenPreferences={handleOpenPreferences}
        onOpenHelp={handleOpenHelp}
        onOpenMmlReference={handleOpenMmlReference}
        onUploadSamples={handleUploadSamples}
      />

      {/* Tab Bar */}
      {documents.size > 0 && (
        <TabBar
          documents={Array.from(documents.values())}
          activeDocumentId={activeDocumentId || ''}
          onSelectTab={handleSelectTab}
          onCloseTab={handleCloseDocument}
        />
      )}

      {runtimeFeedback && (
        <div
          role="alert"
          style={{
            margin: '0 8px 8px',
            padding: '8px 10px',
            border: '1px solid var(--status-error-fg, #d64545)',
            background: 'var(--status-error-bg, rgba(214, 69, 69, 0.12))',
            color: 'var(--status-error-fg, #d64545)',
            borderRadius: '4px',
            fontSize: '12px',
          }}
        >
          {runtimeFeedback}
        </div>
      )}

      {/* Service Worker Update Notification */}
      {updateAvailable && (
        <div
          role="alert"
          style={{
            margin: '0 8px 8px',
            padding: '8px 12px',
            border: '1px solid var(--status-info-fg, #4a90d9)',
            background: 'var(--status-info-bg, rgba(74, 144, 217, 0.12))',
            color: 'var(--status-info-fg, #4a90d9)',
            borderRadius: '4px',
            fontSize: '12px',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
          }}
        >
          <span>↻ Update available {updateVersion && `(v${updateVersion})`}. Refresh to apply.</span>
          <button
            type="button"
            className="button small"
            onClick={() => window.location.reload()}
            style={{ fontSize: '11px', padding: '2px 8px' }}
            aria-label="Refresh to apply update"
          >
            Refresh
          </button>
          <button
            type="button"
            className="button small"
            onClick={() => setUpdateAvailable(false)}
            style={{ fontSize: '11px', padding: '2px 8px' }}
            aria-label="Dismiss update notification"
          >
            Later
          </button>
        </div>
      )}

      {/* Main Layout */}
      <div className="main-layout">
        <div className="editor-column">
          {/* Editor Area */}
          <div
            className={`editor-container${isEditorDragOver ? ' editor-wav-drag-over' : ''}`}
            id="editor-container"
            role="main"
            aria-label="MML Editor"
            onDragOver={handleEditorDragOver}
            onDragEnter={handleEditorDragEnter}
            onDragLeave={handleEditorDragLeave}
            onDrop={handleEditorDrop}
          >
            {activeDocument && (
              <MonacoEditor
                ref={editorRef}
                document={activeDocument}
                onChange={(content) => {
                  useDocumentStore.getState().updateDocumentContent(activeDocumentId!, content);
                }}
                settings={settings.editor}
                currentPosition={traceStatus.currentPosition}
                navigationPosition={navigatePosition}
                activeNoteEvents={activeNoteEvents}
              />
            )}

            {status === 'compiling' && (
              <div className="compile-overlay" role="status" aria-live="polite" aria-label="Compiling">
                <div className="compile-spinner" aria-hidden="true" />
                <div className="compile-overlay-text">
                  <strong>Compiling...</strong>
                  <span>
                    {progressMessage || 'Processing MML in background worker'}
                    {typeof progress === 'number' && progress > 0 ? ` (${Math.round(progress)}%)` : ''}
                  </span>
                </div>
                <button
                  type="button"
                  className="button small danger"
                  onClick={() => cancel()}
                  aria-label="Cancel compilation"
                >
                  Cancel
                </button>
              </div>
            )}

            {isEditorDragOver && (
              <div className="editor-wav-drop-overlay" aria-hidden="true">
                Drop .wav files to add to sample library
              </div>
            )}
          </div>

          {/* Bottom Panel (left/editor side) */}
          <div
            className={`bottom-panel-wrapper ${bottomPaneMinimized ? 'minimized' : ''}`}
            ref={bottomPanelRef}
            style={{ height: bottomPaneMinimized ? '28px' : `${bottomPaneHeight}px` }}
          >
            <button
              type="button"
              className="bottom-panel-resizer"
              onMouseDown={handleBottomPaneResizeStart}
              onClick={handleBottomPaneToggle}
              aria-label={bottomPaneMinimized ? 'Expand bottom panel' : 'Collapse bottom panel'}
              title={bottomPaneMinimized ? 'Drag to resize, click to expand' : 'Drag to resize, click to collapse'}
            />
            <BottomTabs
              tabs={bottomTabs}
              activeTabId={activeBottomTab}
              onTabClick={setActiveBottomTab}
              isMinimized={bottomPaneMinimized}
              onMinimize={() => setBottomPaneMinimized(true)}
              onMaximize={() => setBottomPaneMinimized(false)}
            />
          </div>
        </div>

        {/* Right Sidebar */}
        {isSidebarVisible && rightSidebarPanels.length > 0 && (
          <div className="panel-container" ref={sidebarContainerRef} style={{ width: `${sidebarWidth}px` }}>
            <button
              type="button"
              className="sidebar-border-toggle"
              onMouseDown={handleSidebarResizeStart}
              onClick={handleToggleSidebar}
              aria-label="Hide sidebar"
              title="Drag to resize, click to hide sidebar"
            />
            {rightSidebarPanels}
          </div>
        )}

        {!isSidebarVisible && rightSidebarPanels.length > 0 && (
          <button
            type="button"
            className="sidebar-border-toggle sidebar-border-toggle--collapsed"
            onClick={handleToggleSidebar}
            aria-label="Show sidebar"
            title="Show sidebar"
          />
        )}
      </div>

      {/* Status Bar */}
      <StatusBar
        document={activeDocument}
        compileStatus={status}
        progress={progress}
        progressMessage={progressMessage}
        lastCompileTimingSummary={lastCompileTimingSummary}
        isAudioPlaying={audioRuntimeDebug.isPlaying}
        activeNoteEvents={activeNoteEvents}
      />

      {/* Dialogs */}
      <AboutDialog isOpen={openDialog === 'about'} onClose={closeDialog} />
      <KeyBindingsDialog isOpen={openDialog === 'keyBindings'} onClose={closeDialog} />
      <AudioSettingsDialog
        isOpen={openDialog === 'audioSettings'}
        onClose={closeDialog}
        settings={settings.audio}
        onSave={(updates) => updateAudioSettings(updates)}
      />
      <AdvancedCompileOptionsDialog
        isOpen={openDialog === 'advancedCompileOptions'}
        onClose={closeDialog}
        chips={supportedChipInfo}
        options={advancedCompileOptions}
        onSave={setAdvancedCompileOptions}
      />
      <MidiSettingsDialog isOpen={openDialog === 'midiSettings'} onClose={closeDialog} />
      <HIDSettingsDialog
        isOpen={openDialog === 'hidSettings'}
        onClose={closeDialog}
        settings={settings.hid}
        onSave={updateHIDSettings}
      />
      <SerialSettingsDialog
        isOpen={openDialog === 'serialSettings'}
        onClose={closeDialog}
        settings={settings.serial}
        onSave={updateSerialSettings}
      />
      <PreferencesDialog
        isOpen={openDialog === 'preferences'}
        onClose={closeDialog}
        settings={settings}
        onSave={(updates) => setSettings({ ...settings, ...updates })}
      />
      <HelpDialog isOpen={openDialog === 'help'} onClose={closeDialog} />
      <MmlReferenceDialog isOpen={openDialog === 'mmlReference'} onClose={closeDialog} />
    </div>
  );
};

export default App;
