import React from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useCompileStore } from '@/stores/compileStore';
import { useDocumentStore } from '@/stores/documentStore';

const ErrorListPanel: React.FC = () => {
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

  // Get errors for active document
  const activeResult = activeDocumentId ? getResult(activeDocumentId) : null;
  
  // Combine errors and warnings
  const allIssues: { type: 'error' | 'warning'; message: string; line: number; column: number }[] = [];
  
  if (activeResult) {
    activeResult.errors.forEach((e: any) => {
      allIssues.push({ type: 'error', message: e.message, line: e.line, column: e.column });
    });
    activeResult.warnings.forEach((w: any) => {
      allIssues.push({ type: 'warning', message: w.message, line: w.line, column: w.column });
    });
  }

  // Sort by line, then by column
  allIssues.sort((a, b) => a.line - b.line || a.column - b.column);

  // Handle click on error to navigate to line
  const handleErrorClick = (line: number, column: number) => {
    console.log(`Navigate to line ${line}, column ${column}`);
  };

  return (
    <div className="error-list">
      {allIssues.length === 0 ? (
        <div style={{ padding: '8px', color: 'var(--text-muted)', fontSize: '12px' }}>
          No errors or warnings
        </div>
      ) : (
        allIssues.map((issue, index) => (
          <div
            key={`${index}-${issue.line}-${issue.column}`}
            className={`error-item ${issue.type}`}
            onClick={() => handleErrorClick(issue.line, issue.column)}
          >
            <span className="severity">
              {issue.type === 'error' ? 'X' : '!'}
            </span>
            <span className="message">{issue.message}</span>
            <span className="location">
              Line {issue.line}, Col {issue.column}
            </span>
          </div>
        ))
      )}
    </div>
  );
};

export default ErrorListPanel;
