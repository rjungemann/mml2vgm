/**
 * Service Worker for mml2vgm Browser IDE
 * 
 * Provides offline caching for WASM, assets, and recent files.
 * This enables the IDE to work offline after the first load.
 */

// Cache version - increment when cache contents change
const CACHE_VERSION = 'mml2vgm-ide-v2';
const CACHE_NAME = CACHE_VERSION;
const OFFLINE_URL = '/index.html';

// Core assets that are always cached
const CORE_ASSETS = [
    '/',
    '/index.html',
    '/favicon.svg',
];

// WASM assets - critical for functionality
const WASM_ASSETS = [
    '/assets/mml2vgm_wasm.js',
    '/assets/mml2vgm_wasm_bg.wasm',
    '/assets/mml2vgm_wasm.d.ts',
];

// JS and CSS bundles
const BUNDLE_ASSETS = [
    '/assets/index-*.js',
    '/assets/index-*.css',
    '/assets/compilerWorker-*.ts',
];

// Locale files
const LOCALE_ASSETS = [
    '/locales/en.json',
    '/locales/ja.json',
];

// Sample files (default MML examples)
const SAMPLE_ASSETS = [
    '/samples/*',
];

// All assets to cache at install time
const ASSETS_TO_PRECACHE = [
    ...CORE_ASSETS,
    ...WASM_ASSETS,
    ...LOCALE_ASSETS,
];

// Assets to use stale-while-revalidate strategy for
const STALE_WHILE_REVALIDATE_ASSETS = [
    '/assets/index-*.js',
    '/assets/index-*.css',
    '/assets/compilerWorker-*.ts',
    '/locales/en.json',
    '/locales/ja.json',
];

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Check if a URL matches a pattern
 */
function matchesPattern(url, pattern) {
    if (pattern.includes('*')) {
        const regexPattern = pattern.replace(/\*/g, '.*');
        const regex = new RegExp('^' + regexPattern + '$');
        return regex.test(url.pathname);
    }
    return url.pathname === pattern;
}

/**
 * Check if a URL is for a WASM file
 */
function isWasmRequest(url) {
    return url.pathname.includes('wasm') || url.pathname.endsWith('.wasm');
}

/**
 * Check if a URL is for a locale file
 */
function isLocaleRequest(url) {
    return url.pathname.startsWith('/locales/') && url.pathname.endsWith('.json');
}

/**
 * Check if a URL is for a sample file
 */
function isSampleRequest(url) {
    return url.pathname.startsWith('/samples/');
}

/**
 * Check if a URL is for a bundle asset
 */
function isBundleAsset(url) {
    return url.pathname.startsWith('/assets/') && 
           (url.pathname.endsWith('.js') || url.pathname.endsWith('.css') || url.pathname.endsWith('.ts'));
}

// ============================================================================
// Cache Management
// ============================================================================

/**
 * Get the current cache version
 */
function getCurrentCacheVersion() {
    return CACHE_VERSION;
}

/**
 * Check if there's an update available by comparing cache versions
 */
async function checkForUpdate() {
    const cache = await caches.open(CACHE_NAME);
    const cachedResponse = await cache.match('/');
    
    if (cachedResponse) {
        // Compare the version in the cached index.html
        // or check for a version file
        try {
            const versionResponse = await cache.match('/version.txt');
            if (versionResponse) {
                const cachedVersion = await versionResponse.text();
                return cachedVersion !== CACHE_VERSION;
            }
        } catch (e) {
            // If we can't check, assume no update
        }
    }
    return false;
}

// ============================================================================
// Event Listeners
// ============================================================================

// Install event - cache all core assets
self.addEventListener('install', (event) => {
    event.waitUntil(
        (async () => {
            try {
                console.log(`[SW v${CACHE_VERSION}] Installing service worker`);
                
                const cache = await caches.open(CACHE_NAME);
                
                // First, cache the core assets and WASM files
                const assetsToCache = [
                    ...CORE_ASSETS,
                    ...WASM_ASSETS,
                    ...LOCALE_ASSETS,
                ];
                
                // Also cache any existing bundle files
                // We'll dynamically discover them
                const cachePromises = assetsToCache.map(async (asset) => {
                    // Skip patterns for now, handle them in fetch
                    if (asset.includes('*')) return;
                    try {
                        const url = new URL(asset, self.location.origin);
                        // Try to fetch and cache
                        const response = await fetch(url);
                        if (response && response.ok) {
                            await cache.put(asset, response);
                            console.log(`[SW] Cached: ${asset}`);
                        }
                    } catch (e) {
                        console.warn(`[SW] Could not cache ${asset}:`, e);
                    }
                });
                
                await Promise.all(cachePromises);
                
                // Also cache the samples directory
                try {
                    const samplesResponse = await caches.keys().then(async (cacheNames) => {
                        // This is a placeholder - samples will be cached on demand
                        return true;
                    });
                } catch (e) {
                    console.warn('[SW] Could not precache samples:', e);
                }
                
                console.log('[SW] Core assets cached successfully');
                
                // Force the waiting service worker to become active
                await self.skipWaiting();
                
                // Notify clients that a new version is available
                await self.clients.claim();
                
                // Send update available message to all clients
                const clients = await self.clients.matchAll();
                clients.forEach(client => {
                    client.postMessage({
                        type: 'SW_UPDATE_AVAILABLE',
                        version: CACHE_VERSION
                    });
                });
                
            } catch (error) {
                console.error('[SW] Failed to cache assets:', error);
            }
        })()
    );
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
    event.waitUntil(
        (async () => {
            try {
                console.log(`[SW v${CACHE_VERSION}] Activating service worker`);
                
                // Delete old caches that don't match our current version
                const cacheNames = await caches.keys();
                const deletePromises = cacheNames.map(async (cacheName) => {
                    // Keep caches that start with our prefix but have different versions
                    if (cacheName.startsWith('mml2vgm-ide-') && cacheName !== CACHE_NAME) {
                        console.log(`[SW] Deleting old cache: ${cacheName}`);
                        await caches.delete(cacheName);
                    } else if (cacheName !== CACHE_NAME) {
                        console.log(`[SW] Deleting unknown cache: ${cacheName}`);
                        await caches.delete(cacheName);
                    }
                });
                
                await Promise.all(deletePromises);
                
                console.log('[SW] Service worker activated');
                
                // Take control of all clients immediately
                await self.clients.claim();
                
            } catch (error) {
                console.error('[SW] Activation failed:', error);
            }
        })()
    );
});

// Fetch event - serve cached assets when offline
self.addEventListener('fetch', (event) => {
    const url = new URL(event.request.url);
    
    // Skip cross-origin requests
    if (url.origin !== self.location.origin) {
        event.respondWith(fetch(event.request));
        return;
    }
    
    // For WASM files, use cache-first strategy with fallback to network
    if (isWasmRequest(url) || url.pathname.includes('.wasm')) {
        event.respondWith(
            (async () => {
                const cache = await caches.open(CACHE_NAME);
                const cachedResponse = await cache.match(event.request);
                
                if (cachedResponse) {
                    console.log(`[SW] Serving cached WASM: ${url.pathname}`);
                    return cachedResponse;
                }
                
                console.log(`[SW] Fetching WASM from network: ${url.pathname}`);
                const response = await fetch(event.request);
                
                // Cache the response if it's successful
                if (response && response.ok) {
                    await cache.put(event.request, response.clone());
                }
                
                return response;
            })()
        );
        return;
    }
    
    // For HTML, use network-first with fallback to cached offline page
    if (url.pathname.endsWith('.html') || url.pathname === '/') {
        event.respondWith(
            (async () => {
                try {
                    // Try network first
                    const response = await fetch(event.request);
                    
                    // If successful, cache it
                    if (response && response.ok) {
                        const cache = await caches.open(CACHE_NAME);
                        await cache.put(event.request, response.clone());
                    }
                    
                    return response;
                } catch (e) {
                    console.log('[SW] Offline - serving cached HTML');
                    const cache = await caches.open(CACHE_NAME);
                    return cache.match(OFFLINE_URL) || cache.match('/');
                }
            })()
        );
        return;
    }
    
    // For locale files, use stale-while-revalidate strategy
    if (isLocaleRequest(url)) {
        event.respondWith(
            (async () => {
                const cache = await caches.open(CACHE_NAME);
                const cachedResponse = await cache.match(event.request);
                
                // If we have a cached version, return it immediately
                if (cachedResponse) {
                    console.log(`[SW] Serving cached locale (stale-while-revalidate): ${url.pathname}`);
                    
                    // Start revalidation in the background
                    fetch(event.request).then(async (response) => {
                        if (response && response.ok) {
                            // Check if the content has changed
                            const cachedText = await cachedResponse.clone().text();
                            const newText = await response.clone().text();
                            
                            if (cachedText !== newText) {
                                console.log(`[SW] Locale updated: ${url.pathname}`);
                                await cache.put(event.request, response);
                                
                                // Notify clients that locale has been updated
                                const clients = await self.clients.matchAll();
                                clients.forEach(client => {
                                    client.postMessage({
                                        type: 'LOCALE_UPDATED',
                                        path: url.pathname
                                    });
                                });
                            }
                        }
                    }).catch(() => {
                        // Revalidation failed, continue with cached version
                    });
                    
                    return cachedResponse;
                }
                
                // No cached version, fetch from network
                console.log(`[SW] Fetching locale from network: ${url.pathname}`);
                const response = await fetch(event.request);
                
                if (response && response.ok) {
                    await cache.put(event.request, response.clone());
                }
                
                return response;
            })()
        );
        return;
    }
    
    // For bundle assets (JS/CSS), use stale-while-revalidate strategy
    if (isBundleAsset(url)) {
        event.respondWith(
            (async () => {
                const cache = await caches.open(CACHE_NAME);
                const cachedResponse = await cache.match(event.request);
                
                if (cachedResponse) {
                    console.log(`[SW] Serving cached bundle (stale-while-revalidate): ${url.pathname}`);
                    
                    // Start revalidation in the background
                    fetch(event.request).then(async (response) => {
                        if (response && response.ok) {
                            // Use content hashing to detect changes
                            const cachedHash = await cachedResponse.clone().text().then(t => t.hashCode());
                            const newHash = await response.clone().text().then(t => t.hashCode());
                            
                            if (cachedHash !== newHash) {
                                console.log(`[SW] Bundle updated: ${url.pathname}`);
                                await cache.put(event.request, response);
                                
                                // Notify clients to reload
                                const clients = await self.clients.matchAll();
                                clients.forEach(client => {
                                    client.postMessage({
                                        type: 'ASSET_UPDATED',
                                        path: url.pathname
                                    });
                                });
                            }
                        }
                    }).catch(() => {
                        // Revalidation failed, continue with cached version
                    });
                    
                    return cachedResponse;
                }
                
                // No cached version, fetch from network
                console.log(`[SW] Fetching bundle from network: ${url.pathname}`);
                const response = await fetch(event.request);
                
                if (response && response.ok) {
                    await cache.put(event.request, response.clone());
                }
                
                return response;
            })()
        );
        return;
    }
    
    // For sample files, use cache-first with network fallback
    if (isSampleRequest(url)) {
        event.respondWith(
            caches.match(event.request)
                .then(async (response) => {
                    if (response) {
                        console.log(`[SW] Serving cached sample: ${url.pathname}`);
                        return response;
                    }
                    console.log(`[SW] Fetching sample from network: ${url.pathname}`);
                    const networkResponse = await fetch(event.request);
                    
                    // Cache the response if successful
                    if (networkResponse && networkResponse.ok) {
                        const cache = await caches.open(CACHE_NAME);
                        await cache.put(event.request, networkResponse.clone());
                    }
                    
                    return networkResponse;
                })
        );
        return;
    }
    
    // For all other assets, use cache-first strategy
    event.respondWith(
        caches.match(event.request)
            .then(async (response) => {
                if (response) {
                    console.log(`[SW] Serving cached asset: ${url.pathname}`);
                    return response;
                }
                console.log(`[SW] Fetching asset from network: ${url.pathname}`);
                const networkResponse = await fetch(event.request);
                
                // Cache the response if successful
                if (networkResponse && networkResponse.ok) {
                    const cache = await caches.open(CACHE_NAME);
                    await cache.put(event.request, networkResponse.clone());
                }
                
                return networkResponse;
            })
    );
});

// Message event - handle messages from the client
self.addEventListener('message', (event) => {
    if (!event.source || !(event.source instanceof Client)) {
        return;
    }
    
    const client = event.source;
    const data = event.data;
    
    if (!data || !data.type) {
        return;
    }
    
    switch (data.type) {
        case 'SKIP_WAITING':
            self.skipWaiting();
            client.postMessage({
                type: 'SKIP_WAITING_ACK'
            });
            break;
            
        case 'CACHE_FILE':
            (async () => {
                const { url, content } = data;
                try {
                    const cache = await caches.open(CACHE_NAME);
                    console.log(`[SW] Caching file: ${url}`);
                    await cache.put(url, new Response(content));
                    client.postMessage({
                        type: 'CACHE_FILE_ACK',
                        url
                    });
                } catch (e) {
                    console.error(`[SW] Failed to cache file: ${url}`, e);
                }
            })();
            break;
            
        case 'DELETE_CACHE':
            (async () => {
                try {
                    await caches.delete(data.cacheName);
                    console.log(`[SW] Deleted cache: ${data.cacheName}`);
                } catch (e) {
                    console.error(`[SW] Failed to delete cache: ${data.cacheName}`, e);
                }
            })();
            break;
            
        case 'CHECK_UPDATE':
            (async () => {
                try {
                    // Check for update by trying to fetch a version file or index.html
                    const response = await fetch('/version.txt', { cache: 'no-store' });
                    if (response.ok) {
                        const version = await response.text();
                        if (version !== CACHE_VERSION) {
                            client.postMessage({
                                type: 'UPDATE_AVAILABLE',
                                version
                            });
                        } else {
                            client.postMessage({
                                type: 'NO_UPDATE'
                            });
                        }
                    } else {
                        // No version file, assume no update mechanism
                        client.postMessage({
                            type: 'NO_UPDATE'
                        });
                    }
                } catch (e) {
                    // If we can't check, assume no update
                    client.postMessage({
                        type: 'NO_UPDATE'
                    });
                }
            })();
            break;
            
        case 'GET_CACHE_VERSION':
            client.postMessage({
                type: 'CACHE_VERSION',
                version: CACHE_VERSION
            });
            break;
    }
});

// Background sync for saving files when offline
self.addEventListener('sync', (event) => {
    if (event.tag === 'save-file') {
        event.waitUntil(
            (async () => {
                console.log('[SW] Background sync for save-file');
                // This would be implemented with IndexedDB sync
                // For now, just log the event
            })()
        );
    }
});

// Push notification for update available
self.addEventListener('push', (event) => {
    if (event.data) {
        const data = event.data.json();
        if (data.type === 'UPDATE_AVAILABLE') {
            event.waitUntil(
                self.registration.showNotification('mml2vgm IDE Update Available', {
                    body: 'A new version of the IDE is available. Refresh to update.',
                    data: {
                        url: '/'
                    }
                })
            );
        }
    }
});

// Notification click handler
self.addEventListener('notificationclick', (event) => {
    event.notification.close();
    
    if (event.notification.data && event.notification.data.url) {
        event.waitUntil(
            self.clients.matchAll().then((clients) => {
                if (clients.length > 0) {
                    clients[0].navigate(event.notification.data.url);
                } else {
                    self.clients.openWindow(event.notification.data.url);
                }
            })
        );
    }
});

console.log(`[SW v${CACHE_VERSION}] Service Worker loaded`);
