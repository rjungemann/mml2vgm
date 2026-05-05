/**
 * Service Worker for mml2vgm Browser IDE
 * 
 * Provides offline caching for WASM, assets, and recent files.
 * This enables the IDE to work offline after the first load.
 */

const CACHE_NAME = 'mml2vgm-ide-v1';
const OFFLINE_URL = '/index.html';

// Core assets to cache (critical for offline functionality)
const CORE_ASSETS = [
    '/',
    '/index.html',
    '/favicon.ico',
    '/index.css',
];

// WASM assets
const WASM_ASSETS = [
    '/wasm/pkg/mml2vgm_wasm.js',
    '/wasm/pkg/mml2vgm_wasm_bg.wasm',
    '/wasm/pkg/mml2vgm_wasm.d.ts',
];

// Additional assets to cache
const ADDITIONAL_ASSETS = [
    '/src/main.tsx',
    '/src/App.tsx',
    '/src/index.css',
];

// Combine all assets to cache
const ASSETS_TO_CACHE = [
    ...CORE_ASSETS,
    ...WASM_ASSETS,
    ...ADDITIONAL_ASSETS,
];

// Install event - cache all core assets
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then((cache) => {
                console.log('[SW] Caching core assets');
                return cache.addAll(ASSETS_TO_CACHE);
            })
            .then(() => {
                console.log('[SW] Core assets cached successfully');
                // Force the waiting service worker to become active
                return self.skipWaiting();
            })
            .catch((error) => {
                console.error('[SW] Failed to cache assets:', error);
            })
    );
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches.keys().then((cacheNames) => {
            return Promise.all(
                cacheNames.map((cacheName) => {
                    // Delete old caches that don't match our current version
                    if (cacheName !== CACHE_NAME) {
                        console.log(`[SW] Deleting old cache: ${cacheName}`);
                        return caches.delete(cacheName);
                    }
                })
            );
        })
        .then(() => {
            console.log('[SW] Service worker activated');
            // Take control of all clients immediately
            return self.clients.claim();
        })
    );
});

// Fetch event - serve cached assets when offline
self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);
    
    // Skip cross-origin requests
    if (url.origin !== self.location.origin) {
        return fetch(event.request);
    }
    
    // For WASM files, use cache-first strategy with fallback to network
    if (url.pathname.includes('wasm') || url.pathname.includes('.wasm')) {
        event.respondWith(
            caches.match(event.request)
                .then((response) => {
                    if (response) {
                        console.log(`[SW] Serving cached WASM: ${url.pathname}`);
                        return response;
                    }
                    console.log(`[SW] Fetching WASM from network: ${url.pathname}`);
                    return fetch(event.request);
                })
        );
        return;
    }
    
    // For HTML, use network-first with fallback to cached offline page
    if (url.pathname.endsWith('.html') || url.pathname === '/') {
        event.respondWith(
            fetch(event.request)
                .catch(() => {
                    console.log('[SW] Offline - serving cached HTML');
                    return caches.match(OFFLINE_URL);
                })
        );
        return;
    }
    
    // For CSS, JS, and other assets, use cache-first strategy
    event.respondWith(
        caches.match(event.request)
            .then((response) => {
                if (response) {
                    console.log(`[SW] Serving cached asset: ${url.pathname}`);
                    return response;
                }
                console.log(`[SW] Fetching asset from network: ${url.pathname}`);
                return fetch(event.request);
            })
    );
});

// Message event - handle messages from the client
self.addEventListener('message', (event) => {
    if (event.data && event.data.type === 'SKIP_WAITING') {
        self.skipWaiting();
    }
    
    if (event.data && event.data.type === 'CACHE_FILE') {
        const { url, content } = event.data;
        event.waitUntil(
            caches.open(CACHE_NAME)
                .then((cache) => {
                    console.log(`[SW] Caching file: ${url}`);
                    return cache.put(url, new Response(content));
                })
        );
    }
    
    if (event.data && event.data.type === 'DELETE_CACHE') {
        event.waitUntil(
            caches.delete(event.data.cacheName)
                .then(() => {
                    console.log(`[SW] Deleted cache: ${event.data.cacheName}`);
                })
        );
    }
});

// Background sync for saving files when offline
self.addEventListener('sync', (event) => {
    if (event.tag === 'save-file') {
        event.waitUntil(
            // This would be implemented with IndexedDB sync
            console.log('[SW] Background sync for save-file')
        );
    }
});

console.log('[SW] Service Worker loaded');
