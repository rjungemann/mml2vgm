import React, { useEffect, useState, useCallback, useRef } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { wasmService } from '@/services/wasmService';
import { audioService } from '@/services/audioService';
import { traceService } from '@/services/traceService';
import { useDocumentStore } from '@/stores/documentStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useCompileStore } from '@/stores/compileStore';
import type { PanelType, Position } from '@/types';
import MonacoEditor from '@/components/Editor/MonacoEditor';
import StatusBar from '@/components/StatusBar';
import MenuBar from '@/components/MenuBar';
import ErrorListPanel from '@/components/panels/ErrorListPanel';
import PartCounterPanel from '@/components/panels/PartCounterPanel';
import FolderTreePanel from '@/components/panels/FolderTreePanel';
import PlaybackPanel from '@/components/panels/PlaybackPanel';
import CompileOptionsPanel from '@/components/panels/CompileOptionsPanel';
import InfoPanel from '@/components/panels/InfoPanel';
import { TabBar } from '@/components/TabBar';

export const App: React.FC = () => {
  const [isWasmReady, setIsWasmReady] = useState(false);
  const [wasmError, setWasmError] = useState<string | null>(null);
  const wasmInitialized = useRef(false);

  // Document store
  const { documents, activeDocumentId, createDocument, setActiveDocument, closeDocument } = useDocumentStore(
    useShallow((state) => ({
      documents: state.documents,
      activeDocumentId: state.activeDocumentId,
      createDocument: state.createDocument,
      setActiveDocument: state.setActiveDocument,
      closeDocument: state.closeDocument,
    }))
  );

  // Settings store
  const { settings, setSettings } = useSettingsStore(
    useShallow((state) => ({
      settings: state.settings,
      setSettings: state.setSettings,
    }))
  );

  // Compile store
  const { compile, status, getResult } = useCompileStore(
    useShallow((state) => ({
      compile: state.compile,
      status: state.status,
      getResult: state.getResult,
    }))
  );

  // Get compiled data for active document
  const compiledData = activeDocumentId ? getResult(activeDocumentId)?.data : undefined;

  // Initialize WASM on mount
  useEffect(() => {
    const initializeWasm = async () => {
      if (wasmInitialized.current) return;
      
      try {
        await wasmService.init();
        setIsWasmReady(true);
        wasmInitialized.current = true;
        
        // Create initial document if none exists
        if (documents.size === 0) {
          createDocument();
        }
      } catch (error) {
        setWasmError(`Failed to initialize WASM: ${error}`);
        console.error('WASM initialization error:', error);
      }
    };

    initializeWasm();

    return () => {
      // Cleanup if needed
    };
  }, []);

  // Handle document creation
  const handleNewDocument = useCallback(() => {
    createDocument();
  }, [createDocument]);

  // Handle document close
  const handleCloseDocument = useCallback((id: string) => {
    closeDocument(id);
  }, [closeDocument]);

  // Handle tab selection
  const handleSelectTab = useCallback((id: string) => {
    setActiveDocument(id);
  }, [setActiveDocument]);

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

  // Get active document
  const activeDocument = activeDocumentId ? documents.get(activeDocumentId) || null : null;
  
  // Trace state
  const [traceStatus, setTraceStatus] = useState(traceService.getStatus());

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

  // Handle compile
  const handleCompile = useCallback(async () => {
    if (!activeDocument || status === 'compiling') return;
    
    try {
      const options: any = {
        format: 'vgm',
        target_chips: ['YM2608'],
        clockRate: 7987200,
        optimize: true,
      };
      
      await compile(activeDocumentId!, options);
    } catch (error) {
      console.error('Compilation error:', error);
    }
  }, [activeDocument, activeDocumentId, compile, status]);

  // Render panel based on type
  const renderPanel = useCallback((panelType: PanelType) => {
    switch (panelType) {
      case 'errorList':
        return <ErrorListPanel />;
      case 'partCounter':
        return <PartCounterPanel />;
      case 'folderTree':
        return <FolderTreePanel />;
      case 'playback':
        return <PlaybackPanel compiledData={compiledData} />;
      case 'compileOptions':
        return <CompileOptionsPanel />;
      case 'info':
        return <InfoPanel />;
      default:
        return null;
    }
  }, [compiledData]);

  // Get panels for right sidebar (positioned right)
  const allPanelTypes: PanelType[] = [
    'folder', 'folderTree', 'partCounter', 'errorList', 'log', 'lyrics', 
    'mixer', 'midiKeyboard', 'debug', 'playback', 'compileOptions', 'info'
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
  const rightSidebarPanels = rightSidebarPanelTypes.map((p) => renderPanel(p));

  // Bottom panel
  const bottomPanel = bottomPanelType ? renderPanel(bottomPanelType) : null;

  return (
    <div className="app-container" data-theme={settings.theme}>
      {/* Menu Bar */}
      <MenuBar
        onNewDocument={handleNewDocument}
        onToggleTheme={handleToggleTheme}
        onCompile={handleCompile}
        isCompiling={status === 'compiling'}
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

      {/* Main Layout */}
      <div className="main-layout">
        {/* Editor Area */}
        <div className="editor-container">
          {activeDocument && (
            <MonacoEditor
              document={activeDocument}
              onChange={(content) => {
                useDocumentStore.getState().updateDocumentContent(activeDocumentId!, content);
              }}
              settings={settings.editor}
              isTracing={traceStatus.isTracing}
              currentPosition={traceStatus.currentPosition}
              activeParts={traceStatus.activeParts}
            />
          )}
        </div>

        {/* Right Sidebar */}
        {rightSidebarPanels.length > 0 && (
          <div className="panel-container">
            {rightSidebarPanels}
          </div>
        )}
      </div>

      {/* Bottom Panel */}
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

      {/* Status Bar */}
      <StatusBar
        document={activeDocument}
        compileStatus={status}
        progress={0}
        progressMessage={''}
      />
    </div>
  );
};

export default App;
