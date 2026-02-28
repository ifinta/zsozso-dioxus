// Cache version — increment on every deploy so the old cache gets cleared
const CACHE_NAME = 'zsozso-v0.18';

// We don't use a pre-cache list because Dioxus generates hashed filenames
// (e.g. zsozso-dxhABC123.js) that change with every build.
// Instead we cache at runtime: files are cached on first load.

self.addEventListener('install', event => {
    // Activate immediately, don't wait for the old SW to stop
    self.skipWaiting();
});

self.addEventListener('activate', event => {
    // Delete old cache versions
    event.waitUntil(
        caches.keys().then(keys =>
            Promise.all(
                keys.filter(k => k !== CACHE_NAME).map(k => caches.delete(k))
            )
        )
    );
    self.clients.claim();
});

self.addEventListener('fetch', event => {
    const url = new URL(event.request.url);

    // Navigation requests (HTML pages) → network-first
    // This way index.html always refreshes when there is a network connection
    if (event.request.mode === 'navigate') {
        event.respondWith(
            fetch(event.request)
                .then(response => {
                    if (response.status === 200) {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(c => c.put(event.request, clone));
                    }
                    return response;
                })
                .catch(() => caches.match(event.request)
                    .then(cached => cached || caches.match('index.html'))
                )
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
                if (cached) return cached;
                return fetch(event.request).then(response => {
                    if (response.status === 200) {
                        const clone = response.clone();
                        caches.open(CACHE_NAME).then(c => c.put(event.request, clone));
                    }
                    return response;
                });
            })
        );
        return;
    }

    // Everything else (API calls, manifest.json, etc.) → network-only, fallback to cache
    event.respondWith(
        fetch(event.request)
            .then(response => {
                if (response.status === 200) {
                    const clone = response.clone();
                    caches.open(CACHE_NAME).then(c => c.put(event.request, clone));
                }
                return response;
            })
            .catch(() => caches.match(event.request))
    );
});
