/**
 * Storage Service
 * 
 * Provides offline storage capabilities using IndexedDB.
 * Stores documents, settings, and other persistent data.
 * 
 * This is part of Phase 7: Polish and Optimization.
 */

// ============================================================================
// Types
// ============================================================================

/** Document for offline storage */
export interface StoredDocument {
    id: string;
    filename: string;
    content: string;
    language: string;
    encoding: string;
    createdAt: Date;
    updatedAt: Date;
    lastOpened: Date;
}

/** Workspace for offline storage */
export interface StoredWorkspace {
    id: string;
    name: string;
    path: string;
    createdAt: Date;
    lastOpened: Date;
    documentIds: string[];
}

/** Storage database schema */
export interface StorageSchema {
    documents: StoredDocument[];
    workspaces: StoredWorkspace[];
    settings: Record<string, unknown>;
    recentFiles: Array<{ path: string; openedAt: Date }>;
}

/** Storage service state */
export interface StorageServiceState {
    isSupported: boolean;
    isReady: boolean;
    error: string | null;
    documentCount: number;
    workspaceCount: number;
}

// ============================================================================
// Database Configuration
// ============================================================================

const DATABASE_NAME = 'mml2vgm-ide-storage';
const DATABASE_VERSION = 1;

const STORE_NAMES = {
    DOCUMENTS: 'documents',
    WORKSPACES: 'workspaces',
    SETTINGS: 'settings',
    RECENT_FILES: 'recentFiles',
} as const;

// ============================================================================
// Storage Service Class
// ============================================================================

/**
 * Storage Service
 * 
 * Manages IndexedDB storage for offline document persistence.
 * This enables the IDE to work offline and retain documents between sessions.
 */
export class StorageService {
    private static instance: StorageService | null = null;
    
    private db: IDBDatabase | null = null;
    private _isSupported: boolean = true;
    private _isReady: boolean = false;
    private _error: string | null = null;
    private _documentCount: number = 0;
    private _workspaceCount: number = 0;
    
    // Event listeners
    private stateListeners: Array<(state: StorageServiceState) => void> = [];
    
    // ========================================================================
    // Singleton
    // ========================================================================
    
    public static getInstance(): StorageService {
        if (!StorageService.instance) {
            StorageService.instance = new StorageService();
        }
        return StorageService.instance;
    }
    
    private constructor() {
        this._isSupported = this.checkSupport();
    }
    
    // ========================================================================
    // Support Check
    // ========================================================================
    
    /**
     * Check if IndexedDB is supported.
     */
    private checkSupport(): boolean {
        return 'indexedDB' in window;
    }
    
    /**
     * Check if storage is supported.
     */
    public get isSupported(): boolean {
        return this._isSupported;
    }
    
    // ========================================================================
    // Initialization
    // ========================================================================
    
    /**
     * Initialize the storage service.
     */
    public async init(): Promise<void> {
        if (!this._isSupported) {
            this._error = 'IndexedDB is not supported in this browser';
            this.notifyListeners();
            return;
        }
        
        if (this._isReady) {
            return;
        }
        
        return new Promise((resolve, reject) => {
            const request = indexedDB.open(DATABASE_NAME, DATABASE_VERSION);
            
            request.onerror = (event) => {
                this._error = `Failed to open database: ${event}`;
                this.notifyListeners();
                reject(new Error(this._error));
            };
            
            request.onsuccess = (event) => {
                this.db = (event.target as IDBOpenDBRequest).result;
                this._isReady = true;
                this._error = null;
                
                // Count documents and workspaces
                this.countDocuments();
                this.countWorkspaces();
                
                this.notifyListeners();
                resolve();
            };
            
            request.onupgradeneeded = (event) => {
                const db = (event.target as IDBOpenDBRequest).result;
                
                // Create object stores if they don't exist
                if (!db.objectStoreNames.contains(STORE_NAMES.DOCUMENTS)) {
                    db.createObjectStore(STORE_NAMES.DOCUMENTS, { keyPath: 'id' });
                }
                if (!db.objectStoreNames.contains(STORE_NAMES.WORKSPACES)) {
                    db.createObjectStore(STORE_NAMES.WORKSPACES, { keyPath: 'id' });
                }
                if (!db.objectStoreNames.contains(STORE_NAMES.SETTINGS)) {
                    db.createObjectStore(STORE_NAMES.SETTINGS, { keyPath: 'key' });
                }
                if (!db.objectStoreNames.contains(STORE_NAMES.RECENT_FILES)) {
                    const store = db.createObjectStore(STORE_NAMES.RECENT_FILES, { keyPath: 'path' });
                    store.createIndex('openedAt', 'openedAt', { unique: false });
                }
            };
        });
    }
    
    /**
     * Check if storage is ready.
     */
    public get isReady(): boolean {
        return this._isReady;
    }
    
    /**
     * Get current error.
     */
    public get error(): string | null {
        return this._error;
    }
    
    // ========================================================================
    // Document Storage
    // ========================================================================
    
    /**
     * Count documents in storage.
     */
    private async countDocuments(): Promise<void> {
        if (!this.db) return;
        
        const request = this.db.transaction(STORE_NAMES.DOCUMENTS).objectStore(STORE_NAMES.DOCUMENTS).count();
        request.onsuccess = () => {
            this._documentCount = request.result as number;
            this.notifyListeners();
        };
    }
    
    /**
     * Count workspaces in storage.
     */
    private async countWorkspaces(): Promise<void> {
        if (!this.db) return;
        
        const request = this.db.transaction(STORE_NAMES.WORKSPACES).objectStore(STORE_NAMES.WORKSPACES).count();
        request.onsuccess = () => {
            this._workspaceCount = request.result as number;
            this.notifyListeners();
        };
    }
    
    /**
     * Save a document.
     */
    public async saveDocument(doc: StoredDocument): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const store = this.db.transaction(STORE_NAMES.DOCUMENTS, 'readwrite')
                .objectStore(STORE_NAMES.DOCUMENTS);
            
            const existingRequest = store.get(doc.id);
            existingRequest.onsuccess = () => {
                const existing = existingRequest.result;
                
                const documentToSave: StoredDocument = {
                    ...doc,
                    updatedAt: new Date(),
                    createdAt: existing?.createdAt || new Date(),
                };
                
                const saveRequest = store.put(documentToSave);
                saveRequest.onsuccess = () => {
                    this.countDocuments();
                    resolve();
                };
                saveRequest.onerror = () => {
                    reject(new Error('Failed to save document'));
                };
            };
            existingRequest.onerror = () => {
                reject(new Error('Failed to check existing document'));
            };
        });
    }
    
    /**
     * Load a document by ID.
     */
    public async loadDocument(id: string): Promise<StoredDocument | null> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.DOCUMENTS).objectStore(STORE_NAMES.DOCUMENTS).get(id);
            request.onsuccess = () => {
                resolve(request.result as StoredDocument || null);
            };
            request.onerror = () => {
                reject(new Error('Failed to load document'));
            };
        });
    }
    
    /**
     * Delete a document.
     */
    public async deleteDocument(id: string): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.DOCUMENTS, 'readwrite')
                .objectStore(STORE_NAMES.DOCUMENTS)
                .delete(id);
            
            request.onsuccess = () => {
                this.countDocuments();
                resolve();
            };
            request.onerror = () => {
                reject(new Error('Failed to delete document'));
            };
        });
    }
    
    /**
     * List all documents.
     */
    public async listDocuments(): Promise<StoredDocument[]> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.DOCUMENTS).objectStore(STORE_NAMES.DOCUMENTS).getAll();
            request.onsuccess = () => {
                // Sort by updatedAt, most recent first
                const docs = (request.result as StoredDocument[]).sort(
                    (a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
                );
                resolve(docs);
            };
            request.onerror = () => {
                reject(new Error('Failed to list documents'));
            };
        });
    }
    
    // ========================================================================
    // Workspace Storage
    // ========================================================================
    
    /**
     * Save a workspace.
     */
    public async saveWorkspace(workspace: StoredWorkspace): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const store = this.db.transaction(STORE_NAMES.WORKSPACES, 'readwrite')
                .objectStore(STORE_NAMES.WORKSPACES);
            
            const existingRequest = store.get(workspace.id);
            existingRequest.onsuccess = () => {
                const existing = existingRequest.result;
                
                const workspaceToSave: StoredWorkspace = {
                    ...workspace,
                    lastOpened: new Date(),
                    createdAt: existing?.createdAt || new Date(),
                };
                
                const saveRequest = store.put(workspaceToSave);
                saveRequest.onsuccess = () => {
                    this.countWorkspaces();
                    resolve();
                };
                saveRequest.onerror = () => {
                    reject(new Error('Failed to save workspace'));
                };
            };
            existingRequest.onerror = () => {
                reject(new Error('Failed to check existing workspace'));
            };
        });
    }
    
    /**
     * Load a workspace by ID.
     */
    public async loadWorkspace(id: string): Promise<StoredWorkspace | null> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.WORKSPACES).objectStore(STORE_NAMES.WORKSPACES).get(id);
            request.onsuccess = () => {
                resolve(request.result as StoredWorkspace || null);
            };
            request.onerror = () => {
                reject(new Error('Failed to load workspace'));
            };
        });
    }
    
    /**
     * Delete a workspace.
     */
    public async deleteWorkspace(id: string): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.WORKSPACES, 'readwrite')
                .objectStore(STORE_NAMES.WORKSPACES)
                .delete(id);
            
            request.onsuccess = () => {
                this.countWorkspaces();
                resolve();
            };
            request.onerror = () => {
                reject(new Error('Failed to delete workspace'));
            };
        });
    }
    
    /**
     * List all workspaces.
     */
    public async listWorkspaces(): Promise<StoredWorkspace[]> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.WORKSPACES).objectStore(STORE_NAMES.WORKSPACES).getAll();
            request.onsuccess = () => {
                const workspaces = (request.result as StoredWorkspace[]).sort(
                    (a, b) => new Date(b.lastOpened).getTime() - new Date(a.lastOpened).getTime()
                );
                resolve(workspaces);
            };
            request.onerror = () => {
                reject(new Error('Failed to list workspaces'));
            };
        });
    }
    
    // ========================================================================
    // Settings Storage
    // ========================================================================
    
    /**
     * Save a setting.
     */
    public async saveSetting(key: string, value: unknown): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const store = this.db.transaction(STORE_NAMES.SETTINGS, 'readwrite')
                .objectStore(STORE_NAMES.SETTINGS);
            
            const saveRequest = store.put({ key, value, updatedAt: new Date() });
            saveRequest.onsuccess = () => resolve();
            saveRequest.onerror = () => reject(new Error('Failed to save setting'));
        });
    }
    
    /**
     * Load a setting.
     */
    public async loadSetting<T>(key: string): Promise<T | null> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.SETTINGS).objectStore(STORE_NAMES.SETTINGS).get(key);
            request.onsuccess = () => {
                const result = request.result as { key: string; value: T; updatedAt: Date } | undefined;
                resolve(result?.value ?? null);
            };
            request.onerror = () => reject(new Error('Failed to load setting'));
        });
    }
    
    /**
     * Delete a setting.
     */
    public async deleteSetting(key: string): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.SETTINGS, 'readwrite')
                .objectStore(STORE_NAMES.SETTINGS)
                .delete(key);
            
            request.onsuccess = () => resolve();
            request.onerror = () => reject(new Error('Failed to delete setting'));
        });
    }
    
    /**
     * List all settings.
     */
    public async listSettings(): Promise<Record<string, unknown>> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.SETTINGS).objectStore(STORE_NAMES.SETTINGS).getAll();
            request.onsuccess = () => {
                const settings: Record<string, unknown> = {};
                (request.result as Array<{ key: string; value: unknown }>).forEach(
                    (item) => { settings[item.key] = item.value; }
                );
                resolve(settings);
            };
            request.onerror = () => reject(new Error('Failed to list settings'));
        });
    }
    
    // ========================================================================
    // Recent Files Storage
    // ========================================================================
    
    /**
     * Add a file to recent files.
     */
    public async addRecentFile(path: string): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const store = this.db.transaction(STORE_NAMES.RECENT_FILES, 'readwrite')
                .objectStore(STORE_NAMES.RECENT_FILES);
            
            store.put({ path, openedAt: new Date() });
            
            // Limit to 50 recent files
            const countRequest = store.count();
            countRequest.onsuccess = () => {
                const count = countRequest.result as number;
                if (count > 50) {
                    // Delete oldest files
                    const index = store.index('openedAt');
                    const getAllRequest = index.getAll();
                    getAllRequest.onsuccess = () => {
                        const files = getAllRequest.result as Array<{ path: string; openedAt: Date }>;
                        files.sort((a, b) => new Date(a.openedAt).getTime() - new Date(b.openedAt).getTime());
                        
                        // Keep only 50 most recent
                        while (files.length > 50) {
                            const oldest = files.shift()!;
                            store.delete(oldest.path);
                        }
                        resolve();
                    };
                    getAllRequest.onerror = () => resolve(); // Non-critical
                } else {
                    resolve();
                }
            };
            countRequest.onerror = () => resolve(); // Non-critical
        });
    }
    
    /**
     * Get recent files.
     */
    public async getRecentFiles(): Promise<Array<{ path: string; openedAt: Date }>> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const store = this.db.transaction(STORE_NAMES.RECENT_FILES).objectStore(STORE_NAMES.RECENT_FILES);
            const index = store.index('openedAt');
            const request = index.getAll();
            
            request.onsuccess = () => {
                const files = request.result as Array<{ path: string; openedAt: Date }>;
                // Sort by openedAt, most recent first
                files.sort((a, b) => new Date(b.openedAt).getTime() - new Date(a.openedAt).getTime());
                resolve(files);
            };
            request.onerror = () => reject(new Error('Failed to get recent files'));
        });
    }
    
    /**
     * Clear recent files.
     */
    public async clearRecentFiles(): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const request = this.db.transaction(STORE_NAMES.RECENT_FILES, 'readwrite')
                .objectStore(STORE_NAMES.RECENT_FILES)
                .clear();
            
            request.onsuccess = () => resolve();
            request.onerror = () => reject(new Error('Failed to clear recent files'));
        });
    }
    
    // ========================================================================
    // Bulk Operations
    // ========================================================================
    
    /**
     * Clear all data.
     */
    public async clearAll(): Promise<void> {
        if (!this.db) {
            throw new Error('Storage not initialized');
        }
        
        return new Promise((resolve, reject) => {
            const transaction = this.db.transaction(
                [STORE_NAMES.DOCUMENTS, STORE_NAMES.WORKSPACES, STORE_NAMES.SETTINGS, STORE_NAMES.RECENT_FILES],
                'readwrite'
            );
            
            Promise.all([
                transaction.objectStore(STORE_NAMES.DOCUMENTS).clear(),
                transaction.objectStore(STORE_NAMES.WORKSPACES).clear(),
                transaction.objectStore(STORE_NAMES.SETTINGS).clear(),
                transaction.objectStore(STORE_NAMES.RECENT_FILES).clear(),
            ]).then(() => {
                this._documentCount = 0;
                this._workspaceCount = 0;
                this.notifyListeners();
                resolve();
            }).catch(() => {
                reject(new Error('Failed to clear all data'));
            });
        });
    }
    
    /**
     * Export all data as JSON.
     */
    public async exportData(): Promise<StorageSchema> {
        const [documents, workspaces, settings, recentFiles] = await Promise.all([
            this.listDocuments(),
            this.listWorkspaces(),
            this.listSettings(),
            this.getRecentFiles(),
        ]);
        
        return {
            documents,
            workspaces,
            settings,
            recentFiles,
        };
    }
    
    /**
     * Import data from JSON.
     */
    public async importData(data: Partial<StorageSchema>): Promise<void> {
        if (data.documents) {
            await Promise.all(data.documents.map(doc => this.saveDocument(doc)));
        }
        if (data.workspaces) {
            await Promise.all(data.workspaces.map(ws => this.saveWorkspace(ws)));
        }
        if (data.settings) {
            await Promise.all(
                Object.entries(data.settings).map(([key, value]) => 
                    this.saveSetting(key, value)
                )
            );
        }
        if (data.recentFiles) {
            await Promise.all(data.recentFiles.map(file => this.addRecentFile(file.path)));
        }
    }
    
    // ========================================================================
    // State Management
    // ========================================================================
    
    /**
     * Get current state.
     */
    public getState(): StorageServiceState {
        return {
            isSupported: this._isSupported,
            isReady: this._isReady,
            error: this._error,
            documentCount: this._documentCount,
            workspaceCount: this._workspaceCount,
        };
    }
    
    /**
     * Subscribe to state changes.
     */
    public subscribe(callback: (state: StorageServiceState) => void): () => void {
        this.stateListeners.push(callback);
        callback(this.getState());
        
        return () => {
            const index = this.stateListeners.indexOf(callback);
            if (index >= 0) {
                this.stateListeners.splice(index, 1);
            }
        };
    }
    
    private notifyListeners(): void {
        this.stateListeners.forEach(callback => callback(this.getState()));
    }
}

// ============================================================================
// Service Worker Registration
// ============================================================================

/**
 * Register the service worker for offline support.
 */
export async function registerServiceWorker(): Promise<void> {
    if ('serviceWorker' in navigator) {
        try {
            // Use the window load event to ensure the page is fully loaded
            await new Promise<void>((resolve) => {
                if (document.readyState === 'complete') {
                    resolve();
                } else {
                    window.addEventListener('load', () => resolve());
                }
            });
            
            const registration = await navigator.serviceWorker.register('/sw.js');
            console.log('[SW] Service Worker registered:', registration.scope);
            
            // Listen for controller changes
            navigator.serviceWorker.addEventListener('controllerchange', () => {
                console.log('[SW] Controller changed, reloading...');
                window.location.reload();
            });
            
            // Listen for update found
            registration.addEventListener('updatefound', () => {
                console.log('[SW] New service worker found, updating...');
                const newWorker = registration.installing;
                newWorker?.addEventListener('statechange', () => {
                    if (newWorker.state === 'installed') {
                        console.log('[SW] New service worker installed, activating...');
                        // Skip waiting and activate immediately
                        newWorker.postMessage({ type: 'SKIP_WAITING' });
                    }
                });
            });
            
            // Check for updates periodically
            setInterval(() => {
                registration.update();
            }, 24 * 60 * 60 * 1000); // Check once per day
        } catch (error) {
            console.error('[SW] Failed to register service worker:', error);
        }
    }
}

/**
 * Unregister the service worker.
 */
export async function unregisterServiceWorker(): Promise<void> {
    if ('serviceWorker' in navigator) {
        const registrations = await navigator.serviceWorker.getRegistrations();
        for (const registration of registrations) {
            await registration.unregister();
        }
        console.log('[SW] Service worker unregistered');
    }
}

// ============================================================================
// Singleton Instance
// ============================================================================

/**
 * Singleton instance of the StorageService.
 */
export const storageService = StorageService.getInstance();

export default StorageService;
