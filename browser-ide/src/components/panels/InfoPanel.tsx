import React from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useCompileStore } from '@/stores/compileStore';
import { useDocumentStore } from '@/stores/documentStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { wasmService } from '@/services/wasmService';

const InfoPanel: React.FC = () => {
  const { getResult } = useCompileStore(
    useShallow((state) => ({
      getResult: state.getResult,
    }))
  );

  const { activeDocumentId, documents } = useDocumentStore(
    useShallow((state) => ({
      activeDocumentId: state.activeDocumentId,
      documents: state.documents,
    }))
  );

  const { settings } = useSettingsStore(
    useShallow((state) => ({
      settings: state.settings,
    }))
  );

  // Get active document
  const activeDocument = activeDocumentId ? documents.get(activeDocumentId) : null;
  
  // Get compile result for active document
  const activeResult = activeDocumentId ? getResult(activeDocumentId) : null;

  // Format bytes
  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  // Format duration
  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 1000);
    return `${mins}:${secs.toString().padStart(2, '0')}.${ms.toString().padStart(3, '0')}`;
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '4px', fontSize: '12px' }}>
      <div style={{ marginBottom: '8px' }}>
        <h3 style={{ margin: '0 0 4px 0', fontSize: '13px' }}>IDE Information</h3>
        <div style={{ backgroundColor: 'var(--bg-tertiary)', padding: '4px', borderRadius: '3px' }}>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>Version:</span> 
            <span>mml2vgm Browser IDE v1.0.0</span>
          </div>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>WASM:</span> 
            <span style={{ color: wasmService.isInitialized ? '#4caf50' : '#ff5555' }}>
              {wasmService.isInitialized ? 'Initialized' : 'Not initialized'}
            </span>
          </div>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>Theme:</span> 
            <span>{settings.editor.theme}</span>
          </div>
        </div>
      </div>

      {activeDocument && (
        <div style={{ marginBottom: '8px' }}>
          <h3 style={{ margin: '0 0 4px 0', fontSize: '13px' }}>Document Information</h3>
          <div style={{ backgroundColor: 'var(--bg-tertiary)', padding: '4px', borderRadius: '3px' }}>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Filename:</span> 
              <span>{activeDocument.filename || 'Untitled'}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Language:</span> 
              <span>{activeDocument.language}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Lines:</span> 
              <span>{activeDocument.content.split('\n').length}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Size:</span> 
              <span>{formatBytes(new Blob([activeDocument.content]).size)}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Modified:</span> 
              <span>Just now</span>
            </div>
          </div>
        </div>
      )}

      {activeResult && (
        <div style={{ marginBottom: '8px' }}>
          <h3 style={{ margin: '0 0 4px 0', fontSize: '13px' }}>Compilation Information</h3>
          <div style={{ backgroundColor: 'var(--bg-tertiary)', padding: '4px', borderRadius: '3px' }}>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Status:</span> 
              <span style={{ color: activeResult.errors.length > 0 ? 'var(--accent-error)' : 'var(--accent-primary)' }}>
                {activeResult.errors.length > 0 ? 'Errors found' : 'Success'}
              </span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Duration:</span> 
              <span>{formatDuration(activeResult.duration / 1000)}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Errors:</span> 
              <span>{activeResult.errors.length}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Warnings:</span> 
              <span>{activeResult.warnings.length}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Output Size:</span> 
              <span>{activeResult.data ? formatBytes(activeResult.data.length) : 'N/A'}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Compiled:</span> 
              <span>{activeResult.timestamp.toLocaleTimeString()}</span>
            </div>
          </div>
        </div>
      )}

      <div style={{ marginBottom: '8px' }}>
        <h3 style={{ margin: '0 0 4px 0', fontSize: '13px' }}>System Information</h3>
        <div style={{ backgroundColor: 'var(--bg-tertiary)', padding: '4px', borderRadius: '3px' }}>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>Browser:</span> 
            <span>{navigator.userAgent}</span>
          </div>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>Platform:</span> 
            <span>{navigator.platform}</span>
          </div>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>CPU Cores:</span> 
            <span>{navigator.hardwareConcurrency || 'Unknown'}</span>
          </div>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>Memory:</span> 
            <span>{(navigator as any).deviceMemory || 'Unknown'} GB</span>
          </div>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>WebAudio:</span> 
            <span style={{ color: typeof AudioContext !== 'undefined' ? '#4caf50' : '#ff5555' }}>
              {typeof AudioContext !== 'undefined' ? 'Supported' : 'Not supported'}
            </span>
          </div>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>WebMIDI:</span> 
            <span style={{ color: typeof (navigator as any).requestMIDIAccess === 'function' ? '#4caf50' : '#ff5555' }}>
              {typeof (navigator as any).requestMIDIAccess === 'function' ? 'Supported' : 'Not supported'}
            </span>
          </div>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>File API:</span> 
            <span style={{ color: typeof (window as any).showSaveFilePicker === 'function' ? '#4caf50' : '#ff5555' }}>
              {typeof (window as any).showSaveFilePicker === 'function' ? 'Supported' : 'Not supported'}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default InfoPanel;
