/**
 * File Service
 * 
 * Manages file system operations using the Browser File System Access API.
 * Provides file open/save operations and workspace management.
 */

// ============================================================================
// Types
// ============================================================================

/** File system node for the folder tree */
export interface TreeNode {
    id: string;
    name: string;
    path: string;
    type: 'file' | 'directory';
    size?: number;
    lastModified?: number;
    children?: TreeNode[];
    handle?: FileSystemDirectoryHandle | FileSystemFileHandle;
}

/** Workspace information */
export interface Workspace {
    id: string;
    name: string;
    root: TreeNode;
    handle: FileSystemDirectoryHandle;
    lastOpened: Date;
}

/** File service state */
export interface FileServiceState {
    isSupported: boolean;
    workspaces: Workspace[];
    currentWorkspaceId: string | null;
    currentPath: string;
    isLoading: boolean;
    error: string | null;
}

// ============================================================================
// File Service Class
// ============================================================================

/**
 * Service for managing file system operations.
 * 
 * This class provides:
 * - File open/save operations
 * - Workspace management
 * - Folder tree generation
 * - File system access via Browser API
 */
export class FileService {
    private static instance: FileService | null = null;
    
    // File System Access API support
    private _isSupported: boolean;
    
    // Workspaces
    private _workspaces: Workspace[] = [];
    private _currentWorkspaceId: string | null = null;
    private _currentPath: string = '';
    
    // State
    private _isLoading: boolean = false;
    private _error: string | null = null;
    
    // Event listeners
    private _stateListeners: Array<(state: FileServiceState) => void> = [];
    
    // ========================================================================
    // Singleton
    // ========================================================================
    
    public static getInstance(): FileService {
        if (!FileService.instance) {
            FileService.instance = new FileService();
        }
        return FileService.instance;
    }
    
    private constructor() {
        this._isSupported = this.checkSupport();
    }
    
    // ========================================================================
    // Support Check
    // ========================================================================
    
    /**
     * Check if File System Access API is supported.
     */
    private checkSupport(): boolean {
        return 'showDirectoryPicker' in window || 'showOpenFilePicker' in window;
    }
    
    /**
     * Check if file system access is supported.
     */
    public isSupported(): boolean {
        return this._isSupported;
    }
    
    // ========================================================================
    // State Management
    // ========================================================================
    
    /**
     * Get current state.
     */
    public getState(): FileServiceState {
        return {
            isSupported: this._isSupported,
            workspaces: this._workspaces,
            currentWorkspaceId: this._currentWorkspaceId,
            currentPath: this._currentPath,
            isLoading: this._isLoading,
            error: this._error,
        };
    }
    
    /**
     * Set loading state.
     */
    private setLoading(loading: boolean): void {
        this._isLoading = loading;
        this.emitStateUpdate();
    }
    
    /**
     * Set error.
     */
    private setError(error: string | null): void {
        this._error = error;
        this.emitStateUpdate();
    }
    
    // ========================================================================
    // Workspace Management
    // ========================================================================
    
    /**
     * Open a workspace (directory).
     */
    public async openWorkspace(): Promise<Workspace | null> {
        if (!this._isSupported) {
            this.setError('File System Access API not supported');
            return null;
        }
        
        this.setLoading(true);
        this.setError(null);
        
        try {
            const handle = await (window as any).showDirectoryPicker({
                mode: 'readwrite',
            });
            
            if (!handle) {
                return null; // User cancelled
            }
            
            const root = await this.buildTreeFromHandle(handle, '');
            const workspace: Workspace = {
                id: `workspace-${Date.now()}`,
                name: handle.name,
                root,
                handle,
                lastOpened: new Date(),
            };
            
            this._workspaces.push(workspace);
            this._currentWorkspaceId = workspace.id;
            this._currentPath = handle.name;
            
            this.setLoading(false);
            this.emitStateUpdate();
            
            return workspace;
            
        } catch (error) {
            this.setError(`Failed to open workspace: ${error}`);
            this.setLoading(false);
            return null;
        }
    }
    
    /**
     * Close current workspace.
     */
    public closeWorkspace(): void {
        this._currentWorkspaceId = null;
        this._currentPath = '';
        this.emitStateUpdate();
    }
    
    /**
     * Get current workspace.
     */
    public getCurrentWorkspace(): Workspace | null {
        if (!this._currentWorkspaceId) return null;
        return this._workspaces.find(w => w.id === this._currentWorkspaceId) || null;
    }
    
    /**
     * Get all workspaces.
     */
    public getWorkspaces(): Workspace[] {
        return [...this._workspaces];
    }
    
    // ========================================================================
    // Tree Building
    // ========================================================================
    
    /**
     * Build a tree structure from a directory handle.
     */
    private async buildTreeFromHandle(
        handle: FileSystemDirectoryHandle,
        path: string
    ): Promise<TreeNode> {
        const node: TreeNode = {
            id: handle.name + path,
            name: handle.name,
            path: path ? `${path}/${handle.name}` : handle.name,
            type: 'directory',
            handle,
            children: [],
        };
        
        try {
            for await (const [name, entry] of handle.entries()) {
                const childPath = path ? `${path}/${name}` : name;
                
                if (entry.kind === 'file') {
                    node.children?.push({
                        id: name + childPath,
                        name,
                        path: childPath,
                        type: 'file',
                        size: (await (entry as FileSystemFileHandle).getFile()).size,
                        lastModified: (await (entry as FileSystemFileHandle).getFile()).lastModified,
                        handle: entry as FileSystemFileHandle,
                    });
                } else if (entry.kind === 'directory') {
                    const childNode = await this.buildTreeFromHandle(
                        entry as FileSystemDirectoryHandle,
                        childPath
                    );
                    node.children?.push(childNode);
                }
            }
        } catch (error) {
            console.error('[FileService] Error building tree:', error);
        }
        
        // Sort children: directories first, then files, alphabetically
        node.children?.sort((a, b) => {
            if (a.type !== b.type) {
                return a.type === 'directory' ? -1 : 1;
            }
            return a.name.localeCompare(b.name);
        });
        
        return node;
    }
    
    /**
     * Refresh current workspace tree.
     */
    public async refreshWorkspace(): Promise<boolean> {
        const workspace = this.getCurrentWorkspace();
        if (!workspace || !workspace.handle) return false;
        
        this.setLoading(true);
        
        try {
            workspace.root = await this.buildTreeFromHandle(workspace.handle, '');
            this.emitStateUpdate();
            this.setLoading(false);
            return true;
        } catch (error) {
            this.setError(`Failed to refresh workspace: ${error}`);
            this.setLoading(false);
            return false;
        }
    }
    
    // ========================================================================
    // File Operations
    // ========================================================================
    
    /**
     * Open a file and read its content.
     */
    public async openFile(): Promise<{ content: string; name: string; } | null> {
        if (!this._isSupported) {
            this.setError('File System Access API not supported');
            return null;
        }
        
        this.setLoading(true);
        this.setError(null);
        
        try {
            const [handle] = await (window as any).showOpenFilePicker({
                types: [
                    {
                        description: 'MML Files',
                        accept: {
                            'text/plain': ['.gwi', '.mml', '.muc', '.mdl', '.mus', '.txt'],
                        },
                    },
                ],
                multiple: false,
            });
            
            if (!handle) {
                this.setLoading(false);
                return null; // User cancelled
            }
            
            const file = await handle.getFile();
            const content = await file.text();
            
            this.setLoading(false);
            return {
                content,
                name: file.name,
            };
            
        } catch (error) {
            this.setError(`Failed to open file: ${error}`);
            this.setLoading(false);
            return null;
        }
    }
    
    /**
     * Save content to a file.
     */
    public async saveFile(
        content: string,
        suggestedName: string = 'untitled.gwi'
    ): Promise<boolean> {
        if (!this._isSupported) {
            this.setError('File System Access API not supported');
            return false;
        }
        
        this.setLoading(true);
        this.setError(null);
        
        try {
            const options: any = {
                suggestedName,
                types: [
                    {
                        description: 'MML Files',
                        accept: {
                            'text/plain': ['.gwi', '.mml', '.muc', '.mdl', '.mus'],
                        },
                    },
                ],
            };
            
            const handle = await (window as any).showSaveFilePicker(options);
            
            if (!handle) {
                this.setLoading(false);
                return false; // User cancelled
            }
            
            const writable = await handle.createWritable();
            await writable.write(content);
            await writable.close();
            
            this.setLoading(false);
            return true;
            
        } catch (error) {
            this.setError(`Failed to save file: ${error}`);
            this.setLoading(false);
            return false;
        }
    }
    
    /**
     * Open a file from the workspace tree.
     */
    public async openFileFromTree(node: TreeNode): Promise<string | null> {
        if (node.type !== 'file' || !node.handle) return null;
        
        try {
            const handle = node.handle as FileSystemFileHandle;
            const file = await handle.getFile();
            return await file.text();
        } catch (error) {
            this.setError(`Failed to open file from tree: ${error}`);
            return null;
        }
    }
    
    // ========================================================================
    // File Type Detection
    // ========================================================================
    
    /**
     * Detect MML language from file extension.
     */
    public detectLanguage(filename: string): string {
        const ext = filename.split('.').pop()?.toLowerCase();
        
        switch (ext) {
            case 'gwi':
                return 'gwi';
            case 'muc':
                return 'muc';
            case 'mdl':
                return 'mdl';
            case 'mus':
                return 'mus';
            case 'mml':
            default:
                return 'mml';
        }
    }
    
    // ========================================================================
    // Filtering
    // ========================================================================
    
    /**
     * Filter tree to show only MML-related files.
     */
    public filterMMLFiles(node: TreeNode): TreeNode {
        if (node.type === 'file') {
            const ext = node.name.split('.').pop()?.toLowerCase();
            const mmlExtensions = ['gwi', 'mml', 'muc', 'mdl', 'mus', 'txt'];
            if (mmlExtensions.includes(ext || '')) {
                return node;
            }
            return { ...node, name: `${node.name} (unsupported)` };
        }
        
        if (node.children) {
            return {
                ...node,
                children: node.children
                    .map(child => this.filterMMLFiles(child))
                    .filter(child => child.type === 'directory' || 
                        child.name.split('.').pop()?.toLowerCase() !== 'unsupported'),
            };
        }
        
        return node;
    }
    
    // ========================================================================
    // Event Listeners
    // ========================================================================
    
    /**
     * Add a listener for state changes.
     */
    public addStateListener(listener: (state: FileServiceState) => void): void {
        this._stateListeners.push(listener);
    }
    
    /**
     * Remove a listener.
     */
    public removeStateListener(listener: (state: FileServiceState) => void): void {
        this._stateListeners = this._stateListeners.filter(l => l !== listener);
    }
    
    /**
     * Emit state update to all listeners.
     */
    private emitStateUpdate(): void {
        const state = this.getState();
        this._stateListeners.forEach(listener => listener(state));
    }
    
    // ========================================================================
    // Cleanup
    // ========================================================================
    
    /**
     * Clean up resources.
     */
    /**
     * Save MIDI binary data to a .mid file with download.
     */
    public async saveMidiFile(
        data: Uint8Array,
        suggestedName: string = 'untitled.mid'
    ): Promise<boolean> {
        try {
            // Create a Blob from the MIDI data
            const blob = new Blob([data], { type: 'audio/midi' });
            const url = URL.createObjectURL(blob);
            
            // Create a download link
            const a = document.createElement('a');
            a.href = url;
            a.download = suggestedName;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            
            return true;
        } catch (error) {
            this.setError(`Failed to save MIDI file: ${error}`);
            return false;
        }
    }

    public cleanup(): void {
        this._stateListeners = [];
        this._workspaces = [];
        this._currentWorkspaceId = null;
        this._currentPath = '';
        this._isLoading = false;
        this._error = null;
    }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const fileService = FileService.getInstance();

// Type declarations for File System Access API
declare global {
    interface Window {
        showDirectoryPicker: (options?: any) => Promise<FileSystemDirectoryHandle | null>;
        showOpenFilePicker: (options?: any) => Promise<FileSystemFileHandle[]>;
        showSaveFilePicker: (options?: any) => Promise<FileSystemFileHandle | null>;
    }
    
    interface FileSystemDirectoryHandle {
        entries(): AsyncIterable<[string, FileSystemFileHandle | FileSystemDirectoryHandle]>;
    }
}
