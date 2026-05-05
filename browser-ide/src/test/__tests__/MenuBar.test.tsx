/**
 * MenuBar Component Tests
 * 
 * Tests for keyboard navigation and accessibility.
 * Part of Phase 7: Polish and Optimization.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import MenuBar from '@/components/MenuBar';

// Mock component props
const mockProps = {
    onNewDocument: vi.fn(),
    onToggleTheme: vi.fn(),
    onCompile: vi.fn(),
    onCompileAndPlay: vi.fn(),
    onPlay: vi.fn(),
    onStop: vi.fn(),
    isCompiling: false,
    isPlaying: false,
};

describe('MenuBar', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    describe('Rendering', () => {
        it('should render all menu buttons', () => {
            render(<MenuBar {...mockProps} />);
            
            expect(screen.getByText('File')).toBeInTheDocument();
            expect(screen.getByText('Edit')).toBeInTheDocument();
            expect(screen.getByText('View')).toBeInTheDocument();
            expect(screen.getByText('Compile')).toBeInTheDocument();
            expect(screen.getByText('Play')).toBeInTheDocument();
            expect(screen.getByText('Tools')).toBeInTheDocument();
            expect(screen.getByText('Help')).toBeInTheDocument();
        });

        it('should render compile button', () => {
            render(<MenuBar {...mockProps} />);
            
            expect(screen.getByText('Compile')).toBeInTheDocument();
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

        it('should call onCompile when clicking Compile button', () => {
            render(<MenuBar {...mockProps} />);
            
            const compileButton = screen.getByText('Compile');
            fireEvent.click(compileButton);
            
            expect(mockProps.onCompile).toHaveBeenCalledTimes(1);
        });

        it('should call onToggleTheme when clicking Toggle Theme', () => {
            render(<MenuBar {...mockProps} />);
            
            const viewButton = screen.getByText('View');
            fireEvent.click(viewButton);
            
            const toggleThemeItem = screen.getByText('Toggle Theme');
            fireEvent.click(toggleThemeItem);
            
            expect(mockProps.onToggleTheme).toHaveBeenCalledTimes(1);
        });
    });

    describe('Disabled Items', () => {
        it('should show disabled state on menu items', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            // Open... should be disabled
            const openItem = screen.getByText('Open...');
            expect(openItem).toHaveClass('disabled');
        });

        it('should not call action for disabled items', () => {
            render(<MenuBar {...mockProps} />);
            
            const fileButton = screen.getByText('File');
            fireEvent.click(fileButton);
            
            // Save should be disabled
            const saveItem = screen.getByText('Save');
            fireEvent.click(saveItem);
            
            // No action should be called
            expect(mockProps.onNewDocument).not.toHaveBeenCalled();
        });
    });

    describe('Play Menu', () => {
        it('should enable Stop and Pause when playing', () => {
            render(<MenuBar {...mockProps} isPlaying={true} />);
            
            const playButton = screen.getByText('Play');
            fireEvent.click(playButton);
            
            // Stop and Pause should be enabled
            expect(screen.getByText('Stop')).not.toHaveClass('disabled');
            expect(screen.getByText('Pause')).not.toHaveClass('disabled');
        });

        it('should disable Stop and Pause when not playing', () => {
            render(<MenuBar {...mockProps} isPlaying={false} />);
            
            const playButton = screen.getByText('Play');
            fireEvent.click(playButton);
            
            // Stop and Pause should be disabled
            expect(screen.getByText('Stop')).toHaveClass('disabled');
            expect(screen.getByText('Pause')).toHaveClass('disabled');
        });
    });
});
