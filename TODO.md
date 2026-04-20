# todo:

## simple steps:
- debug gundb and aes implementation

## known issues:
#### (not yet solved, but it isn't mandant to correct it):
- The status not changing - partially - if I change the language and we are in an async function

## bigger steps:
- a good graphics design (styles...(learn it) use components!)
- RWA Page (and logic)

# for dev's:
#### (rarely updated - we are at the beginning):

The architecture and critical logic of this project are the results of human-led AI-assisted engineering. This unique workflow ensures industrial-grade reliability and accelerated deployment.

## Architecture (it needs to be updated)

The application targets **PWA (Progressive Web App) only** — all code compiles to WebAssembly and runs in the browser. There are no desktop or native feature flags; the single `web` feature is the default. Platform differences (clipboard, storage, timers) use browser APIs directly.

```
src/
├── main.rs                  # Entry point — Dioxus web launch
├── i18n.rs                  # Language enum (English, Hungarian, French, German, Spanish)
├── ledger/
│   ├── mod.rs               # Ledger trait — abstract blockchain interface
│   ├── stellar.rs           # Stellar implementation (Horizon API, XDR, ed25519)
│   ├── sc/
│   │   ├── mod.rs           # SmartContract trait — Soroban invoke helpers
│   │   ├── zsozso_sc.rs     # ZsozsoSc — concrete Zsozso contract bindings
│   │   └── i18n/            # ScI18n trait + per-language implementations
│   └── i18n/                # LedgerI18n trait + per-language implementations
├── db/
│   ├── mod.rs               # Db trait — abstract graph database interface
│   ├── gundb.rs             # GUN.js bridge (via window.__gun_bridge)
│   ├── sea.rs               # SEA crypto bridge (via window.__sea_bridge)
│   └── i18n/                # DbI18n trait + per-language implementations
├── store/
│   ├── mod.rs               # Store trait — abstract secret storage interface
│   ├── local_storage.rs     # Browser localStorage implementation
│   ├── indexed_db.rs        # IndexedDB implementation (encrypted secret storage)
│   ├── passkey.rs           # Passkey/WebAuthn bridge — init, verify, encrypt/decrypt via PRF
│   └── i18n/                # StoreI18n trait + per-language implementations
└── ui/
    ├── mod.rs               # Dioxus UI entry — app() component
    ├── clipboard.rs         # Clipboard — navigator.clipboard API
    ├── actions.rs           # Async UI actions (submit tx, generate keypair, etc.)
    ├── state.rs             # Reactive wallet state (signals)
    ├── controller.rs        # AppController — bridges state ↔ actions
    ├── status.rs            # TxStatus enum
    ├── toast.rs             # UpdateNotification — shows "update available" toast when SW detects a new version
    ├── view.rs              # Main view layout, auth gate, tab bar
    ├── qr_scanner.rs        # QR scanner — calls wascan JS bridge from Rust/WASM
    ├── tabs/
    │   ├── mod.rs           # Tab enum (Home, Networking, Info, Log, Settings)
    │   ├── home.rs          # Home tab — welcome screen
    │   ├── networking.rs    # Networking tab — Ping contract, Scan QR code
    │   ├── info.rs          # Info tab — public key QR code display
    │   ├── log.rs           # Log tab — log viewer with refresh, upload, and clear
    │   └── settings.rs      # Settings tab — key management, network/language toggle
    └── i18n/                # UiI18n trait — all UI-facing strings
        ├── mod.rs           # Trait definition + factory function
        ├── english.rs       # (All i18n/ dirs follow the same pattern:
        ├── hungarian.rs     #  mod.rs with trait + factory, and one
        ├── french.rs        #  implementation file per language)
        ├── german.rs
        └── spanish.rs

assets/
├── gun_bridge.js            # GUN.js ↔ Rust bridge (window.__gun_bridge)
├── sea_bridge.js            # SEA crypto ↔ Rust bridge (window.__sea_bridge)
├── passkey_bridge.js        # WebAuthn Passkey + Web Crypto bridge (window.__passkey_bridge)
├── qr_scanner_bridge.js     # QR scanner bridge using wascan (window.__qr_scanner_bridge)
├── log_bridge.js            # In-app log ring buffer — captures console.log/error + SW logs + upload
├── manifest.json            # PWA manifest
├── icon-192.png             # PWA icon 192×192
└── icon-512.png             # PWA icon 512×512

(root — static files and build tooling)
├── sw.js                    # Base Service Worker — offline caching, update detection, log forwarding
├── gun.js                   # GUN library
├── sea.js                   # GUN SEA crypto library
├── wascan.js                # QR scanner WASM module (JS glue)
├── wascan_bg.wasm           # QR scanner WASM binary
├── gun_bridge.js            # GUN bridge (copied to dist/ by build.sh)
├── sea_bridge.js            # SEA bridge (copied to dist/ by build.sh)
├── passkey_bridge.js        # Passkey bridge (copied to dist/ by build.sh)
├── qr_scanner_bridge.js     # QR scanner bridge (copied to dist/ by build.sh)
├── log_bridge.js            # Log bridge (copied to dist/ by build.sh)
├── bundle_sw.js             # Node.js bundling script — creates offline PWA deployment
└── build.sh                 # Build pipeline: dx build → stamp CACHE_NAME → copy statics → bundle

server/
├── nginx.conf               # Main nginx configuration
├── zsozso.info.conf         # Site config — PWA hosting, SW headers, upload endpoint
└── log_upload_server.py     # Lightweight log upload HTTP helper (port 9977, 50 MB quota)
```

### Core Traits (it needs to be updated)

| Trait | Purpose | Implementation |
|-------|---------|----------------|
| `Ledger` | Blockchain operations (keygen, signing, submitting) | `StellarLedger` |
| `SmartContract` | Soroban contract invocation (simulate, sign, send, poll) | `ZsozsoSc` |
| `Store` | Secure secret persistence | `IndexedDbStore` (encrypted via passkey PRF) |
| `Db` | Graph database (GUN) | `GunDb` (delegates to gun_bridge.js) |
| `Sea` | GUN SEA crypto operations | `GunSea` (delegates to sea_bridge.js) |

### JS Bridges

The WASM application communicates with browser-only APIs and external JS libraries through bridge objects on `window`:

| Bridge | JS file | Rust module | Purpose |
|--------|---------|-------------|---------|
| `__gun_bridge` | `gun_bridge.js` | `db::gundb` | GUN decentralised database |
| `__sea_bridge` | `sea_bridge.js` | `db::sea` | GUN SEA crypto (keypair, sign, verify, encrypt, decrypt) |
| `__passkey_bridge` | `passkey_bridge.js` | `store::passkey` | WebAuthn registration/auth, PRF key derivation, AES-GCM encrypt/decrypt |
| `__qr_scanner_bridge` | `qr_scanner_bridge.js` | `ui::qr_scanner` | Camera-based QR code scanning via wascan (loaded from CDN) |
| `__zsozso_log` | `log_bridge.js` | `ui::tabs::log` | In-app log ring buffer (get, clear, upload) |

### Log Upload

The Log tab has an **Upload** button that POSTs the current in-app log buffer to the server for remote debugging:

- **Client side**: `log_bridge.js` exposes `window.__zsozso_log.upload()` → `POST /app/upload_log` with `text/plain` body
- **Nginx**: The `/app/upload_log` location proxies to a lightweight Python helper on `127.0.0.1:9977`; `client_max_body_size 1m` limits each upload to 1 MB
- **Server helper**: `server/log_upload_server.py` writes timestamped `.log` files to `/var/www/html/app/uploads/` and enforces a 50 MB directory quota (oldest files are deleted)

To start the upload helper on the server:
```bash
python3 /path/to/server/log_upload_server.py &
# Env vars: UPLOAD_DIR (default /var/www/html/app/uploads), MAX_DIR_MB (default 50), LISTEN_PORT (default 9977)
```

### Service Worker Update Strategy

The SW (`sw.js`) handles offline caching and version management:

- **`index.html`** registers the SW with `updateViaCache: 'none'` — the browser always fetches `sw.js` from the network, bypassing HTTP cache
- Every page load triggers a byte-comparison check against the server copy
- When the browser detects a change, the new SW calls `skipWaiting()` + `clients.claim()` to take control immediately
- The `controllerchange` event in `index.html` auto-reloads the page once (guarded by `_swRefreshing` flag to prevent loops)
- **`CACHE_NAME`** in `sw.js` must be incremented on every deploy (e.g. `zsozso-v2` → `zsozso-v3`) so the old cache is purged (the build.sh do it)
- A toast (in `index.html`) polls `window.__ZSOZSO_UPDATE_READY` and shows a manual "Refresh" button when an update is detected
- The SW also forwards its own log entries to the main page via `postMessage`, visible in the Log tab

### Bundled Offline Deployment (bundle.js)

For static hosting, `bundle.js` creates a fully
self-contained deployment from the `dist/` build output. The result is just
two physical files: `index.html` and `sw.js`.
I will be served some icons, manifest file with it.

**How it works (JSON-in-HTML):**

1. All files in `dist/` are gzip-compressed and base64-encoded
2. A bootloader `index.html` is generated that registers the SW and shows a
   loading spinner
3. The actual app `index.html` (from `dist/`) is itself gzip+base64 encoded
   and embedded in the bootloader — it contains all the asset entries
4. On SW `install`, assets are decoded from the embedded JSON map and stored
   in CacheStorage
5. On `activate`, the SW intercepts all fetch requests and serves from cache
6. PWA metadata (manifest, icons) is embedded as data URIs in the bootloader
   so PWA install works even before the SW activates

**Build pipeline (`build.sh`):**

```bash
# 1. Stamp CACHE_NAME with date+time+commit into sw.js
# 2. dx build --release --platform web
# 3. Stage build output to dist/zsozso-dioxus/
# 4. Copy root static files (sw.js, gun.js, sea.js, wascan.js, ...) into dist/
# 5. Run: node bundle_sw.js dist/zsozso-dioxus -j -dioxus -logging
# 6. Output: deploy/zsozso-dioxus/{index.html, sw.js}
```

### Internationalization (i18n) Traits (it needs to be updated)

Every user-facing string in the application is abstracted behind i18n traits, with factory functions selecting the correct implementation based on the active `Language`. Each module owns its own i18n layer:

| Trait | Module | Purpose | Languages |
|-------|--------|---------|-----------|
| `UiI18n` | `ui/i18n` | All UI-facing strings — button labels, status messages, placeholders, format helpers | EN, HU, FR, DE, ES |
| `LedgerI18n` | `ledger/i18n` | Blockchain operation messages — faucet, Horizon, XDR, and transaction errors/statuses | EN, HU, FR, DE, ES |
| `StoreI18n` | `store/i18n` | Secret storage messages — save/load/storage errors | EN, HU, FR, DE, ES |
| `ScI18n` | `ledger/sc/i18n` | Smart contract messages — RPC, simulation, transaction status | EN, HU, FR, DE, ES |
| `DbI18n` | `db/i18n` | Database messages — GUN/SEA errors | EN, HU, FR, DE, ES |

**Adding a new language** requires three steps:

1. Add a variant to the `Language` enum in `src/i18n.rs`
2. Create a new implementation file in each `i18n/` directory (`ui/i18n/`, `ledger/i18n/`, `store/i18n/`, `db/i18n/`, `ledger/sc/i18n/`)
3. Register the new implementation in each factory function (`ui_i18n()`, `ledger_i18n()`, `store_i18n()`, `sc_i18n()`, `db_i18n()`)

## Target Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| Web (WASM/PWA) | ✅ Supported | Primary target — installable via browser |
| iOS Safari (PWA) | ✅ Supported | Share → "Add to Home Screen" |
| Android Chrome (PWA) | ✅ Supported | Menu → "Add to Home screen" |
| Desktop Chrome/Edge (PWA) | ✅ Supported | Address bar install icon |

## Prerequisites

### Web (WASM)

```bash
# Rust toolchain (if not installed yet)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# Dioxus CLI (for dx serve / dx build)
cargo install dioxus-cli
```

## Build & Run

```bash
# Clone the repository
git clone https://github.com/ifinta/zsozso-dioxus.git
cd zsozso-dioxus

# Development server with hot-reload
dx serve --platform web

# Full release build + bundled offline deployment
./build.sh
# Output:
#   dist/zsozso-dioxus/   — traditional deployment (all files)
#   deploy/zsozso-dioxus/ — bundled offline deployment (index.html + sw.js only)

# Serve the bundled deployment locally:
npx serve deploy/ -l 8080
# → http://localhost:8080/zsozso-dioxus/

# Serve the traditional deployment locally:
npx serve dist/ -l 8080
# → http://localhost:8080/zsozso-dioxus/
```
## Deployment

See the "Bundled Offline Deployment" section above for details on `bundle.js`.

### Traditional (nginx / static server)

Copy the contents of `dist/app/` to your web server root.

### Feature Flag

| Flag | Description |
|------|-------------|
| `web` | Browser PWA via WASM (default) — Dioxus web, navigator.clipboard, gloo-timers, browser localStorage |

## Mobile Deployment (PWA)

A PWA allows users to "install" the app directly from the browser to their home screen — no app store required. It works on iOS Safari, Android Chrome, and desktop browsers.

### Setup

The project includes PWA support out of the box via the following files:

- **`manifest.json`** (in `assets/`) — PWA manifest with app metadata, icons, and display settings
- **`sw.js`** (in repo root) — Base Service Worker for offline caching of assets
- **`index.html`** — Includes PWA meta tags, manifest link, and service worker registration
- **`icon-192.png` and `icon-512.png`** (in `assets/`) — App icons
- **`bundle_sw.js`** (in repo root) — Node.js script that creates bundled offline deployments

### How Users Install It

- **Android Chrome** — use Menu (⋮) → "Add to Home screen"
- **iOS Safari** — Share (↑) → "Add to Home Screen"
- **Desktop Chrome/Edge** — Address bar install icon or Menu → "Install Zsozso Wallet"

### Offline Support

In **traditional mode** (nginx deployment), the SW caches assets on first visit
and serves from cache on subsequent requests, with network fallback for updates.
