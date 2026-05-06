import React from 'react';
import type { Document, CompileStatus, MMLLanguage } from '@/types';

interface StatusBarProps {
  document: Document | null;
  compileStatus: CompileStatus;
  progress: number;
  progressMessage?: string;
  lastCompileTimingSummary?: string;
  isAudioPlaying?: boolean;
  onToggleRuntimeDebug?: () => void;
  runtimeDebugVisible?: boolean;
}

const StatusBar: React.FC<StatusBarProps> = ({
  document,
  compileStatus,
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
      <div className="flex items-center" style={{ gap: '16px' }}>
        {/* Document info */}
        {document && (
          <>
            <div className="status-bar-item">
              <span>{document.filename || 'Untitled'}</span>
            </div>
            
            <div className="status-bar-item">
              <span>{getLanguageDisplayName(document.language)}</span>
            </div>
          </>
        )}

        {/* Encoding */}
        <div className="status-bar-item">
          <span>{encoding}</span>
        </div>

        {/* Line/Column */}
        <div className="status-bar-item">
          <span>{cursorPos}</span>
        </div>
        
        {/* File size and stats */}
        {document && (
          <>
            <div className="status-bar-item">
              <span>{formatFileSize(document.content)}</span>
            </div>

            <div className="status-bar-item">
              <span>{formatLineColumnCount(document.content)}</span>
            </div>
          </>
        )}
      </div>
    </div>
  );
};

export default StatusBar;
