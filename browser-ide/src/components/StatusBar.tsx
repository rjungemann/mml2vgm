import React from 'react';
import type { Document, CompileStatus, MMLLanguage } from '@/types';

interface StatusBarProps {
  document: Document | null;
  compileStatus: CompileStatus;
  progress: number;
  progressMessage?: string;
}

const StatusBar: React.FC<StatusBarProps> = ({
  document,
  compileStatus,
  progress,
  progressMessage,
}) => {
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
        {/* Compile status */}
        <div 
          className={`status-bar-item ${
            compileStatus === 'error' ? 'error' :
            compileStatus === 'success' ? 'success' : ''
          }`}
        >
          <span>{getStatusText()}</span>
          {compileStatus === 'compiling' && progress > 0 && (
            <span> ({Math.round(progress * 100)}%)</span>
          )}
          {progressMessage && compileStatus === 'compiling' && (
            <span>: {progressMessage}</span>
          )}
        </div>

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
