const CACHE_NAME = 'zsozso-v1';
const ASSETS_TO_CACHE = [
    '/',
    '/index.html',
    '/assets/dioxus/zsozso.js',
    '/assets/dioxus/zsozso_bg.wasm',
    '/manifest.json',
];

self.addEventListener('install', event => {
    event.waitUntil(
        caches.open(CACHE_NAME).then(cache => cache.addAll(ASSETS_TO_CACHE))
    );
    self.skipWaiting();
});

self.addEventListener('activate', event => {
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
    event.respondWith(
        caches.match(event.request).then(cached => {
            if (cached) return cached;
            return fetch(event.request).then(response => {
                if (response.status === 200) {
                    const clone = response.clone();
                    caches.open(CACHE_NAME).then(cache => cache.put(event.request, clone));
                }
                return response;
            });
        }).catch(() => {
            // Only fallback to index.html for navigation requests
            if (event.request.mode === 'navigate') {
                return caches.match('/index.html');
            }
            // For other requests, return a network error
            return new Response('Network error', { status: 408, statusText: 'Request Timeout' });
        })
    );
});
