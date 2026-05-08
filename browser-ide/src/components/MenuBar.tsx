import React, { useState, useCallback, useRef, useEffect } from 'react';

interface MenuBarProps {
  onNewDocument: () => void;
  onOpenFile: () => void;
  onCloseDocument: () => void;
  onCloseAllDocuments: () => void;
  onToggleTheme: () => void;
  onToggleSidebar?: () => void;
  isSidebarVisible?: boolean;
  onCompile: () => void;
  onCompileAndPlay: () => void;
  onPlay: () => void;
  onStop: () => void;
  onLoadExample: (filename: string) => void;
  hasActiveDocument: boolean;
  hasMultipleDocuments: boolean;
  isCompiling: boolean;
  isPlaying: boolean;
  // File menu - Save
  onSave: () => void;
  onSaveAs: () => void;
  // File menu - Exit
  onExit: () => void;
  // File menu - Export
  onExportBinary: (format: string) => void;
  hasCompileResult: boolean;
  // Edit menu
  onFind: () => void;
  onReplace: () => void;
  onSelectAll: () => void;
  onCut: () => void;
  onCopy: () => void;
  onPaste: () => void;
  onDelete: () => void;
  onUndo: () => void;
  onRedo: () => void;
  hasSelection: boolean;
  canUndo: boolean;
  canRedo: boolean;
  // Phase 3.1: Zoom controls
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onZoomReset?: () => void;
  fontSize?: number;
  // Phase 5.2: Playback speed and loop
  onSetPlaybackRate?: (rate: number) => void;
  playbackRate?: number;
  onToggleLoop?: () => void;
  isLooping?: boolean;
  // Phase 4.1: Output format selection
  onSetOutputFormat?: (format: string) => void;
  outputFormat?: string;
  // Phase 4.2 / 3.2: Panel access and visibility
  onShowPanel?: (panel: string) => void;
  onTogglePanel?: (panel: string) => void;
  panelVisibility?: Record<string, boolean>;
  // Phase 4.3: Advanced compile options dialog
  onOpenAdvancedCompileOptions?: () => void;
  // Phase 5.3: Audio settings dialog
  onOpenAudioSettings?: () => void;
  // Phase 6.4: MIDI settings dialog
  onOpenMidiSettings?: () => void;
  // WebHID: HID MIDI controller settings dialog (experimental)
  onOpenHIDSettings?: () => void;
  // WebSerial: Hardware serial settings dialog (experimental)
  onOpenSerialSettings?: () => void;
  // Phase 6.5: Key bindings dialog
  onOpenKeyBindings?: () => void;
  // Phase 6.6: Preferences dialog
  onOpenPreferences?: () => void;
  // Phase 7.1: Help topics dialog
  onOpenHelp?: () => void;
  // Phase 7.2: MML reference dialog
  onOpenMmlReference?: () => void;
  // Phase 7.3: About dialog
  onOpenAbout?: () => void;
  // Sample library upload
  onUploadSamples?: () => void;
}

// Menu definitions for keyboard navigation
const MENUS = ['file', 'edit', 'view', 'compile', 'play', 'tools', 'instruments', 'examples', 'help'] as const;
type MenuName = typeof MENUS[number];

// List of example files in public/samples/
const EXAMPLE_FILES = [
  // ── Beginner ───────────────────────────────────────────────────────────────
  { filename: 'hello_world.gwi', label: 'Hello World' },
  { filename: '01_fm_basics.gwi', label: 'FM Basics' },
  { filename: '02_psg_basics.gwi', label: 'PSG Basics' },
  { filename: '03_notes_and_lengths.gwi', label: 'Notes and Lengths' },
  { filename: '04_octaves_and_volumes.gwi', label: 'Octaves and Volumes' },
  { filename: '05_loops.gwi', label: 'Loops' },
  // ── Intermediate ───────────────────────────────────────────────────────────
  { filename: 'arpeggio.gwi', label: 'Arpeggio' },
  { filename: 'chord_progression.gwi', label: 'Chord Progression' },
  { filename: 'drum_pattern.gwi', label: 'Drum Pattern' },
  { filename: 'ay8910_test.gwi', label: 'AY-3-8910 Test' },
  { filename: '10_fm_algorithms.gwi', label: 'FM Algorithms (0-7)' },
  { filename: '11_fm_adsr.gwi', label: 'FM Envelope (ADSR)' },
  { filename: '12_quantize.gwi', label: 'Quantize / Gate Time' },
  { filename: '13_fm_feedback.gwi', label: 'FM Feedback' },
  { filename: '14_fm_psg_combo.gwi', label: 'FM + PSG Combo' },
  { filename: '15_tempo_changes.gwi', label: 'Tempo Changes' },
  { filename: '16_song_structure.gwi', label: 'Song Structure' },
  { filename: '17_ym2203_opn.gwi', label: 'YM2203 (OPN)' },
  { filename: '18_ym2151_opm.gwi', label: 'YM2151 (OPM)' },
  { filename: '19_ym3812_opl2.gwi', label: 'YM3812 (OPL2)' },
  { filename: '20_psg_extended.gwi', label: 'PSG Extended' },
  // ── Advanced ───────────────────────────────────────────────────────────────
  { filename: '30_ym2608_opna.gwi', label: 'YM2608 (OPNA)' },
  { filename: '31_ymf262_opl3.gwi', label: 'YMF262 (OPL3)' },
  { filename: '35_ensemble.gwi', label: 'Full Ensemble' },
  // ── Test / Reference ───────────────────────────────────────────────────────
  { filename: 'c140_test.gwi', label: 'C140 Test' },
  { filename: 'general_test.gwi', label: 'General Test' },
  { filename: 'pcm_test.gwi', label: 'PCM Test' },
  { filename: 'pcm_test_2.gwi', label: 'PCM Test 2' },
  { filename: 'sega_pcm_test.gwi', label: 'Sega PCM Test' },
] as const;

// Panels available in the View → Panels submenu
const VIEW_PANELS: Array<{ key: string; label: string }> = [
  { key: 'errorList', label: 'Error List' },
  { key: 'runtime', label: 'Runtime' },
  { key: 'compilation', label: 'Compilation' },
  { key: 'waveform', label: 'Waveform' },
  { key: 'compileOptions', label: 'Compile Options' },
  { key: 'folderTree', label: 'Folder Tree' },
  { key: 'partCounter', label: 'Part Counter' },
];

const MenuBar: React.FC<MenuBarProps> = ({
  onNewDocument,
  onOpenFile,
  onCloseDocument,
  onCloseAllDocuments,
  onSave,
  onSaveAs,
  onExit,
  onExportBinary,
  hasCompileResult,
  onToggleTheme,
  onToggleSidebar,
  isSidebarVisible,
  onCompile,
  onCompileAndPlay,
  onPlay,
  onStop,
  onLoadExample,
  hasActiveDocument,
  hasMultipleDocuments,
  isCompiling,
  isPlaying,
  // Edit menu
  onFind,
  onReplace,
  onSelectAll,
  onCut,
  onCopy,
  onPaste,
  onDelete,
  onUndo,
  onRedo,
  hasSelection,
  canUndo,
  canRedo,
  // Phase 3.1
  onZoomIn,
  onZoomOut,
  onZoomReset,
  // fontSize is available for future zoom-percentage display in labels
  // Phase 5.2
  onSetPlaybackRate,
  playbackRate,
  onToggleLoop,
  isLooping,
  // Phase 4.1
  onSetOutputFormat,
  outputFormat,
  // Phase 4.2 / 3.2
  onShowPanel,
  onTogglePanel,
  panelVisibility,
  // Dialogs
  onOpenAdvancedCompileOptions,
  onOpenAudioSettings,
  onOpenMidiSettings,
  onOpenHIDSettings,
  onOpenSerialSettings,
  onOpenKeyBindings,
  onOpenPreferences,
  onOpenHelp,
  onOpenMmlReference,
  onOpenAbout,
  onUploadSamples,
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
    const items: Array<{
      label: string;
      onClick?: () => void;
      disabled: boolean;
      key?: string;
      checked?: boolean;
    }> = [];

    switch (menu) {
      case 'file':
        items.push({ label: 'New', onClick: onNewDocument, disabled: false });
        items.push({ label: 'Open...', onClick: onOpenFile, disabled: false });
        items.push({ label: 'Save', onClick: onSave, disabled: !hasActiveDocument });
        items.push({ label: 'Save As...', onClick: onSaveAs, disabled: !hasActiveDocument });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Export...', disabled: true });
        items.push({ label: 'Export as VGM', onClick: () => onExportBinary('vgm'), disabled: !hasActiveDocument || !hasCompileResult });
        items.push({ label: 'Export as XGM', onClick: () => onExportBinary('xgm'), disabled: !hasActiveDocument || !hasCompileResult });
        items.push({ label: 'Export as ZGM', onClick: () => onExportBinary('zgm'), disabled: !hasActiveDocument || !hasCompileResult });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Import...', disabled: true });
        items.push({ label: 'Upload Samples…', onClick: onUploadSamples, disabled: !onUploadSamples });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Close', onClick: onCloseDocument, disabled: !hasActiveDocument });
        items.push({ label: 'Close All', onClick: onCloseAllDocuments, disabled: !hasMultipleDocuments });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Exit', onClick: onExit, disabled: false });
        break;
      case 'edit':
        items.push({ label: 'Undo', onClick: onUndo, disabled: !hasActiveDocument || !canUndo });
        items.push({ label: 'Redo', onClick: onRedo, disabled: !hasActiveDocument || !canRedo });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Cut', onClick: onCut, disabled: !hasActiveDocument || !hasSelection });
        items.push({ label: 'Copy', onClick: onCopy, disabled: !hasActiveDocument || !hasSelection });
        items.push({ label: 'Paste', onClick: onPaste, disabled: !hasActiveDocument });
        items.push({ label: 'Delete', onClick: onDelete, disabled: !hasActiveDocument || !hasSelection });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Select All', onClick: onSelectAll, disabled: !hasActiveDocument });
        items.push({ label: 'Find...', onClick: onFind, disabled: !hasActiveDocument });
        items.push({ label: 'Replace...', onClick: onReplace, disabled: !hasActiveDocument });
        break;
      case 'view':
        items.push({ label: 'Toggle Theme', onClick: onToggleTheme, disabled: false });
        items.push({
          label: isSidebarVisible ? 'Hide Sidebar' : 'Show Sidebar',
          onClick: onToggleSidebar,
          disabled: !onToggleSidebar,
        });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Zoom In', onClick: onZoomIn, disabled: !onZoomIn });
        items.push({ label: 'Zoom Out', onClick: onZoomOut, disabled: !onZoomOut });
        items.push({ label: 'Reset Zoom', onClick: onZoomReset, disabled: !onZoomReset });
        items.push({ label: 'Separator', disabled: true });
        if (onTogglePanel && panelVisibility) {
          VIEW_PANELS.forEach(p => {
            items.push({
              label: p.label,
              onClick: () => onTogglePanel(p.key),
              disabled: false,
              checked: !!panelVisibility[p.key],
            });
          });
        } else {
          items.push({ label: 'Show/Hide Panels', disabled: true });
        }
        break;
      case 'compile':
        items.push({ label: 'Compile (F7)', onClick: onCompile, disabled: false });
        items.push({ label: 'Compile & Play (F5)', onClick: onCompileAndPlay, disabled: false, key: 'bold' });
        items.push({ label: 'Stop Compilation', onClick: () => {}, disabled: false });
        items.push({ label: 'Separator', disabled: true });
        items.push({
          label: 'Compile to VGM',
          onClick: onSetOutputFormat ? () => onSetOutputFormat('vgm') : undefined,
          disabled: !onSetOutputFormat,
          checked: outputFormat === 'vgm',
        });
        items.push({
          label: 'Compile to XGM',
          onClick: onSetOutputFormat ? () => onSetOutputFormat('xgm') : undefined,
          disabled: !onSetOutputFormat,
          checked: outputFormat === 'xgm',
        });
        items.push({
          label: 'Compile to XGM2',
          onClick: onSetOutputFormat ? () => onSetOutputFormat('xgm2') : undefined,
          disabled: !onSetOutputFormat,
          checked: outputFormat === 'xgm2',
        });
        items.push({
          label: 'Compile to ZGM',
          onClick: onSetOutputFormat ? () => onSetOutputFormat('zgm') : undefined,
          disabled: !onSetOutputFormat,
          checked: outputFormat === 'zgm',
        });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Output Format Settings...', disabled: true });
        items.push({
          label: 'Compile Options...',
          onClick: onShowPanel ? () => onShowPanel('compileOptions') : undefined,
          disabled: !onShowPanel,
        });
        items.push({
          label: 'Advanced Options...',
          onClick: onOpenAdvancedCompileOptions,
          disabled: !onOpenAdvancedCompileOptions,
        });
        break;
      case 'play':
        items.push({ label: 'Play (F5)', onClick: onCompileAndPlay, disabled: false, key: 'bold' });
        items.push({ label: 'Stop', onClick: onStop, disabled: !isPlaying });
        items.push({ label: 'Pause', onClick: onPlay, disabled: !isPlaying });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Play from Start', disabled: true });
        items.push({ label: 'Play Selection', disabled: true });
        items.push({ label: 'Separator', disabled: true });
        // Playback speed
        if (onSetPlaybackRate) {
          const rates = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0];
          rates.forEach(rate => {
            items.push({
              label: `Speed ${Math.round(rate * 100)}%`,
              onClick: () => onSetPlaybackRate(rate),
              disabled: false,
              checked: playbackRate === rate,
            });
          });
          items.push({ label: 'Separator', disabled: true });
        }
        // Loop toggle
        items.push({
          label: 'Loop',
          onClick: onToggleLoop,
          disabled: !onToggleLoop,
          checked: !!isLooping,
        });
        items.push({ label: 'Separator', disabled: true });
        items.push({ label: 'Playback Settings...', disabled: true });
        items.push({
          label: 'Audio Settings...',
          onClick: onOpenAudioSettings,
          disabled: !onOpenAudioSettings,
        });
        break;
      case 'tools':
        items.push({
          label: 'Part Counter',
          onClick: onShowPanel ? () => onShowPanel('partCounter') : undefined,
          disabled: !onShowPanel,
        });
        items.push({
          label: 'Error List',
          onClick: onShowPanel ? () => onShowPanel('errorList') : undefined,
          disabled: !onShowPanel,
        });
        items.push({
          label: 'Folder Tree',
          onClick: onShowPanel ? () => onShowPanel('folderTree') : undefined,
          disabled: !onShowPanel,
        });
        items.push({ label: 'Separator', disabled: true });
        items.push({
          label: 'MIDI Settings...',
          onClick: onOpenMidiSettings,
          disabled: !onOpenMidiSettings,
        });
        items.push({
          label: 'HID MIDI Controller... (Experimental)',
          onClick: onOpenHIDSettings,
          disabled: !onOpenHIDSettings,
        });
        items.push({
          label: 'Hardware Serial... (Experimental)',
          onClick: onOpenSerialSettings,
          disabled: !onOpenSerialSettings,
        });
        items.push({
          label: 'Key Bindings...',
          onClick: onOpenKeyBindings,
          disabled: !onOpenKeyBindings,
        });
        items.push({
          label: 'Preferences...',
          onClick: onOpenPreferences,
          disabled: !onOpenPreferences,
        });
        break;
      case 'instruments':
        items.push({
          label: 'FM Tone Editor',
          onClick: onShowPanel ? () => onShowPanel('fmToneEditor') : undefined,
          disabled: !onShowPanel,
        });
        items.push({
          label: 'Envelope Editor',
          onClick: onShowPanel ? () => onShowPanel('envelopeEditor') : undefined,
          disabled: !onShowPanel,
        });
        items.push({
          label: 'Arpeggio Editor',
          onClick: onShowPanel ? () => onShowPanel('arpeggioEditor') : undefined,
          disabled: !onShowPanel,
        });
        break;
      case 'examples':
        EXAMPLE_FILES.forEach((example) => {
          items.push({ label: example.label, onClick: () => onLoadExample(example.filename), disabled: false });
        });
        break;
      case 'help':
        items.push({ label: 'Help Topics', onClick: onOpenHelp, disabled: !onOpenHelp });
        items.push({ label: 'MML Reference', onClick: onOpenMmlReference, disabled: !onOpenMmlReference });
        items.push({ label: 'About...', onClick: onOpenAbout, disabled: !onOpenAbout });
        items.push({ label: 'Check for Updates', disabled: true });
        break;
    }
    return items;
  }, [
    onNewDocument,
    onSave,
    onSaveAs,
    onExit,
    onToggleTheme,
    onToggleSidebar,
    isSidebarVisible,
    onCompile,
    onCompileAndPlay,
    onPlay,
    onStop,
    isPlaying,
    onLoadExample,
    onFind,
    onReplace,
    onSelectAll,
    onCut,
    onCopy,
    onPaste,
    onDelete,
    onUndo,
    onRedo,
    onExportBinary,
    hasActiveDocument,
    hasMultipleDocuments,
    hasSelection,
    canUndo,
    canRedo,
    hasCompileResult,
    onZoomIn,
    onZoomOut,
    onZoomReset,
    onSetOutputFormat,
    outputFormat,
    onShowPanel,
    onTogglePanel,
    panelVisibility,
    onSetPlaybackRate,
    playbackRate,
    onToggleLoop,
    isLooping,
    onOpenAdvancedCompileOptions,
    onOpenAudioSettings,
    onOpenMidiSettings,
    onOpenHIDSettings,
    onOpenSerialSettings,
    onOpenKeyBindings,
    onOpenPreferences,
    onOpenHelp,
    onOpenMmlReference,
    onOpenAbout,
  ]);

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
                  <span style={{ width: '1em', display: 'inline-block', flexShrink: 0 }}>
                    {item.checked ? '✓' : ''}
                  </span>
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
      {renderMenu('instruments', 'Instruments')}
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
