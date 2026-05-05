/**
 * Storage Service Tests
 * 
 * Tests for the IndexedDB storage service.
 * Part of Phase 7: Polish and Optimization.
 */

import { describe, it, expect, beforeAll, afterAll, vi } from 'vitest';
import { StorageService, storageService } from '@/services/storageService';

// Mock IndexedDB
const mockIndexedDB = {
    open: vi.fn(),
    deleteDatabase: vi.fn(),
};

// Mock IDBOpenDBRequest
class MockIDBOpenDBRequest {
    result: any;
    onsuccess: ((event: any) => void) | null = null;
    onerror: ((event: any) => void) | null = null;
    onupgradeneeded: ((event: any) => void) | null = null;
    
    constructor(result: any) {
        this.result = result;
    }
}

// Mock IDBDatabase
class MockIDBDatabase {
    name: string;
    version: number;
    objectStoreNames: DOMStringList;
    
    constructor(name: string, version: number) {
        this.name = name;
        this.version = version;
        this.objectStoreNames = new DOMStringList();
    }
    
    transaction(storeNames: string[], mode: IDBTransactionMode): MockIDBTransaction {
        return new MockIDBTransaction(this, storeNames, mode);
    }
    
    close() {}
}

// Mock DOMStringList
class DOMStringList {
    private items: string[] = [];
    
    contains(item: string): boolean {
        return this.items.includes(item);
    }
    
    add(item: string) {
        this.items.push(item);
    }
}

// Mock IDBTransaction
class MockIDBTransaction {
    private db: MockIDBDatabase;
    private storeNames: string[];
    private mode: IDBTransactionMode;
    
    constructor(db: MockIDBDatabase, storeNames: string[], mode: IDBTransactionMode) {
        this.db = db;
        this.storeNames = storeNames;
        this.mode = mode;
    }
    
    objectStore(name: string): MockIDBObjectStore {
        return new MockIDBObjectStore(this.db, name, this.mode);
    }
}

// Mock IDBObjectStore
class MockIDBObjectStore {
    private db: MockIDBDatabase;
    private name: string;
    private mode: IDBTransactionMode;
    private data: Map<string, any> = new Map();
    
    constructor(db: MockIDBDatabase, name: string, mode: IDBTransactionMode) {
        this.db = db;
        this.name = name;
        this.mode = mode;
    }
    
    get(key: string): MockIDBRequest {
        return new MockIDBRequest(this.data.get(key));
    }
    
    put(value: any, key?: string): MockIDBRequest {
        const actualKey = key || (value as any).id || (value as any).key;
        this.data.set(actualKey, value);
        return new MockIDBRequest(value);
    }
    
    delete(key: string): MockIDBRequest {
        this.data.delete(key);
        return new MockIDBRequest(undefined);
    }
    
    getAll(): MockIDBRequest {
        return new MockIDBRequest(Array.from(this.data.values()));
    }
    
    clear(): MockIDBRequest {
        this.data.clear();
        return new MockIDBRequest(undefined);
    }
    
    count(): MockIDBRequest {
        return new MockIDBRequest(this.data.size);
    }
    
    createIndex(name: string, keyPath: string, options?: IDBIndexParameters): MockIDBIndex {
        return new MockIDBIndex(name, keyPath);
    }
    
    index(name: string): MockIDBIndex {
        return new MockIDBIndex(name, '');
    }
}

// Mock IDBRequest
class MockIDBRequest {
    result: any;
    onsuccess: ((event: any) => void) | null = null;
    onerror: ((event: any) => void) | null = null;
    
    constructor(result: any) {
        this.result = result;
    }
}

// Mock IDBIndex
class MockIDBIndex {
    name: string;
    keyPath: string;
    
    constructor(name: string, keyPath: string) {
        this.name = name;
        this.keyPath = keyPath;
    }
    
    getAll(): MockIDBRequest {
        return new MockIDBRequest([]);
    }
}

// Setup mocks
beforeAll(() => {
    global.indexedDB = mockIndexedDB as any;
});

afterAll(() => {
    vi.restoreAllMocks();
});

describe('StorageService', () => {
    describe('Singleton Pattern', () => {
        it('should return the same instance', () => {
            const instance1 = StorageService.getInstance();
            const instance2 = StorageService.getInstance();
            expect(instance1).toBe(instance2);
        });
    });

    describe('Support Check', () => {
        it('should detect IndexedDB support', () => {
            global.indexedDB = mockIndexedDB as any;
            const service = new StorageService();
            expect(service.isSupported).toBe(true);
        });

        it('should detect when IndexedDB is not supported', () => {
            const originalIndexedDB = global.indexedDB;
            delete (global as any).indexedDB;
            
            const service = new StorageService();
            expect(service.isSupported).toBe(false);
            
            global.indexedDB = originalIndexedDB as any;
        });
    });

    describe('Initialization', () => {
        it('should handle initialization when not supported', async () => {
            const originalIndexedDB = global.indexedDB;
            delete (global as any).indexedDB;
            
            const service = new StorageService();
            await expect(service.init()).rejects.toThrow();
            expect(service.isSupported).toBe(false);
            
            global.indexedDB = originalIndexedDB as any;
        });
    });

    describe('Document Storage', () => {
        it('should save and load a document', async () => {
            const mockDoc: any = {
                id: 'test-doc-1',
                filename: 'test.gwi',
                content: '@0 v10 o4 l4 cdefgab',
                language: 'gwi',
                encoding: 'utf-8',
                createdAt: new Date(),
                updatedAt: new Date(),
                lastOpened: new Date(),
            };

            const service = new StorageService();
            
            // Mock the IndexedDB operations
            const mockDB = new MockIDBDatabase('mml2vgm-ide-storage', 1);
            mockDB.objectStoreNames.add('documents');
            
            const mockRequest = new MockIDBOpenDBRequest(mockDB);
            mockIndexedDB.open = vi.fn().mockReturnValue(mockRequest);

            // Manually set the database for testing
            (service as any).db = mockDB;
            (service as any)._isReady = true;

            await service.saveDocument(mockDoc);
            const loaded = await service.loadDocument('test-doc-1');
            
            expect(loaded).toBeDefined();
            expect(loaded?.id).toBe('test-doc-1');
        });
    });

    describe('State Management', () => {
        it('should return initial state', () => {
            const service = new StorageService();
            const state = service.getState();
            
            expect(state.isSupported).toBe(true);
            expect(state.isReady).toBe(false);
            expect(state.error).toBe(null);
            expect(state.documentCount).toBe(0);
            expect(state.workspaceCount).toBe(0);
        });

        it('should allow subscription to state changes', () => {
            const service = new StorageService();
            const callback = vi.fn();
            
            const unsubscribe = service.subscribe(callback);
            
            // Initial call
            expect(callback).toHaveBeenCalledTimes(1);
            
            // Unsubscribe
            unsubscribe();
        });
    });
});

describe('storageService (singleton)', () => {
    it('should be an instance of StorageService', () => {
        expect(storageService).toBeInstanceOf(StorageService);
    });
});
