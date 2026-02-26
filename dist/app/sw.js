// Cache verzió — minden deploy-nál növeld, hogy a régi cache törlődjön
const CACHE_NAME = 'zsozso-v2';

// Nem használunk pre-cache listát, mert a Dioxus hash-elt fájlneveket generál
// (pl. zsozso-dxhABC123.js), amelyek minden build-nél változnak.
// Ehelyett runtime cache-elünk: a fájlok az első betöltéskor kerülnek cache-be.

self.addEventListener('install', event => {
    // Azonnal aktiválódjon, ne várjon a régi SW leállására
    self.skipWaiting();
});

self.addEventListener('activate', event => {
    // Régi cache verziók törlése
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

    // Navigációs kérések (HTML oldalak) → network-first
    // Így az index.html mindig frissül, ha van hálózat
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

    // Hash-elt asset-ek (.js, .wasm) → cache-first
    // Ezek tartalma soha nem változik (a hash garantálja), tehát
    // elég egyszer letölteni és utána mindig a cache-ből szolgálni.
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

    // Minden más (API hívások, manifest.json stb.) → network-only, fallback cache
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
