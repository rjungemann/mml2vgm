import React, { useState, useCallback, useRef, useEffect } from 'react';

interface MenuBarProps {
  onNewDocument: () => void;
  onToggleTheme: () => void;
  onCompile: () => void;
  onCompileAndPlay: () => void;
  onPlay: () => void;
  onStop: () => void;
  isCompiling: boolean;
  isPlaying: boolean;
}

const MenuBar: React.FC<MenuBarProps> = ({
  onNewDocument,
  onToggleTheme,
  onCompile,
  onCompileAndPlay,
  onPlay,
  onStop,
  isCompiling,
  isPlaying,
}) => {
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setActiveMenu(null);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const toggleMenu = useCallback((menu: string) => {
    setActiveMenu((prev) => (prev === menu ? null : menu));
  }, []);

  const closeMenu = useCallback(() => {
    setActiveMenu(null);
  }, []);

  const handleMenuItemClick = useCallback((action: () => void) => {
    action();
    closeMenu();
  }, [closeMenu]);

  return (
    <div className="menu-bar" ref={menuRef}>
      {/* File Menu */}
      <div className="dropdown">
        <button className="menu-item" onClick={() => toggleMenu('file')}>
          File
        </button>
        {activeMenu === 'file' && (
          <div className="dropdown-menu show">
            <div 
              className="dropdown-item" 
              onClick={() => handleMenuItemClick(onNewDocument)}
            >
              New
            </div>
            <div className="dropdown-item disabled">Open...</div>
            <div className="dropdown-item disabled">Save</div>
            <div className="dropdown-item disabled">Save As...</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Export...</div>
            <div className="dropdown-item disabled">Import...</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Close</div>
            <div className="dropdown-item disabled">Close All</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Exit</div>
          </div>
        )}
      </div>

      {/* Edit Menu */}
      <div className="dropdown">
        <button className="menu-item" onClick={() => toggleMenu('edit')}>
          Edit
        </button>
        {activeMenu === 'edit' && (
          <div className="dropdown-menu show">
            <div className="dropdown-item disabled">Undo</div>
            <div className="dropdown-item disabled">Redo</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Cut</div>
            <div className="dropdown-item disabled">Copy</div>
            <div className="dropdown-item disabled">Paste</div>
            <div className="dropdown-item disabled">Delete</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Select All</div>
            <div className="dropdown-item disabled">Find...</div>
            <div className="dropdown-item disabled">Replace...</div>
          </div>
        )}
      </div>

      {/* View Menu */}
      <div className="dropdown">
        <button className="menu-item" onClick={() => toggleMenu('view')}>
          View
        </button>
        {activeMenu === 'view' && (
          <div className="dropdown-menu show">
            <div className="dropdown-item" onClick={() => handleMenuItemClick(onToggleTheme)}>
              Toggle Theme
            </div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Zoom In</div>
            <div className="dropdown-item disabled">Zoom Out</div>
            <div className="dropdown-item disabled">Reset Zoom</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Show/Hide Panels</div>
          </div>
        )}
      </div>

      {/* Compile Menu */}
      <div className="dropdown">
        <button className="menu-item" onClick={() => toggleMenu('compile')}>
          Compile
        </button>
        {activeMenu === 'compile' && (
          <div className="dropdown-menu show">
            <div 
              className="dropdown-item" 
              onClick={() => handleMenuItemClick(onCompile)}
            >
              Compile (F7)
            </div>
            <div 
              className="dropdown-item" 
              onClick={() => handleMenuItemClick(onCompileAndPlay)}
              style={{ fontWeight: 'bold' }}
            >
              Compile & Play (F5)
            </div>
            <div className="dropdown-item" onClick={() => handleMenuItemClick(() => {})}>
              Stop Compilation
            </div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Compile to VGM</div>
            <div className="dropdown-item disabled">Compile to XGM</div>
            <div className="dropdown-item disabled">Compile to ZGM</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Output Format Settings...</div>
            <div className="dropdown-item disabled">Compile Options...</div>
          </div>
        )}
      </div>

      {/* Play Menu */}
      <div className="dropdown">
        <button className="menu-item" onClick={() => toggleMenu('play')}>
          Play
        </button>
        {activeMenu === 'play' && (
          <div className="dropdown-menu show">
            <div 
              className="dropdown-item" 
              onClick={() => handleMenuItemClick(onCompileAndPlay)}
              style={{ fontWeight: 'bold' }}
            >
              Play (F5)
            </div>
            <div 
              className="dropdown-item" 
              onClick={() => handleMenuItemClick(onStop)}
              disabled={!isPlaying}
            >
              Stop
            </div>
            <div 
              className="dropdown-item" 
              onClick={() => handleMenuItemClick(onPlay)}
              disabled={!isPlaying}
            >
              Pause
            </div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Play from Start</div>
            <div className="dropdown-item disabled">Play Selection</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Playback Settings...</div>
            <div className="dropdown-item disabled">Audio Settings...</div>
          </div>
        )}
      </div>

      {/* Tools Menu */}
      <div className="dropdown">
        <button className="menu-item" onClick={() => toggleMenu('tools')}>
          Tools
        </button>
        {activeMenu === 'tools' && (
          <div className="dropdown-menu show">
            <div className="dropdown-item disabled">Part Counter</div>
            <div className="dropdown-item disabled">Error List</div>
            <div className="dropdown-item disabled">Folder Tree</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">MIDI Settings...</div>
            <div className="dropdown-item disabled">Key Bindings...</div>
            <div className="dropdown-item disabled">Preferences...</div>
          </div>
        )}
      </div>

      {/* Help Menu */}
      <div className="dropdown">
        <button className="menu-item" onClick={() => toggleMenu('help')}>
          Help
        </button>
        {activeMenu === 'help' && (
          <div className="dropdown-menu show">
            <div className="dropdown-item disabled">Help Topics</div>
            <div className="dropdown-item disabled">MML Reference</div>
            <div className="dropdown-item disabled">About...</div>
            <div className="context-menu-separator" />
            <div className="dropdown-item disabled">Check for Updates</div>
          </div>
        )}
      </div>

      {/* Quick Actions */}
      <div style={{ flex: 1 }} />
      
      {/* Compile Button */}
      <button
        className="menu-item"
        onClick={onCompile}
        disabled={isCompiling}
        title="Compile (F7)"
      >
        {isCompiling ? 'Compiling...' : 'Compile'}
      </button>
    </div>
  );
};

export default MenuBar;
