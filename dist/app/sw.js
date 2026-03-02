// Cache version — increment on every deploy so the old cache gets cleared
const CACHE_NAME = 'zsozso-v0.19990-';

// ── SW-side log ring buffer (max 100) ──
const _swLogBuffer = [];
const _SW_LOG_MAX = 100;

function _ts() {
    const d = new Date();
    return d.toLocaleTimeString('en-GB', { hour12: false }) + '.' +
        String(d.getMilliseconds()).padStart(3, '0');
}

const LOG = (...args) => {
    const text = args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ');
    const entry = _ts() + ' ' + CACHE_NAME + ' [SW] ' + text;
    _swLogBuffer.push(entry);
    if (_swLogBuffer.length > _SW_LOG_MAX) _swLogBuffer.shift();
    console.log(`[SW ${CACHE_NAME}]`, ...args);
    _forward(entry);
};
const ERR = (...args) => {
    const text = args.map(a => typeof a === 'string' ? a : JSON.stringify(a)).join(' ');
    const entry = _ts() + ' ' + CACHE_NAME + ' [SW ERR] ' + text;
    _swLogBuffer.push(entry);
    if (_swLogBuffer.length > _SW_LOG_MAX) _swLogBuffer.shift();
    console.error(`[SW ${CACHE_NAME}]`, ...args);
    _forward(entry);
};

// Forward log lines to the main page so the in-app Log tab can display them
function _forward(text) {
    self.clients.matchAll({ type: 'window' }).then(clients => {
        clients.forEach(c => c.postMessage({ type: '__ZSOZSO_SW_LOG', text: text }));
    });
}

LOG('Script evaluated');

// We don't use a pre-cache list because Dioxus generates hashed filenames
// (e.g. zsozso-dxhABC123.js) that change with every build.
// Instead we cache at runtime: files are cached on first load.

self.addEventListener('message', event => {
    if (event.data && event.data.type === 'GET_LOGS') {
        event.ports[0].postMessage({ logs: _swLogBuffer.slice() });
        return;
    }
    if (event.data && event.data.type === 'CLEAR_LOGS') {
        _swLogBuffer.length = 0;
        return;
    }
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
    // 1. Claim clients IMMEDIATELY
    event.waitUntil(self.clients.claim());

    // 2. Then proceed with cleanup and notifications
    // Delete old cache versions
    event.waitUntil(
        caches.keys().then(keys => {
            const old = keys.filter(k => k !== CACHE_NAME);
            LOG('Existing caches:', keys, '| Deleting:', old);
            // Track whether this is a genuine update (old caches exist)
            const isUpdate = old.length > 0;
            return Promise.all(old.map(k => caches.delete(k))).then(() => isUpdate);
        }).then(isUpdate => {
            LOG('Old caches deleted, calling clients.claim()');
            return self.clients.claim().then(() => isUpdate);
        }).then(isUpdate => {
            // Only notify clients to reload when we actually replaced an older version.
            // On iOS the SW can be terminated and re-activated by the OS —
            // that is NOT an update and must not trigger a reload loop.
            if (isUpdate) {
                return self.clients.matchAll({ type: 'window' }).then(clients => {
                    clients.forEach(c => c.postMessage({ type: '__ZSOZSO_SW_UPDATED' }));
                    LOG('Update detected — notified', clients.length, 'client(s) to reload');
                });
            } else {
                LOG('No old caches found — not an update, skipping reload notification');
            }
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
    // IMPORTANT: exclude sw.js itself — the browser must always fetch it
    // fresh so it can detect updates and trigger the install event.
    const isCacheableAsset =
        /\.(js|wasm|css|png|jpg|svg|ico|woff2?)$/.test(url.pathname) &&
        !url.pathname.endsWith('/sw.js') &&
        !url.pathname.endsWith('/log_bridge.js');

    if (isCacheableAsset) {
        event.respondWith(
            caches.match(event.request).then(cached => {
                if (cached) return cached;

                // Not in cache, try network
                return fetch(event.request).then(response => {
                    if (response.status === 200) {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(c => c.put(event.request, clone));
                    }
                    return response;
                }).catch(err => {
                    // If network fails, try ANY cache version before giving up
                    return caches.match(event.request);
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
