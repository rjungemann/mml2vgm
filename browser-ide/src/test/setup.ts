/**
 * Test Setup
 * 
 * Global test setup for Vitest.
 * This file runs before all tests.
 */

import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

// Clean up after each test
afterEach(() => {
    cleanup();
});

// Mock WebAssembly for tests
// Since we can't actually run WASM in a test environment
// we mock the global WebAssembly object
if (!globalThis.WebAssembly) {
    globalThis.WebAssembly = {
        compile: vi.fn().mockResolvedValue({}),
        instantiate: vi.fn().mockResolvedValue({
            exports: {},
            instance: {},
        }),
        validate: vi.fn().mockResolvedValue(true),
        Module: class {
            constructor() {
                return {
                    exports: {},
                };
            }
        },
        Memory: class {
            constructor() {
                return {
                    buffer: new ArrayBuffer(0),
                    grow: vi.fn(),
                };
            }
        },
    } as any;
}

// Mock console methods to reduce noise in tests
const originalConsoleError = console.error;
const originalConsoleWarn = console.warn;

beforeAll(() => {
    // Suppress console.error and console.warn during tests
    // unless explicitly needed
    console.error = vi.fn((...args) => {
        originalConsoleError(...args);
    });
    console.warn = vi.fn((...args) => {
        originalConsoleWarn(...args);
    });
});

afterAll(() => {
    console.error = originalConsoleError;
    console.warn = originalConsoleWarn;
});

// Mock navigator for tests
Object.defineProperty(window.navigator, 'language', {
    value: 'en-US',
    writable: true,
});

// Mock service worker
if ('serviceWorker' in navigator) {
    Object.defineProperty(navigator, 'serviceWorker', {
        value: {
            register: vi.fn().mockResolvedValue({
                scope: '/',
                update: vi.fn(),
                addEventListener: vi.fn(),
                removeEventListener: vi.fn(),
            }),
            getRegistrations: vi.fn().mockResolvedValue([]),
        },
        writable: true,
    });
}

// Mock localStorage
const localStorageMock = (() => {
    let store: Record<string, string> = {};
    return {
        getItem: vi.fn((key: string) => store[key] || null),
        setItem: vi.fn((key: string, value: string) => {
            store[key] = value.toString();
        }),
        removeItem: vi.fn((key: string) => {
            delete store[key];
        }),
        clear: vi.fn(() => {
            store = {};
        }),
    };
})();

Object.defineProperty(window, 'localStorage', {
    value: localStorageMock,
    writable: true,
});

// Mock IndexedDB
class MockIDBFactory {
    private databases: Record<string, MockIDBDatabase> = {};
    
    open(name: string, version?: number): IDBOpenDBRequest {
        if (!this.databases[name]) {
            this.databases[name] = new MockIDBDatabase(name, version);
        }
        const request = new MockIDBOpenDBRequest(this.databases[name]);
        setTimeout(() => {
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBOpenDBRequest;
    }
    
    deleteDatabase(name: string): IDBOpenDBRequest {
        delete this.databases[name];
        const request = {} as IDBOpenDBRequest;
        setTimeout(() => {
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request;
    }
    
    cmp(a: any, b: any): number {
        return a < b ? -1 : a > b ? 1 : 0;
    }
}

class MockIDBDatabase {
    constructor(public name: string, public version: number = 1) {
        this.objectStoreNames = new DOMStringList();
    }
    
    objectStoreNames: DOMStringList;
    createObjectStore(name: string, options?: IDBObjectStoreParameters): IDBObjectStore {
        this.objectStoreNames.add(name);
        return new MockIDBObjectStore(name);
    }
    
    transaction(storeNames: string | string[], mode: IDBTransactionMode): IDBTransaction {
        return new MockIDBTransaction(this, storeNames, mode);
    }
    
    close(): void {
        // Mock implementation
    }
}

class MockIDBObjectStore {
    constructor(public name: string) {
        this.indexNames = new DOMStringList();
        this.keyPath = null;
    }
    
    indexNames: DOMStringList;
    keyPath: string | null;
    autoIncrement: boolean = false;
    
    put(value: any, key?: IDBValidKey): IDBRequest {
        const request = new MockIDBRequest();
        setTimeout(() => {
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBRequest;
    }
    
    get(key: IDBValidKey): IDBRequest {
        const request = new MockIDBRequest();
        setTimeout(() => {
            request.result = null;
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBRequest;
    }
    
    getAll(query?: IDBValidKey | IDBKeyRange, count?: number): IDBRequest {
        const request = new MockIDBRequest();
        setTimeout(() => {
            request.result = [];
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBRequest;
    }
    
    delete(key: IDBValidKey): IDBRequest {
        const request = new MockIDBRequest();
        setTimeout(() => {
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBRequest;
    }
    
    clear(): IDBRequest {
        const request = new MockIDBRequest();
        setTimeout(() => {
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBRequest;
    }
    
    count(): IDBRequest {
        const request = new MockIDBRequest();
        setTimeout(() => {
            request.result = 0;
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBRequest;
    }
    
    createIndex(name: string, keyPath: string | string[], options?: IDBIndexParameters): IDBIndex {
        this.indexNames.add(name);
        return new MockIDBIndex(name, keyPath);
    }
}

class MockIDBTransaction {
    constructor(
        public db: MockIDBDatabase,
        public storeNames: string | string[],
        public mode: IDBTransactionMode
    ) {
        this.objectStoreNames = new DOMStringList();
    }
    
    objectStoreNames: DOMStringList;
    
    objectStore(name: string): IDBObjectStore {
        return this.db.createObjectStore(name);
    }
    
    commit(): void {
        // Mock implementation
    }
    
    abort(): void {
        // Mock implementation
    }
    
    oncomplete: ((this: IDBTransaction, ev: Event) => any) | null = null;
    onerror: ((this: IDBTransaction, ev: Event) => any) | null = null;
    onabort: ((this: IDBTransaction, ev: Event) => any) | null = null;
}

class MockIDBRequest {
    readyState: 'pending' | 'done' = 'pending';
    result: any = undefined;
    error: DOMError | null = null;
    source: IDBObjectStore | IDBIndex | IDBCursor | null = null;
    transaction: IDBTransaction | null = null;
    
    onsuccess: ((this: IDBRequest, ev: Event) => any) | null = null;
    onerror: ((this: IDBRequest, ev: Event) => any) | null = null;
}

class MockIDBOpenDBRequest extends MockIDBRequest {
    constructor(public db: MockIDBDatabase) {
        super();
    }
}

class MockIDBIndex {
    constructor(public name: string, public keyPath: string | string[]) {}
    
    get(key: IDBValidKey): IDBRequest {
        const request = new MockIDBRequest();
        setTimeout(() => {
            request.result = null;
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBRequest;
    }
    
    getAll(query?: IDBValidKey | IDBKeyRange, count?: number): IDBRequest {
        const request = new MockIDBRequest();
        setTimeout(() => {
            request.result = [];
            request.onsuccess?.(new Event('success') as any);
        }, 0);
        return request as unknown as IDBRequest;
    }
}

// DOMStringList mock
class DOMStringList {
    private items: string[] = [];
    
    add(value: string): void {
        if (!this.items.includes(value)) {
            this.items.push(value);
        }
    }
    
    remove(value: string): void {
        this.items = this.items.filter(item => item !== value);
    }
    
    contains(value: string): boolean {
        return this.items.includes(value);
    }
    
    item(index: number): string | null {
        return this.items[index] || null;
    }
    
    get length(): number {
        return this.items.length;
    }
    
    [Symbol.iterator](): IterableIterator<string> {
        return this.items.values();
    }
}

// Install mock IndexedDB
window.indexedDB = new MockIDBFactory() as unknown as IDBFactory;
window.IDBFactory = MockIDBFactory as unknown as typeof IDBFactory;
window.IDBDatabase = MockIDBDatabase as unknown as typeof IDBDatabase;
window.IDBObjectStore = MockIDBObjectStore as unknown as typeof IDBObjectStore;
window.IDBTransaction = MockIDBTransaction as unknown as typeof IDBTransaction;
window.IDBRequest = MockIDBRequest as unknown as typeof IDBRequest;
window.IDBOpenDBRequest = MockIDBOpenDBRequest as unknown as typeof IDBOpenDBRequest;
window.IDBIndex = MockIDBIndex as unknown as typeof IDBIndex;

// Mock ResizeObserver
class MockResizeObserver {
    constructor(private callback: ResizeObserverCallback) {}
    
    observe(target: Element): void {
        // Mock implementation
    }
    
    unobserve(target: Element): void {
        // Mock implementation
    }
    
    disconnect(): void {
        // Mock implementation
    }
}

window.ResizeObserver = MockResizeObserver as unknown as typeof ResizeObserver;

// Mock Monaco Editor for tests
vi.mock('@monaco-editor/react', () => ({
    default: vi.fn(({ value, onChange, ...props }) => {
        const [text, setText] = React.useState(value || '');
        React.useEffect(() => {
            setText(value || '');
        }, [value]);
        return (
            <textarea
                value={text}
                onChange={(e) => {
                    setText(e.target.value);
                    onChange?.(e.target.value);
                }}
                data-testid="monaco-editor"
                {...props}
            />
        );
    }),
    useMonaco: vi.fn(),
}));

// Export test utilities
export * from '@testing-library/react';
export { default as userEvent } from '@testing-library/user-event';
