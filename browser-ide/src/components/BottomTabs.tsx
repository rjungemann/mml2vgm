import React, { ReactNode } from 'react';

export interface BottomTab {
  id: string;
  label: string;
  content: ReactNode;
}

interface BottomTabsProps {
  tabs: BottomTab[];
  activeTabId: string;
  onTabClick: (tabId: string) => void;
  isMinimized: boolean;
  onMinimize: () => void;
  onMaximize: () => void;
}

const BottomTabs: React.FC<BottomTabsProps> = ({
  tabs,
  activeTabId,
  onTabClick,
  isMinimized,
  onMinimize,
  onMaximize,
}) => {
  const activeTab = tabs.find((t) => t.id === activeTabId);

  const handleTabClick = (tabId: string) => {
    if (tabId === activeTabId && !isMinimized) {
      // Clicking the active tab minimizes the pane
      onMinimize();
    } else if (isMinimized) {
      // Clicking any tab on minimized pane expands it
      onMaximize();
    }
    // Switch to the tab
    onTabClick(tabId);
  };

  return (
    <div className={`bottom-tabs-container ${isMinimized ? 'minimized' : ''}`}>
      {/* Tab bar */}
      <div className="bottom-tabs-header">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`bottom-tab ${tab.id === activeTabId ? 'active' : ''}`}
            onClick={() => handleTabClick(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      {!isMinimized && activeTab && (
        <div className="bottom-tabs-content">
          {activeTab.content}
        </div>
      )}
    </div>
  );
};

export default BottomTabs;
