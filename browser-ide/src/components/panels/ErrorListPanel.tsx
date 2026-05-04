import React, { useCallback } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useCompileStore } from '@/stores/compileStore';
import { useDocumentStore } from '@/stores/documentStore';
import type { CompileError, Position } from '@/types';

// Type for error list item
interface ErrorListItem {
  type: 'error' | 'warning' | 'info';
  message: string;
  line: number;
  column: number;
  length: number;
  severity: 'error' | 'warning' | 'info';
  code?: string;
}

interface ErrorListPanelProps {
  // Optional callback for navigation
  onNavigateToPosition?: (position: Position) => void;
}

const ErrorListPanel: React.FC<ErrorListPanelProps> = ({ onNavigateToPosition }) => {
  const { getResult } = useCompileStore(
    useShallow((state) => ({
      getResult: state.getResult,
    }))
  );

  const { activeDocumentId } = useDocumentStore(
    useShallow((state) => ({
      activeDocumentId: state.activeDocumentId,
    }))
  );

  // Get errors for active document from compile store
  const activeResult = activeDocumentId ? getResult(activeDocumentId) : null;
  
  // Combine errors and warnings from compile result
  const allIssues: ErrorListItem[] = [];

  if (activeResult) {
    // Add errors
    activeResult.errors.forEach((e: CompileError) => {
      allIssues.push({ 
        type: e.type as 'error' | 'warning' | 'info',
        message: e.message,
        line: e.line,
        column: e.column,
        length: e.length,
        severity: e.severity,
        code: e.code,
      });
    });
    
    // Add warnings
    activeResult.warnings.forEach((w: CompileError) => {
      allIssues.push({ 
        type: w.type as 'error' | 'warning' | 'info',
        message: w.message,
        line: w.line,
        column: w.column,
        length: w.length,
        severity: w.severity,
        code: w.code,
      });
    });
  }

  // Sort by line, then by column
  allIssues.sort((a, b) => a.line - b.line || a.column - b.column);

  // Get error count
  const errorCount = allIssues.filter(i => i.severity === 'error').length;
  const warningCount = allIssues.filter(i => i.severity === 'warning').length;

  // Handle click on error to navigate to line
  const handleErrorClick = useCallback((line: number, column: number) => {
    if (onNavigateToPosition) {
      onNavigateToPosition({ line, column });
    }
  }, [onNavigateToPosition]);

  return (
    <div className="error-list">
      {/* Header with counts */}
      <div 
        style={{
          padding: '4px 8px',
          fontSize: '11px',
          color: 'var(--text-muted)',
          borderBottom: '1px solid var(--border-color)',
        }}
      >
        {errorCount} error{errorCount !== 1 ? 's' : ''}, {warningCount} warning{warningCount !== 1 ? 's' : ''}
      </div>
      
      {allIssues.length === 0 ? (
        <div style={{ padding: '8px', color: 'var(--text-muted)', fontSize: '12px' }}>
          No errors or warnings
        </div>
      ) : (
        <div style={{ maxHeight: '300px', overflowY: 'auto' }}>
          {allIssues.map((issue, index) => (
            <div
              key={`${index}-${issue.line}-${issue.column}`}
              className={`error-item ${issue.severity}`}
              onClick={() => handleErrorClick(issue.line, issue.column)}
              title={`Click to navigate to line ${issue.line}, column ${issue.column}`}
            >
              <span className="severity">
                {issue.severity === 'error' ? '✗' : issue.severity === 'warning' ? '⚠' : 'ℹ'}
              </span>
              <span className="message" style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                {issue.message}
              </span>
              <span className="location">
                {issue.line}:{issue.column}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default ErrorListPanel;
