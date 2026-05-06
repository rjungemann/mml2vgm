import React, { useEffect, useState } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useCompileStore } from '@/stores/compileStore';
import { useDocumentStore } from '@/stores/documentStore';

const CompilationPanel: React.FC = () => {
  const [compileStartedAt, setCompileStartedAt] = useState<number | null>(null);
  const [elapsedMs, setElapsedMs] = useState(0);

  const { getResult, status, progress, progressMessage, lastCompileTimingSummary } = useCompileStore(
    useShallow((state) => ({
      getResult: state.getResult,
      status: state.status,
      progress: state.progress,
      progressMessage: state.progressMessage,
      lastCompileTimingSummary: state.lastCompileTimingSummary,
    }))
  );

  const { activeDocumentId } = useDocumentStore(
    useShallow((state) => ({
      activeDocumentId: state.activeDocumentId,
    }))
  );

  // Get active compile result
  const activeResult = activeDocumentId ? getResult(activeDocumentId) : null;

  useEffect(() => {
    if (status !== 'compiling') {
      setCompileStartedAt(null);
      setElapsedMs(0);
      return;
    }

    const startAt = compileStartedAt ?? Date.now();
    if (!compileStartedAt) {
      setCompileStartedAt(startAt);
    }

    const interval = window.setInterval(() => {
      setElapsedMs(Date.now() - startAt);
    }, 200);

    return () => window.clearInterval(interval);
  }, [status, compileStartedAt]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%', padding: '4px', fontSize: '12px' }}>
      <div style={{ marginBottom: '8px' }}>
        <h3 style={{ margin: '0 0 4px 0', fontSize: '13px' }}>Compile Status</h3>
        <div style={{ backgroundColor: 'var(--bg-tertiary)', padding: '4px', borderRadius: '3px' }}>
          <div style={{ marginBottom: '2px' }}>
            <span style={{ color: 'var(--text-muted)' }}>Status:</span>{' '}
            <span>{status}</span>
            {status === 'compiling' && typeof progress === 'number' && (
              <span>{` (${Math.round(progress)}%)`}</span>
            )}
          </div>
          {status === 'compiling' && (
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Elapsed:</span>{' '}
              <span>{(elapsedMs / 1000).toFixed(1)}s</span>
            </div>
          )}
          {progressMessage && (
            <div style={{ marginBottom: '2px', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
              <span style={{ color: 'var(--text-muted)' }}>Phase:</span>{' '}
              <span>{progressMessage}</span>
            </div>
          )}
          {lastCompileTimingSummary && status !== 'compiling' && (
            <div style={{ marginBottom: '2px', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
              <span style={{ color: 'var(--text-muted)' }}>Last timing:</span>{' '}
              <span>{lastCompileTimingSummary}</span>
            </div>
          )}
        </div>
      </div>

      {activeResult && (
        <div style={{ marginBottom: '8px' }}>
          <h3 style={{ margin: '0 0 4px 0', fontSize: '13px' }}>Output</h3>
          <div style={{ backgroundColor: 'var(--bg-tertiary)', padding: '4px', borderRadius: '3px' }}>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Output bytes:</span>{' '}
              <span>{activeResult.data?.length || 0}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Parts:</span>{' '}
              <span>{activeResult.partCount || 0}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Commands:</span>{' '}
              <span>{activeResult.commandCount || 0}</span>
            </div>
            <div style={{ marginBottom: '2px' }}>
              <span style={{ color: 'var(--text-muted)' }}>Chips used:</span>{' '}
              <span>{(activeResult.chipsUsed || []).join(', ') || '(none)'}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default CompilationPanel;
