import React, { useState, useCallback, useRef, useEffect } from 'react';

interface MenuBarProps {
  onNewDocument: () => void;
  onOpenFile: () => void;
  onCloseDocument: () => void;
  onCloseAllDocuments: () => void;
  onToggleTheme: () => void;
  onCompile: () => void;
  onCompileAndPlay: () => void;
  onPlay: () => void;
  onStop: () => void;
  onLoadExample: (filename: string) => void;
  hasActiveDocument: boolean;
  hasMultipleDocuments: boolean;
  isCompiling: boolean;
  isPlaying: boolean;
}

// Menu definitions for keyboard navigation
const MENUS = ['file', 'edit', 'view', 'compile', 'play', 'tools', 'examples', 'help'] as const;
type MenuName = typeof MENUS[number];

// List of example files in public/samples/
const EXAMPLE_FILES = [
  { filename: 'hello_world.gwi', label: 'Hello World' },
  { filename: 'arpeggio.gwi', label: 'Arpeggio' },
  { filename: 'chord_progression.gwi', label: 'Chord Progression' },
  { filename: 'drum_pattern.gwi', label: 'Drum Pattern' },
  { filename: 'c140_test.gwi', label: 'C140 Test' },
  { filename: 'ay8910_test.gwi', label: 'AY-3-8910 Test' },
  { filename: 'general_test.gwi', label: 'General Test' },
  { filename: 'pcm_test.gwi', label: 'PCM Test' },
  { filename: 'pcm_test_2.gwi', label: 'PCM Test 2' },
  { filename: 'sega_pcm_test.gwi', label: 'Sega PCM Test' },
] as const;

const MenuBar: React.FC<MenuBarProps> = ({
  onNewDocument,
  onOpenFile,
  onCloseDocument,
  onCloseAllDocuments,
  onToggleTheme,
  onCompile,
  onCompileAndPlay,
  onPlay,
  onStop,
  onLoadExample,
  hasActiveDocument,
  hasMultipleDocuments,
  isCompiling,
  isPlaying,
}) => {
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [activeMenuItemIndex, setActiveMenuItemIndex] = useState<number>(0);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuButtonRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  // Close menu when clicking outside or pressing Escape
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setActiveMenu(null);
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      // Escape closes the active menu
      if (event.key === 'Escape') {
        setActiveMenu(null);
        return;
      }

      // If no menu is active, handle menu bar navigation
      if (!activeMenu) {
        if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
          event.preventDefault();
          const currentIndex = MENUS.findIndex(m => {
            const btn = menuButtonRefs.current.get(m);
            return btn && btn === document.activeElement;
          });
          
          let newIndex: number;
          if (event.key === 'ArrowRight') {
            newIndex = currentIndex < MENUS.length - 1 ? currentIndex + 1 : 0;
          } else {
            newIndex = currentIndex > 0 ? currentIndex - 1 : MENUS.length - 1;
          }
          
          menuButtonRefs.current.get(MENUS[newIndex])?.focus();
        } else if (event.key === 'ArrowDown') {
          event.preventDefault();
          // Open the menu under the focused button
          const focusedMenu = MENUS.find(m => {
            const btn = menuButtonRefs.current.get(m);
            return btn && btn === document.activeElement;
          });
          if (focusedMenu) {
            setActiveMenu(focusedMenu);
            setActiveMenuItemIndex(0);
          }
        } else if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          const focusedButton = document.activeElement as HTMLButtonElement;
          if (focusedButton && focusedButton.classList.contains('menu-item')) {
            focusedButton.click();
          }
        }
        return;
      }

      // Handle menu item navigation
      if (activeMenu) {
        const menuItems = getMenuItems(activeMenu);
        if (menuItems.length === 0) return;

        if (event.key === 'ArrowDown') {
          event.preventDefault();
          setActiveMenuItemIndex((prev) => (prev + 1) % menuItems.length);
        } else if (event.key === 'ArrowUp') {
          event.preventDefault();
          setActiveMenuItemIndex((prev) => (prev - 1 + menuItems.length) % menuItems.length);
        } else if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          const menuItems = getMenuItems(activeMenu);
          const item = menuItems[activeMenuItemIndex];
          if (item && !item.disabled && item.onClick) {
            item.onClick();
            setActiveMenu(null);
          }
        } else if (event.key === 'ArrowRight') {
          event.preventDefault();
          setActiveMenu(null);
          // Move to next menu button
          const currentIndex = MENUS.findIndex(m => m === activeMenu);
          const nextIndex = currentIndex < MENUS.length - 1 ? currentIndex + 1 : 0;
          menuButtonRefs.current.get(MENUS[nextIndex])?.focus();
        } else if (event.key === 'ArrowLeft') {
          event.preventDefault();
          setActiveMenu(null);
          // Move to previous menu button
          const currentIndex = MENUS.findIndex(m => m === activeMenu);
          const prevIndex = currentIndex > 0 ? currentIndex - 1 : MENUS.length - 1;
          menuButtonRefs.current.get(MENUS[prevIndex])?.focus();
        }
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [activeMenu, activeMenuItemIndex]);

  // Get menu items for a specific menu
  const getMenuItems = useCallback((menu: string) => {
    const items: Array<{ label: string; onClick?: () => void; disabled: boolean; key?: string }> = [];
    
    switch (menu) {
      case 'file':
        items.push({ label: 'New', onClick: onNewDocument, disabled: false });
        items.push({ label: 'Open...', onClick: onOpenFile, disabled: false });
        items.push({ label: 'Save', disabled: true });
        items.push({ label: 'Save As...', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Export...', disabled: true });
        items.push({ label: 'Import...', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Close', onClick: onCloseDocument, disabled: !hasActiveDocument });
        items.push({ label: 'Close All', onClick: onCloseAllDocuments, disabled: !hasMultipleDocuments });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Exit', disabled: true });
        break;
      case 'edit':
        items.push({ label: 'Undo', disabled: true });
        items.push({ label: 'Redo', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Cut', disabled: true });
        items.push({ label: 'Copy', disabled: true });
        items.push({ label: 'Paste', disabled: true });
        items.push({ label: 'Delete', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Select All', disabled: true });
        items.push({ label: 'Find...', disabled: true });
        items.push({ label: 'Replace...', disabled: true });
        break;
      case 'view':
        items.push({ label: 'Toggle Theme', onClick: onToggleTheme, disabled: false });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Zoom In', disabled: true });
        items.push({ label: 'Zoom Out', disabled: true });
        items.push({ label: 'Reset Zoom', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Show/Hide Panels', disabled: true });
        break;
      case 'compile':
        items.push({ label: 'Compile (F7)', onClick: onCompile, disabled: false });
        items.push({ label: 'Compile & Play (F5)', onClick: onCompileAndPlay, disabled: false, key: 'bold' });
        items.push({ label: 'Stop Compilation', onClick: () => {}, disabled: false });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Compile to VGM', disabled: true });
        items.push({ label: 'Compile to XGM', disabled: true });
        items.push({ label: 'Compile to ZGM', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Output Format Settings...', disabled: true });
        items.push({ label: 'Compile Options...', disabled: true });
        break;
      case 'play':
        items.push({ label: 'Play (F5)', onClick: onCompileAndPlay, disabled: false, key: 'bold' });
        items.push({ label: 'Stop', onClick: onStop, disabled: !isPlaying });
        items.push({ label: 'Pause', onClick: onPlay, disabled: !isPlaying });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Play from Start', disabled: true });
        items.push({ label: 'Play Selection', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Playback Settings...', disabled: true });
        items.push({ label: 'Audio Settings...', disabled: true });
        break;
      case 'tools':
        items.push({ label: 'Part Counter', disabled: true });
        items.push({ label: 'Error List', disabled: true });
        items.push({ label: 'Folder Tree', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'MIDI Settings...', disabled: true });
        items.push({ label: 'Key Bindings...', disabled: true });
        items.push({ label: 'Preferences...', disabled: true });
        break;
      case 'examples':
        EXAMPLE_FILES.forEach((example) => {
          items.push({ label: example.label, onClick: () => onLoadExample(example.filename), disabled: false });
        });
        break;
      case 'help':
        items.push({ label: 'Help Topics', disabled: true });
        items.push({ label: 'MML Reference', disabled: true });
        items.push({ label: 'About...', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Check for Updates', disabled: true });
        break;
    }
    return items;
  }, [onNewDocument, onToggleTheme, onCompile, onCompileAndPlay, onPlay, onStop, isPlaying, onLoadExample]);

  const toggleMenu = useCallback((menu: string) => {
    setActiveMenu((prev) => (prev === menu ? null : menu));
    if (activeMenu !== menu) {
      setActiveMenuItemIndex(0);
    }
  }, [activeMenu]);

  const closeMenu = useCallback(() => {
    setActiveMenu(null);
  }, []);

  const handleMenuItemClick = useCallback((action: () => void) => {
    action();
    closeMenu();
  }, [closeMenu]);

  // Render a single menu
  const renderMenu = (menu: MenuName, label: string) => {
    const items = getMenuItems(menu);
    const isOpen = activeMenu === menu;
    const hasItems = items.length > 0;
    
    return (
      <div className="dropdown" key={menu}>
        <button
          ref={(el) => { if (el) menuButtonRefs.current.set(menu, el); }}
          className="menu-item"
          onClick={() => toggleMenu(menu)}
          onFocus={() => { if (!isOpen) setActiveMenu(null); }}
          aria-expanded={isOpen}
          aria-haspopup="true"
          aria-label={label}
        >
          {label}
        </button>
        {isOpen && hasItems && (
          <div
            className="dropdown-menu show"
            role="menu"
            aria-label={label}
          >
            {items.map((item, index) => {
              if (item.label === 'Separator') {
                return <div className="context-menu-separator" role="separator" key={`sep-${menu}-${index}`} />;
              }
              
              const isActive = activeMenu === menu && activeMenuItemIndex === index;
              const isDisabled = item.disabled || !item.onClick;
              
              return (
                <div
                  key={`${menu}-${index}`}
                  className={`dropdown-item ${isDisabled ? 'disabled' : ''} ${isActive ? 'active' : ''}`}
                  onClick={() => { if (!isDisabled && item.onClick) handleMenuItemClick(item.onClick); }}
                  onFocus={() => { if (activeMenu === menu) setActiveMenuItemIndex(index); }}
                  role="menuitem"
                  aria-disabled={isDisabled}
                  aria-label={item.label}
                  tabIndex={isActive ? 0 : -1}
                  ref={(el) => { if (isActive && el) el.focus(); }}
                  style={item.key === 'bold' ? { fontWeight: 'bold' } : undefined}
                >
                  {item.label}
                </div>
              );
            })}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="menu-bar" ref={menuRef} role="menubar" aria-label="Main menu">
      {/* Render all menus */}
      {renderMenu('file', 'File')}
      {renderMenu('edit', 'Edit')}
      {renderMenu('view', 'View')}
      {renderMenu('compile', 'Compile')}
      {renderMenu('play', 'Play')}
      {renderMenu('tools', 'Tools')}
      {renderMenu('examples', 'Examples')}
      {renderMenu('help', 'Help')}

      {/* Quick Actions */}
      <div style={{ flex: 1 }} />
      
      {/* Compile Button */}
      <button
        className="menu-item"
        onClick={onCompile}
        disabled={isCompiling}
        title="Compile (F7)"
        aria-label="Compile"
      >
        {isCompiling ? 'Compiling...' : 'Compile'}
      </button>
    </div>
  );
};

export default MenuBar;
