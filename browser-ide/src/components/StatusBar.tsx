import React, { useEffect, useRef } from 'react';
import type { Document, CompileStatus, MMLLanguage } from '@/types';

interface StatusBarProps {
  document: Document | null;
  compileStatus: CompileStatus;
  progress: number;
  progressMessage?: string;
  lastCompileTimingSummary?: string;
  waveformSamples?: number[];
  isAudioPlaying?: boolean;
  onToggleRuntimeDebug?: () => void;
  runtimeDebugVisible?: boolean;
}

const StatusBar: React.FC<StatusBarProps> = ({
  document,
  compileStatus,
  progress,
  progressMessage,
  lastCompileTimingSummary,
  waveformSamples = [],
  isAudioPlaying = false,
  onToggleRuntimeDebug,
  runtimeDebugVisible = false,
}) => {
  const waveformCanvasRef = useRef<HTMLCanvasElement | null>(null);

  // Format file size
  const formatFileSize = (content: string): string => {
    const bytes = new Blob([content]).size;
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  // Get language display name
  const getLanguageDisplayName = (lang: MMLLanguage): string => {
    const names: Record<MMLLanguage, string> = {
      gwi: 'GWI',
      mml: 'MML',
      muc: 'MUC',
      mdl: 'MDL',
      mus: 'MUS',
    };
    return names[lang] || lang;
  };

  // Get status text
  const getStatusText = (): string => {
    const statuses: Record<CompileStatus, string> = {
      idle: 'Ready',
      queued: 'Queued',
      compiling: 'Compiling...',
      success: 'Compilation Complete',
      error: 'Compilation Error',
    };
    return statuses[compileStatus] || 'Ready';
  };

  const normalizedProgress = progress <= 1 ? progress * 100 : progress;

  // Format line and column count
  const formatLineColumnCount = (content: string): string => {
    const lines = content.split('\n');
    const totalLines = lines.length;
    const totalChars = content.length;
    return `Lines: ${totalLines}, Chars: ${totalChars}`;
  };

  // Get encoding display
  const encoding = document?.encoding || 'UTF-8';

  // Get cursor position (would be provided by editor in real implementation)
  const cursorPos = 'Ln 1, Col 1';

  useEffect(() => {
    const canvas = waveformCanvasRef.current;
    if (!canvas) return;

    const cssWidth = 96;
    const cssHeight = 16;
    const dpr = window.devicePixelRatio || 1;

    canvas.width = Math.floor(cssWidth * dpr);
    canvas.height = Math.floor(cssHeight * dpr);

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, cssWidth, cssHeight);

    ctx.fillStyle = 'var(--bg-secondary)';
    ctx.fillRect(0, 0, cssWidth, cssHeight);

    const centerY = cssHeight / 2;
    ctx.strokeStyle = 'var(--border-color)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, centerY);
    ctx.lineTo(cssWidth, centerY);
    ctx.stroke();

    if (!waveformSamples.length) {
      return;
    }

    ctx.strokeStyle = isAudioPlaying ? 'var(--accent-primary)' : 'var(--text-muted)';
    ctx.lineWidth = 1;
    ctx.beginPath();

    const step = waveformSamples.length > 1
      ? cssWidth / (waveformSamples.length - 1)
      : cssWidth;
    const amplitude = cssHeight * 0.42;

    for (let i = 0; i < waveformSamples.length; i++) {
      const x = i * step;
      const clamped = Math.max(-1, Math.min(1, waveformSamples[i]));
      const y = centerY - clamped * amplitude;
      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }
    ctx.stroke();
  }, [waveformSamples, isAudioPlaying]);

  return (
    <div className="status-bar">
      {/* Left side */}
      <div className="flex items-center gap-16">
        {/* Document info */}
        {document && (
          <div className="status-bar-item">
            <span>{document.filename || 'Untitled'}</span>
            <span className="text-muted">|</span>
            <span>{getLanguageDisplayName(document.language)}</span>
          </div>
        )}

        {/* Encoding */}
        <div className="status-bar-item">
          <span>{encoding}</span>
        </div>

        {/* Line/Column */}
        <div className="status-bar-item">
          <span>{cursorPos}</span>
        </div>
      </div>

      {/* Right side */}
      <div className="flex items-center gap-16">
        <button
          type="button"
          className="status-bar-item status-bar-waveform status-bar-waveform-button"
          onClick={() => onToggleRuntimeDebug?.()}
          title="Toggle Runtime Debug panel"
          aria-label="Toggle Runtime Debug panel"
          aria-pressed={runtimeDebugVisible}
        >
          <span>Wave</span>
          <canvas
            ref={waveformCanvasRef}
            className="status-bar-waveform-canvas"
            aria-label="Status bar waveform"
          />
        </button>

        {/* Compile status */}
        <div 
          className={`status-bar-item ${
            compileStatus === 'error' ? 'error' :
            compileStatus === 'success' ? 'success' : ''
          }`}
        >
          <span>{getStatusText()}</span>
          {compileStatus === 'compiling' && progress > 0 && (
            <span> ({Math.round(normalizedProgress)}%)</span>
          )}
          {progressMessage && compileStatus === 'compiling' && (
            <span>: {progressMessage}</span>
          )}
        </div>

        {compileStatus !== 'compiling' && lastCompileTimingSummary && (
          <div className="status-bar-item">
            <span>Last compile: {lastCompileTimingSummary}</span>
          </div>
        )}

        {/* File size and stats */}
        {document && (
          <div className="status-bar-item">
            <span>{formatFileSize(document.content)}</span>
            <span className="text-muted" style={{ margin: '0 4px' }}>|</span>
            <span>{formatLineColumnCount(document.content)}</span>
          </div>
        )}

        {/* Insert/Overwrite mode */}
        <div className="status-bar-item">
          <span>INSERT</span>
        </div>
      </div>
    </div>
  );
};

export default StatusBar;
