// Cache version — increment on every deploy so the old cache gets cleared
const CACHE_NAME = 'zsozso-v2';

const LOG = (...args) => console.log(`[SW ${CACHE_NAME}]`, ...args);
const ERR = (...args) => console.error(`[SW ${CACHE_NAME}]`, ...args);

LOG('Script evaluated');

// We don't use a pre-cache list because Dioxus generates hashed filenames
// (e.g. zsozso-dxhABC123.js) that change with every build.
// Instead we cache at runtime: files are cached on first load.

self.addEventListener('message', event => {
    LOG('Message received:', event.data);
    if (event.data && event.data.type === 'GET_VERSION') {
        event.ports[0].postMessage({ version: CACHE_NAME });
        LOG('Replied with version:', CACHE_NAME);
    }
});

self.addEventListener('install', event => {
    LOG('Install event — calling skipWaiting()');
    // Activate immediately, don't wait for the old SW to stop
    self.skipWaiting();
});

self.addEventListener('activate', event => {
    LOG('Activate event — cleaning old caches');
    // Delete old cache versions
    event.waitUntil(
        caches.keys().then(keys => {
            const old = keys.filter(k => k !== CACHE_NAME);
            LOG('Existing caches:', keys, '| Deleting:', old);
            return Promise.all(old.map(k => caches.delete(k)));
        }).then(() => {
            LOG('Old caches deleted, calling clients.claim()');
            return self.clients.claim();
        })
    );
});

self.addEventListener('fetch', event => {
    const url = new URL(event.request.url);

    // Navigation requests (HTML pages) → network-first
    // This way index.html always refreshes when there is a network connection
    if (event.request.mode === 'navigate') {
        LOG('NAVIGATE fetch:', url.pathname);
        event.respondWith(
            fetch(event.request)
                .then(response => {
                    LOG('NAVIGATE network response:', response.status, url.pathname);
                    if (response.status === 200) {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(c => c.put(event.request, clone));
                        LOG('NAVIGATE cached fresh copy:', url.pathname);
                    }
                    return response;
                })
                .catch(err => {
                    ERR('NAVIGATE network failed:', err.message || err, url.pathname);
                    return caches.match(event.request)
                        .then(cached => {
                            if (cached) {
                                LOG('NAVIGATE serving from cache:', url.pathname);
                                return cached;
                            }
                            LOG('NAVIGATE fallback to index.html');
                            return caches.match('index.html');
                        });
                })
        );
        return;
    }

    // Hashed assets (.js, .wasm) → cache-first
    // Their content never changes (guaranteed by the hash), so
    // it's enough to download once and always serve from cache afterwards.
    const isCacheableAsset = /\.(js|wasm|css|png|jpg|svg|ico|woff2?)$/.test(url.pathname);

    if (isCacheableAsset) {
        event.respondWith(
            caches.match(event.request).then(cached => {
                if (cached) {
                    LOG('ASSET cache-hit:', url.pathname);
                    return cached;
                }
                LOG('ASSET cache-miss, fetching:', url.pathname);
                return fetch(event.request).then(response => {
                    LOG('ASSET network response:', response.status, url.pathname);
                    if (response.status === 200) {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(c => c.put(event.request, clone));
                    }
                    return response;
                }).catch(err => {
                    ERR('ASSET fetch failed:', err.message || err, url.pathname);
                    throw err;
                });
            })
        );
        return;
    }

    // Everything else (API calls, manifest.json, etc.) → network-only, fallback to cache
    LOG('OTHER fetch:', url.pathname);
    event.respondWith(
        fetch(event.request)
            .then(response => {
                LOG('OTHER network response:', response.status, url.pathname);
                if (response.status === 200) {
                    const clone = response.clone();
                    caches.open(CACHE_NAME).then(c => c.put(event.request, clone));
                }
                return response;
            })
            .catch(err => {
                ERR('OTHER fetch failed, trying cache:', err.message || err, url.pathname);
                return caches.match(event.request);
            })
    );
});
