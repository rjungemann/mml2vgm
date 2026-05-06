import React, { useEffect, useState, useCallback, useRef } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { wasmService } from '@/services/wasmService';
import { audioService, type AudioRuntimeDebugInfo } from '@/services/audioService';
import { traceService } from '@/services/traceService';
import { partService } from '@/services/partService';
import { formatService } from '@/services/formatService';
import { scriptService } from '@/services/scriptService';
import { storageService, registerServiceWorker } from '@/services/storageService';
import { i18nService } from '@/services/i18nService';
import { fileService } from '@/services/fileService';
import { useDocumentStore } from '@/stores/documentStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useCompileStore } from '@/stores/compileStore';
import type { PanelType, Position, MMLLanguage } from '@/types';
import MonacoEditor from '@/components/Editor/MonacoEditor';
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
import AudioWaveformView from '@/components/AudioWaveformView';
import { TabBar } from '@/components/TabBar';

export const App: React.FC = () => {
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [wasmError, setWasmError] = useState<string | null>(null);
  const [runtimeFeedback, setRuntimeFeedback] = useState<string | null>(null);
  const [defaultCompileOptions, setDefaultCompileOptions] = useState<any>(null);
  const [showRuntimeDebug, setShowRuntimeDebug] = useState(false);
  const [audioRuntimeDebug, setAudioRuntimeDebug] = useState<AudioRuntimeDebugInfo>(
    audioService.getRuntimeDebugInfo()
  );
  const [waveformSamples, setWaveformSamples] = useState<number[]>(() => Array.from(audioService.getWaveformSnapshot(512)));
  const wasmInitialized = useRef(false);

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
  const { settings, setSettings } = useSettingsStore(
    useShallow((state) => ({
      settings: state.settings,
      setSettings: state.setSettings,
    }))
  );

  // Get active document
  const activeDocument = activeDocumentId ? documents.get(activeDocumentId) || null : null;

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
    // Current VGM codegen path still relies on PSG writes for many browser samples.
    return ['ym2608', 'sn76489'];
  }, []);

  // Compile store
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
  const { updateDocumentContent, updateDocumentFilename, updateDocumentLanguage } = useDocumentStore(
    useShallow((state) => ({
      updateDocumentContent: state.updateDocumentContent,
      updateDocumentFilename: state.updateDocumentFilename,
      updateDocumentLanguage: state.updateDocumentLanguage,
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
        const docId = createDocument(detectedFormat || 'gwi');
        updateDocumentContent(docId, content);
        updateDocumentFilename(docId, file.name);
        setActiveDocument(docId);
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
      case 'script':
        return (
          <ScriptPanel
            documentId={activeDocumentId || undefined}
            documentContent={activeDoc?.content || ''}
            documentLanguage={activeDoc?.language || 'gwi'}
          />
        );
      default:
        return null;
    }
  }, [compiledData, handleNavigateToError, activeDocumentId, activeDoc]);

  // Get panels for right sidebar (positioned right)
  const allPanelTypes: PanelType[] = [
    'folder', 'folderTree', 'partCounter', 'errorList', 'log', 'lyrics', 
    'mixer', 'midiKeyboard', 'debug', 'playback', 'compileOptions', 'info', 'script'
  ];
  
  const rightSidebarPanelTypes: PanelType[] = allPanelTypes.filter(
    (p) => settings.panelPositions[p] === 'right' && settings.panelVisibility[p]
  );

  // Get panel for bottom (positioned bottom)
  const bottomPanelType: PanelType | null = allPanelTypes.find(
    (p) => settings.panelPositions[p] === 'bottom' && settings.panelVisibility[p]
  ) || null;

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

  // Bottom panel
  const bottomPanel = bottomPanelType ? renderPanel(bottomPanelType) : null;

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
        onCompile={handleCompile}
        onCompileAndPlay={handleCompileAndPlay}
        onPlay={handlePlay}
        onStop={handleStop}
        onLoadExample={handleLoadExample}
        hasActiveDocument={!!activeDocumentId}
        hasMultipleDocuments={documents.size > 1}
        isCompiling={status === 'compiling'}
        isPlaying={audioService.isPlaying()}
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

      <div
        style={{
          margin: '0 8px 8px',
          border: '1px solid var(--border-color)',
          borderRadius: '4px',
          background: 'var(--bg-tertiary)',
          fontSize: '12px',
        }}
      >
        <button
          type="button"
          onClick={() => setShowRuntimeDebug((v) => !v)}
          style={{
            width: '100%',
            textAlign: 'left',
            border: 'none',
            background: 'transparent',
            color: 'var(--text-primary)',
            padding: '8px 10px',
            cursor: 'pointer',
            fontSize: '12px',
            fontWeight: 600,
          }}
        >
          Runtime Debug {showRuntimeDebug ? '▼' : '▶'}
        </button>

        {showRuntimeDebug && (
          <div
            style={{
              padding: '8px 10px 10px',
              borderTop: '1px solid var(--border-color)',
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
              gap: '6px 12px',
              color: 'var(--text-primary)',
            }}
          >
            <div>Compile bytes: {activeCompileResult?.data?.length || 0}</div>
            <div>Compile parts: {activeCompileResult?.partCount || 0}</div>
            <div>Compile commands: {activeCompileResult?.commandCount || 0}</div>
            <div>Compile chips: {(activeCompileResult?.chipsUsed || []).join(', ') || '(none)'}</div>

            <div>Audio playing: {audioRuntimeDebug.isPlaying ? 'yes' : 'no'}</div>
            <div>Audio paused: {audioRuntimeDebug.isPaused ? 'yes' : 'no'}</div>
            <div>VGM bytes loaded: {audioRuntimeDebug.vgmDataLength}</div>
            <div>Parsed VGM commands: {audioRuntimeDebug.parsedCommandCount}</div>
            <div>Commands applied: {audioRuntimeDebug.appliedWriteCount}</div>
            <div>Commands skipped: {audioRuntimeDebug.skippedWriteCount}</div>
            <div>Pending commands: {audioRuntimeDebug.pendingCommandCount}</div>
            <div>Buffers generated: {audioRuntimeDebug.generatedBufferCount}</div>
            <div>Silent buffer streak: {audioRuntimeDebug.silentBufferCount}</div>
            <div>Last peak: {audioRuntimeDebug.lastPeak.toExponential(3)}</div>
            <div>Silence warning emitted: {audioRuntimeDebug.emittedSilenceWarning ? 'yes' : 'no'}</div>
            <div>Playback chips: {audioRuntimeDebug.chips.join(', ') || '(none)'}</div>

            <div style={{ gridColumn: '1 / -1' }}>
              <div style={{ marginBottom: 6, fontWeight: 600 }}>Waveform</div>
              <AudioWaveformView samples={waveformSamples} />
            </div>
          </div>
        )}
      </div>

      {/* Main Layout */}
      <div className="main-layout">
        <div className="editor-column">
          {/* Editor Area */}
          <div className="editor-container" id="editor-container" role="main" aria-label="MML Editor">
            {activeDocument && (
              <MonacoEditor
                document={activeDocument}
                onChange={(content) => {
                  useDocumentStore.getState().updateDocumentContent(activeDocumentId!, content);
                }}
                settings={settings.editor}
                currentPosition={traceStatus.currentPosition}
                navigationPosition={navigatePosition}
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
          </div>

          {/* Bottom Panel (left/editor side) */}
          {bottomPanel && (
            <div className="bottom-panel">
              <div className="bottom-panel-header">
                <span className="panel-title">
                  {bottomPanelType === 'errorList' ? 'Errors' :
                   bottomPanelType === 'info' ? 'Information' :
                   bottomPanelType === 'log' ? 'Output Log' :
                   'Output'}
                </span>
              </div>
              <div className="bottom-panel-content">
                {bottomPanel}
              </div>
              <div className="bottom-panel-resizer" />
            </div>
          )}
        </div>

        {/* Right Sidebar */}
        {rightSidebarPanels.length > 0 && (
          <div className="panel-container">
            {rightSidebarPanels}
          </div>
        )}
      </div>

      {/* Status Bar */}
      <StatusBar
        document={activeDocument}
        compileStatus={status}
        progress={progress}
        progressMessage={progressMessage}
        lastCompileTimingSummary={lastCompileTimingSummary}
        waveformSamples={waveformSamples}
        isAudioPlaying={audioRuntimeDebug.isPlaying}
        onToggleRuntimeDebug={() => setShowRuntimeDebug((v) => !v)}
        runtimeDebugVisible={showRuntimeDebug}
      />
    </div>
  );
};

export default App;
