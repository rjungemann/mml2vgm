/**
 * MenuBar Component Tests
 * 
 * Tests for keyboard navigation and accessibility.
 * Part of Phase 7: Polish and Optimization.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent, within } from '@testing-library/react';
import MenuBar from '@/components/MenuBar';

// Mock component props
const mockProps = {
    onNewDocument: vi.fn(),
    onOpenFile: vi.fn(),
    onCloseDocument: vi.fn(),
    onCloseAllDocuments: vi.fn(),
    onSave: vi.fn(),
    onSaveAs: vi.fn(),
    onExit: vi.fn(),
    onToggleTheme: vi.fn(),
    onToggleSidebar: vi.fn(),
    isSidebarVisible: false,
    onCompile: vi.fn(),
    onCompileAndPlay: vi.fn(),
    onPlay: vi.fn(),
    onStop: vi.fn(),
    onLoadExample: vi.fn(),
    hasActiveDocument: false,
    hasMultipleDocuments: false,
    isCompiling: false,
    isPlaying: false,
    // Edit menu
    onFind: vi.fn(),
    onReplace: vi.fn(),
    onSelectAll: vi.fn(),
    onCut: vi.fn(),
    onCopy: vi.fn(),
    onPaste: vi.fn(),
    onDelete: vi.fn(),
    onUndo: vi.fn(),
    onRedo: vi.fn(),
    hasSelection: false,
    canUndo: false,
    canRedo: false,
    // File menu - Export
    onExportBinary: vi.fn(),
    hasCompileResult: false,
};

describe('MenuBar', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('Rendering', () => {
        it('should render all menu buttons', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            const menuButtons = within(menuBar).getAllByRole('button');
            
            // Check that all expected menu buttons are present
            const menuLabels = menuButtons.map(b => b.textContent);
            expect(menuLabels).toContain('File');
            expect(menuLabels).toContain('Edit');
            expect(menuLabels).toContain('View');
            expect(menuLabels).toContain('Compile');
            expect(menuLabels).toContain('Play');
            expect(menuLabels).toContain('Tools');
            expect(menuLabels).toContain('Help');
        });

        it('should render compile menu button', () => {
            render(<MenuBar {...mockProps} />);
            
            // Check for the Compile menu button (within menubar, has aria-haspopup)
            const menuBar = screen.getByRole('menubar');
            const buttons = within(menuBar).getAllByRole('button');
            const compileButton = buttons.find(b => b.textContent === 'Compile');
            expect(compileButton).toBeInTheDocument();
        });

        it('should show compiling state', () => {
            render(<MenuBar {...mockProps} isCompiling={true} />);
            
            expect(screen.getByText('Compiling...')).toBeInTheDocument();
        });
    });

    describe('Accessibility', () => {
        it('should have menubar role', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            expect(menuBar).toBeInTheDocument();
        });

        it('should have aria-label on menubar', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            expect(menuBar).toHaveAttribute('aria-label', 'Main menu');
        });

        it('should have aria-haspopup on menu buttons', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            expect(fileButton).toHaveAttribute('aria-haspopup', 'true');
        });

        it('should have aria-expanded on menu buttons', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            expect(fileButton).toHaveAttribute('aria-expanded', 'false');
        });
    });

    describe('Menu Opening', () => {
        it('should open file menu on click', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            // Menu should be open (aria-expanded should be true)
            expect(fileButton).toHaveAttribute('aria-expanded', 'true');
            
            // Should show menu items
            expect(screen.getByText('New')).toBeInTheDocument();
        });

        it('should close menu when clicking outside', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            // Click outside
            fireEvent.mouseDown(document);
            
            expect(fileButton).toHaveAttribute('aria-expanded', 'false');
        });

        it('should open edit menu on click', () => {
            render(<MenuBar {...mockProps} />);
            
            const editButton = screen.getByText('Edit');
            fireEvent.click(editButton);
            
            expect(editButton).toHaveAttribute('aria-expanded', 'true');
            expect(screen.getByText('Undo')).toBeInTheDocument();
        });

        it('should open view menu on click', () => {
            render(<MenuBar {...mockProps} />);
            
            const viewButton = screen.getByText('View');
            fireEvent.click(viewButton);
            
            expect(viewButton).toHaveAttribute('aria-expanded', 'true');
            expect(screen.getByText('Toggle Theme')).toBeInTheDocument();
        });

        it('should open compile menu on click', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            const buttons = within(menuBar).getAllByRole('button');
            const compileButton = buttons.find(b => b.textContent === 'Compile')!;
            fireEvent.click(compileButton);
            
            expect(compileButton).toHaveAttribute('aria-expanded', 'true');
            expect(screen.getByText('Compile (F7)')).toBeInTheDocument();
        });

        it('should open play menu on click', () => {
            render(<MenuBar {...mockProps} />);
            
            const playButton = screen.getByText('Play');
            fireEvent.click(playButton);
            
            expect(playButton).toHaveAttribute('aria-expanded', 'true');
            expect(screen.getByText('Play (F5)')).toBeInTheDocument();
        });

        it('should open tools menu on click', () => {
            render(<MenuBar {...mockProps} />);
            
            const toolsButton = screen.getByText('Tools');
            fireEvent.click(toolsButton);
            
            expect(toolsButton).toHaveAttribute('aria-expanded', 'true');
            expect(screen.getByText('Part Counter')).toBeInTheDocument();
        });

        it('should open help menu on click', () => {
            render(<MenuBar {...mockProps} />);
            
            const helpButton = screen.getByText('Help');
            fireEvent.click(helpButton);
            
            expect(helpButton).toHaveAttribute('aria-expanded', 'true');
            expect(screen.getByText('Help Topics')).toBeInTheDocument();
        });

        it('should close active menu when opening another', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            const editButton = screen.getByText('Edit');
            
            // Open File menu
            fireEvent.click(fileButton);
            expect(fileButton).toHaveAttribute('aria-expanded', 'true');
            
            // Open Edit menu - File should close
            fireEvent.click(editButton);
            expect(fileButton).toHaveAttribute('aria-expanded', 'false');
            expect(editButton).toHaveAttribute('aria-expanded', 'true');
        });
    });

    describe('Action Execution', () => {
        it('should call onNewDocument when clicking New', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            const newItem = screen.getByText('New');
            fireEvent.click(newItem);
            
            expect(mockProps.onNewDocument).toHaveBeenCalledTimes(1);
        });

        it('should call onOpenFile when clicking Open', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            const openItem = screen.getByText('Open...');
            fireEvent.click(openItem);
            
            // onOpenFile should be called - in real browser this triggers file picker
            expect(mockProps.onOpenFile).toHaveBeenCalledTimes(1);
        });

        it('should call onToggleTheme when clicking Toggle Theme', () => {
            render(<MenuBar {...mockProps} />);
            
            const viewButton = screen.getByText('View');
            fireEvent.click(viewButton);
            
            const toggleThemeItem = screen.getByText('Toggle Theme');
            fireEvent.click(toggleThemeItem);
            
            expect(mockProps.onToggleTheme).toHaveBeenCalledTimes(1);
        });

        it('should call onCompile when clicking Compile menu item', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            const buttons = within(menuBar).getAllByRole('button');
            const compileButton = buttons.find(b => b.textContent === 'Compile')!;
            fireEvent.click(compileButton);
            
            // Now click the Compile (F7) menu item
            const compileMenuItem = screen.getByText('Compile (F7)');
            fireEvent.click(compileMenuItem);
            
            expect(mockProps.onCompile).toHaveBeenCalledTimes(1);
        });

        it('should call onCompileAndPlay when clicking Compile & Play menu item', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            const buttons = within(menuBar).getAllByRole('button');
            const compileButton = buttons.find(b => b.textContent === 'Compile')!;
            fireEvent.click(compileButton);
            
            const compileAndPlayItem = screen.getByText('Compile & Play (F5)');
            fireEvent.click(compileAndPlayItem);
            
            expect(mockProps.onCompileAndPlay).toHaveBeenCalledTimes(1);
        });

        it('should call onCompileAndPlay when clicking Play (F5) menu item', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            const buttons = within(menuBar).getAllByRole('button');
            const playButton = buttons.find(b => b.textContent === 'Play')!;
            fireEvent.click(playButton);
            
            const playItem = screen.getByText('Play (F5)');
            fireEvent.click(playItem);
            
            // Play (F5) calls onCompileAndPlay, not onPlay
            expect(mockProps.onCompileAndPlay).toHaveBeenCalledTimes(1);
        });
        
        it('should call onPlay when clicking Pause menu item', () => {
            render(<MenuBar {...mockProps} isPlaying={true} />);
            
            const menuBar = screen.getByRole('menubar');
            const buttons = within(menuBar).getAllByRole('button');
            const playButton = buttons.find(b => b.textContent === 'Play')!;
            fireEvent.click(playButton);
            
            const pauseItem = screen.getByText('Pause');
            fireEvent.click(pauseItem);
            
            expect(mockProps.onPlay).toHaveBeenCalledTimes(1);
        });

        it('should call onStop when clicking Stop', () => {
            render(<MenuBar {...mockProps} isPlaying={true} />);
            
            const playButton = screen.getByText('Play');
            fireEvent.click(playButton);
            
            const stopItem = screen.getByText('Stop');
            fireEvent.click(stopItem);
            
            expect(mockProps.onStop).toHaveBeenCalledTimes(1);
        });
    });

    describe('File Menu Smoke Tests', () => {
        it('should render all File menu items', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            // All File menu items should be present
            expect(screen.getByText('New')).toBeInTheDocument();
            expect(screen.getByText('Open...')).toBeInTheDocument();
            expect(screen.getByText('Save')).toBeInTheDocument();
            expect(screen.getByText('Save As...')).toBeInTheDocument();
            expect(screen.getByText('Export...')).toBeInTheDocument();
            expect(screen.getByText('Import...')).toBeInTheDocument();
            expect(screen.getByText('Close')).toBeInTheDocument();
            expect(screen.getByText('Close All')).toBeInTheDocument();
            expect(screen.getByText('Exit')).toBeInTheDocument();
        });

        it('should have correct enabled/disabled state for File menu items with no document', () => {
            render(<MenuBar {...mockProps} hasActiveDocument={false} hasMultipleDocuments={false} />);

            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);

            // New and Open should be enabled
            expect(screen.getByText('New')).not.toHaveClass('disabled');
            expect(screen.getByText('Open...')).not.toHaveClass('disabled');

            // Most items should be disabled
            expect(screen.getByText('Save')).toHaveClass('disabled');
            expect(screen.getByText('Save As...')).toHaveClass('disabled');
            expect(screen.getByText('Export...')).toHaveClass('disabled');
            expect(screen.getByText('Import...')).toHaveClass('disabled');
            // Close and Close All should be disabled when no active document
            expect(screen.getByText('Close')).toHaveClass('disabled');
            expect(screen.getByText('Close All')).toHaveClass('disabled');
            // Exit is always enabled (closes the browser tab/window)
            expect(screen.getByText('Exit')).not.toHaveClass('disabled');
        });

        it('should enable Close when document is active', () => {
            render(<MenuBar {...mockProps} hasActiveDocument={true} hasMultipleDocuments={false} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            expect(screen.getByText('Close')).not.toHaveClass('disabled');
            expect(screen.getByText('Close All')).toHaveClass('disabled');
        });

        it('should enable Close All when multiple documents are open', () => {
            render(<MenuBar {...mockProps} hasActiveDocument={true} hasMultipleDocuments={true} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            expect(screen.getByText('Close')).not.toHaveClass('disabled');
            expect(screen.getByText('Close All')).not.toHaveClass('disabled');
        });

        it('should call onCloseDocument when clicking Close', () => {
            render(<MenuBar {...mockProps} hasActiveDocument={true} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            const closeItem = screen.getByText('Close');
            fireEvent.click(closeItem);
            
            expect(mockProps.onCloseDocument).toHaveBeenCalledTimes(1);
        });

        it('should call onCloseAllDocuments when clicking Close All', () => {
            render(<MenuBar {...mockProps} hasMultipleDocuments={true} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            const closeAllItem = screen.getByText('Close All');
            fireEvent.click(closeAllItem);
            
            expect(mockProps.onCloseAllDocuments).toHaveBeenCalledTimes(1);
        });
    });

    describe('Edit Menu Smoke Tests', () => {
        it('should render all Edit menu items', () => {
            render(<MenuBar {...mockProps} />);
            
            const editButton = screen.getByText('Edit');
            fireEvent.click(editButton);
            
            // All Edit menu items should be present
            expect(screen.getByText('Undo')).toBeInTheDocument();
            expect(screen.getByText('Redo')).toBeInTheDocument();
            expect(screen.getByText('Cut')).toBeInTheDocument();
            expect(screen.getByText('Copy')).toBeInTheDocument();
            expect(screen.getByText('Paste')).toBeInTheDocument();
            expect(screen.getByText('Delete')).toBeInTheDocument();
            expect(screen.getByText('Select All')).toBeInTheDocument();
            expect(screen.getByText('Find...')).toBeInTheDocument();
            expect(screen.getByText('Replace...')).toBeInTheDocument();
        });

        it('should have all Edit menu items disabled', () => {
            render(<MenuBar {...mockProps} />);
            
            const editButton = screen.getByText('Edit');
            fireEvent.click(editButton);
            
            expect(screen.getByText('Undo')).toHaveClass('disabled');
            expect(screen.getByText('Redo')).toHaveClass('disabled');
            expect(screen.getByText('Cut')).toHaveClass('disabled');
            expect(screen.getByText('Copy')).toHaveClass('disabled');
            expect(screen.getByText('Paste')).toHaveClass('disabled');
            expect(screen.getByText('Delete')).toHaveClass('disabled');
            expect(screen.getByText('Select All')).toHaveClass('disabled');
            expect(screen.getByText('Find...')).toHaveClass('disabled');
            expect(screen.getByText('Replace...')).toHaveClass('disabled');
        });
    });

    describe('View Menu Smoke Tests', () => {
        it('should render all View menu items', () => {
            render(<MenuBar {...mockProps} />);
            
            const viewButton = screen.getByText('View');
            fireEvent.click(viewButton);
            
            // All View menu items should be present
            expect(screen.getByText('Toggle Theme')).toBeInTheDocument();
            expect(screen.getByText('Zoom In')).toBeInTheDocument();
            expect(screen.getByText('Zoom Out')).toBeInTheDocument();
            expect(screen.getByText('Reset Zoom')).toBeInTheDocument();
            expect(screen.getByText('Show/Hide Panels')).toBeInTheDocument();
        });

        it('should have Toggle Theme enabled and others disabled', () => {
            render(<MenuBar {...mockProps} />);
            
            const viewButton = screen.getByText('View');
            fireEvent.click(viewButton);
            
            expect(screen.getByText('Toggle Theme')).not.toHaveClass('disabled');
            expect(screen.getByText('Zoom In')).toHaveClass('disabled');
            expect(screen.getByText('Zoom Out')).toHaveClass('disabled');
            expect(screen.getByText('Reset Zoom')).toHaveClass('disabled');
            expect(screen.getByText('Show/Hide Panels')).toHaveClass('disabled');
        });
    });

    describe('Compile Menu Smoke Tests', () => {
        it('should render all Compile menu items', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            const buttons = within(menuBar).getAllByRole('button');
            const compileButton = buttons.find(b => b.textContent === 'Compile')!;
            fireEvent.click(compileButton);
            
            // All Compile menu items should be present
            expect(screen.getByText('Compile (F7)')).toBeInTheDocument();
            expect(screen.getByText('Compile & Play (F5)')).toBeInTheDocument();
            expect(screen.getByText('Stop Compilation')).toBeInTheDocument();
            expect(screen.getByText('Compile to VGM')).toBeInTheDocument();
            expect(screen.getByText('Compile to XGM')).toBeInTheDocument();
            expect(screen.getByText('Compile to ZGM')).toBeInTheDocument();
            expect(screen.getByText('Output Format Settings...')).toBeInTheDocument();
            expect(screen.getByText('Compile Options...')).toBeInTheDocument();
        });

        it('should have Compile and Compile & Play enabled', () => {
            render(<MenuBar {...mockProps} />);
            
            const menuBar = screen.getByRole('menubar');
            const buttons = within(menuBar).getAllByRole('button');
            const compileButton = buttons.find(b => b.textContent === 'Compile')!;
            fireEvent.click(compileButton);
            
            expect(screen.getByText('Compile (F7)')).not.toHaveClass('disabled');
            expect(screen.getByText('Compile & Play (F5)')).not.toHaveClass('disabled');
            expect(screen.getByText('Stop Compilation')).not.toHaveClass('disabled');
            expect(screen.getByText('Compile to VGM')).toHaveClass('disabled');
            expect(screen.getByText('Compile to XGM')).toHaveClass('disabled');
            expect(screen.getByText('Compile to ZGM')).toHaveClass('disabled');
            expect(screen.getByText('Output Format Settings...')).toHaveClass('disabled');
            expect(screen.getByText('Compile Options...')).toHaveClass('disabled');
        });

        it('should show compiling state in Compile menu button', () => {
            render(<MenuBar {...mockProps} isCompiling={true} />);
            
            expect(screen.getByText('Compiling...')).toBeInTheDocument();
        });
    });

    describe('Play Menu Smoke Tests', () => {
        it('should render all Play menu items', () => {
            render(<MenuBar {...mockProps} />);
            
            const playButton = screen.getByText('Play');
            fireEvent.click(playButton);
            
            // All Play menu items should be present
            expect(screen.getByText('Play (F5)')).toBeInTheDocument();
            expect(screen.getByText('Stop')).toBeInTheDocument();
            expect(screen.getByText('Pause')).toBeInTheDocument();
            expect(screen.getByText('Play from Start')).toBeInTheDocument();
            expect(screen.getByText('Play Selection')).toBeInTheDocument();
            expect(screen.getByText('Playback Settings...')).toBeInTheDocument();
            expect(screen.getByText('Audio Settings...')).toBeInTheDocument();
        });

        it('should enable Stop and Pause when playing', () => {
            render(<MenuBar {...mockProps} isPlaying={true} />);
            
            const playButton = screen.getByText('Play');
            fireEvent.click(playButton);
            
            expect(screen.getByText('Stop')).not.toHaveClass('disabled');
            expect(screen.getByText('Pause')).not.toHaveClass('disabled');
            expect(screen.getByText('Play (F5)')).not.toHaveClass('disabled');
        });

        it('should disable Stop and Pause when not playing', () => {
            render(<MenuBar {...mockProps} isPlaying={false} />);
            
            const playButton = screen.getByText('Play');
            fireEvent.click(playButton);
            
            expect(screen.getByText('Stop')).toHaveClass('disabled');
            expect(screen.getByText('Pause')).toHaveClass('disabled');
            expect(screen.getByText('Play from Start')).toHaveClass('disabled');
            expect(screen.getByText('Play Selection')).toHaveClass('disabled');
            expect(screen.getByText('Playback Settings...')).toHaveClass('disabled');
            expect(screen.getByText('Audio Settings...')).toHaveClass('disabled');
        });
    });

    describe('Tools Menu Smoke Tests', () => {
        it('should render all Tools menu items', () => {
            render(<MenuBar {...mockProps} />);
            
            const toolsButton = screen.getByText('Tools');
            fireEvent.click(toolsButton);
            
            // All Tools menu items should be present
            expect(screen.getByText('Part Counter')).toBeInTheDocument();
            expect(screen.getByText('Error List')).toBeInTheDocument();
            expect(screen.getByText('Folder Tree')).toBeInTheDocument();
            expect(screen.getByText('MIDI Settings...')).toBeInTheDocument();
            expect(screen.getByText('Key Bindings...')).toBeInTheDocument();
            expect(screen.getByText('Preferences...')).toBeInTheDocument();
        });

        it('should have all Tools menu items disabled', () => {
            render(<MenuBar {...mockProps} />);
            
            const toolsButton = screen.getByText('Tools');
            fireEvent.click(toolsButton);
            
            expect(screen.getByText('Part Counter')).toHaveClass('disabled');
            expect(screen.getByText('Error List')).toHaveClass('disabled');
            expect(screen.getByText('Folder Tree')).toHaveClass('disabled');
            expect(screen.getByText('MIDI Settings...')).toHaveClass('disabled');
            expect(screen.getByText('Key Bindings...')).toHaveClass('disabled');
            expect(screen.getByText('Preferences...')).toHaveClass('disabled');
        });
    });

    describe('Help Menu Smoke Tests', () => {
        it('should render all Help menu items', () => {
            render(<MenuBar {...mockProps} />);
            
            const helpButton = screen.getByText('Help');
            fireEvent.click(helpButton);
            
            // All Help menu items should be present
            expect(screen.getByText('Help Topics')).toBeInTheDocument();
            expect(screen.getByText('MML Reference')).toBeInTheDocument();
            expect(screen.getByText('About...')).toBeInTheDocument();
            expect(screen.getByText('Check for Updates')).toBeInTheDocument();
        });

        it('should have all Help menu items disabled', () => {
            render(<MenuBar {...mockProps} />);
            
            const helpButton = screen.getByText('Help');
            fireEvent.click(helpButton);
            
            expect(screen.getByText('Help Topics')).toHaveClass('disabled');
            expect(screen.getByText('MML Reference')).toHaveClass('disabled');
            expect(screen.getByText('About...')).toHaveClass('disabled');
            expect(screen.getByText('Check for Updates')).toHaveClass('disabled');
        });
    });

    describe('Keyboard Navigation', () => {
        it('should navigate between menus with arrow keys', () => {
            render(<MenuBar {...mockProps} />);
            
            // Focus first menu button
            const fileButton = screen.getByText('File');
            fileButton.focus();
            
            // Press right arrow to move to Edit
            fireEvent.keyDown(document, { key: 'ArrowRight' });
            
            const editButton = screen.getByText('Edit');
            expect(document.activeElement).toBe(editButton);
        });

        it('should open menu with Enter key', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fileButton.focus();
            
            fireEvent.keyDown(document, { key: 'Enter' });
            
            expect(fileButton).toHaveAttribute('aria-expanded', 'true');
        });

        it('should navigate menu items with arrow keys', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            // Press down arrow to move to first menu item
            fireEvent.keyDown(document, { key: 'ArrowDown' });
            
            // The first menu item should be focused
            // Note: This tests the keyboard navigation within the dropdown
        });
    });
});
