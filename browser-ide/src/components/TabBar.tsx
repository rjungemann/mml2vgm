import React from 'react';
import { Document } from '@/types';

interface TabBarProps {
  documents: Document[];
  activeDocumentId: string;
  onSelectTab: (id: string) => void;
  onCloseTab: (id: string) => void;
}

export const TabBar: React.FC<TabBarProps> = ({
  documents,
  activeDocumentId,
  onSelectTab,
  onCloseTab,
}) => {
  return (
    <div className="tab-bar">
      {documents.map((doc) => {
        const isActive = doc.id === activeDocumentId;
        const isDirty = doc.isDirty;
        
        return (
          <button
            key={doc.id}
            className={`tab ${isActive ? 'active' : ''}`}
            onClick={(e) => {
              // Check if click was on close button
              if ((e.target as HTMLElement).classList.contains('close-button')) {
                onCloseTab(doc.id);
              } else {
                onSelectTab(doc.id);
              }
            }}
          >
            <span>{doc.filename || 'Untitled'}</span>
            {isDirty && <span style={{ color: 'var(--accent-warning)' }}> *</span>}
            <span
              className="close-button"
              onClick={(e) => {
                e.stopPropagation();
                onCloseTab(doc.id);
              }}
            >
              x
            </span>
          </button>
        );
      })}
    </div>
  );
};

export default TabBar;
