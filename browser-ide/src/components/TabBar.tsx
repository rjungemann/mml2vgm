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
            <span className={`tab-filename ${isDirty ? 'dirty' : ''}`}>{doc.filename || 'Untitled'}</span>
            <span
              className="close-button"
              aria-label={`Close ${doc.filename || 'Untitled'}`}
              onClick={(e) => {
                e.stopPropagation();
                onCloseTab(doc.id);
              }}
            >
              &times;
            </span>
          </button>
        );
      })}
    </div>
  );
};

export default TabBar;
