# todo:

## simple steps:
- setup a gun server
- debug gundb and aes implementation
- understand and debug the sw.js . target to find the PWA App refreshing issue

## known issues:
#### (not yet solved, but it isn't mandant to correct it):
- The status not changing - partially - if I change the language and we are in an async function

## bigger steps:
- a good graphics design (styles...?)
- RWA Page (and logic)

# for dev's:
#### (rarely updated - we are at the beginning):

## Architecture

The application targets **PWA (Progressive Web App) only** — all code compiles to WebAssembly and runs in the browser. There are no desktop or native feature flags; the single `web` feature is the default. Platform differences (clipboard, storage, timers) use browser APIs directly.

```
src/
├── main.rs                  # Entry point — Dioxus web launch
├── i18n.rs                  # Language enum (English, Hungarian)
├── ledger/
│   ├── mod.rs               # Ledger trait — abstract blockchain interface
│   ├── stellar.rs           # Stellar implementation (Horizon API, XDR, ed25519)
│   ├── sc/
│   │   ├── mod.rs           # SmartContract trait — Soroban invoke helpers
│   │   ├── zsozso_sc.rs     # ZsozsoSc — concrete Zsozso contract bindings
│   │   └── i18n/            # Smart contract i18n
│   └── i18n/
│       ├── mod.rs           # LedgerI18n trait — ledger error/status messages
│       ├── english.rs       # English implementation
│       └── hungarian.rs     # Hungarian implementation
├── db/
│   ├── mod.rs               # Db trait — abstract graph database interface
│   ├── gundb.rs             # GUN.js bridge (via window.__gun_bridge)
│   ├── sea.rs               # SEA crypto bridge (via window.__sea_bridge)
│   └── i18n/                # Database i18n
├── store/
│   ├── mod.rs               # Store trait — abstract secret storage interface
│   ├── local_storage.rs     # Browser localStorage implementation
│   ├── indexed_db.rs        # IndexedDB implementation (encrypted secret storage)
│   ├── passkey.rs           # Passkey/WebAuthn bridge — init, verify, encrypt/decrypt via PRF
│   └── i18n/
│       ├── mod.rs           # StoreI18n trait — storage error messages
│       ├── english.rs       # English implementation
│       └── hungarian.rs     # Hungarian implementation
└── ui/
    ├── mod.rs               # Dioxus UI entry — app() component
    ├── clipboard.rs         # Clipboard — navigator.clipboard API
    ├── actions.rs           # Async UI actions (submit tx, generate keypair, etc.)
    ├── state.rs             # Reactive wallet state (signals)
    ├── controller.rs        # AppController — bridges state ↔ actions
    ├── status.rs            # TxStatus enum
    ├── view.rs              # Main view layout, auth gate, tab bar
    ├── qr_scanner.rs        # QR scanner — calls wascan JS bridge from Rust/WASM
    ├── tabs/
    │   ├── mod.rs           # Tab enum (Home, Networking, Info, Settings)
    │   ├── home.rs          # Home tab — welcome screen
    │   ├── networking.rs    # Networking tab — Ping contract, Scan QR code
    │   ├── info.rs          # Info tab — public key QR code display
    │   └── settings.rs      # Settings tab — key management, network/language toggle
    └── i18n/
        ├── mod.rs           # UiI18n trait — all UI-facing strings
        ├── english.rs       # English implementation
        └── hungarian.rs     # Hungarian implementation

assets/
├── gun_bridge.js            # GUN.js ↔ Rust bridge (window.__gun_bridge)
├── sea_bridge.js            # SEA crypto ↔ Rust bridge (window.__sea_bridge)
├── passkey_bridge.js        # WebAuthn Passkey + Web Crypto bridge (window.__passkey_bridge)
├── qr_scanner_bridge.js     # QR scanner bridge using wascan (window.__qr_scanner_bridge)
├── manifest.json            # PWA manifest
├── sw.js                    # Service Worker for offline caching
├── icon-192.png             # PWA icon 192×192
└── icon-512.png             # PWA icon 512×512
```

### Core Traits

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

### Internationalization (i18n) Traits

Every user-facing string in the application is abstracted behind i18n traits, with factory functions selecting the correct implementation based on the active `Language`. Each module owns its own i18n layer:

| Trait | Module | Purpose | Implementations |
|-------|--------|---------|-----------------|
| `UiI18n` | `ui/i18n` | All UI-facing strings — button labels, status messages, placeholders, format helpers | `EnglishUi`, `HungarianUi` |
| `LedgerI18n` | `ledger/i18n` | Blockchain operation messages — faucet, Horizon, XDR, and transaction errors/statuses | `EnglishLedger`, `HungarianLedger` |
| `StoreI18n` | `store/i18n` | Secret storage messages — save/load/storage errors | `EnglishStore`, `HungarianStore` |
| `ScI18n` | `ledger/sc/i18n` | Smart contract messages — RPC, simulation, transaction status | (per-language) |

**Adding a new language** requires three steps:

1. Add a variant to the `Language` enum in `src/i18n.rs`
2. Create a new implementation file in each `i18n/` directory (`ui/i18n/`, `ledger/i18n/`, `store/i18n/`, `ledger/sc/i18n/`)
3. Register the new implementation in each factory function (`ui_i18n()`, `ledger_i18n()`, `store_i18n()`, `sc_i18n()`)

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

# Release build
dx build --release --platform web --features web
# Output in target/dx/zsozso/release/web/public/

# Serve the release build locally
python3 -m http.server 8080 -d target/dx/zsozso/release/web/public/
# Or with npx:
npx serve target/dx/zsozso/release/web/public/ -l 8080
```
## Deployment

```
/var/www/html/app/
├── index.html (from build output)
├── sw.js (from assets/ or from project root)
├── manifest.json (from assets/ or from project root)
├── icon-192.png (from assets/ or from project root)
├── icon-512.png (from assets/ or from project root)
├── <... other files ...> (from assets/ or from project root)
└── assets/
    ├── zsozso-dxh*.js (from build output)
    └── zsozso_bg-dxh*.wasm (from build output)

A change of the CACHE_NAME in the sw.js at every deploy a need 
(the browser will reread the cache at the user...):
...
const CACHE_NAME = 'zsozso-v2'; => 'zsozso-v3' ...
...
```

```bash
# I use an intermediate directory to collect the deployment files in dist/app
# Later it is possible to use python or npx to serve the pages from dist/
npx serve dist/ -l 8080

# After the launch the App is reachable with this link: http://localhost:8080/app/ in a browser 
```

### Feature Flag

| Flag | Description |
|------|-------------|
| `web` | Browser PWA via WASM (default) — Dioxus web, navigator.clipboard, gloo-timers, browser localStorage |

## Mobile Deployment (PWA)

A PWA allows users to "install" the app directly from the browser to their home screen — no app store required. It works on iOS Safari, Android Chrome, and desktop browsers.

### Setup

The project includes PWA support out of the box via the following files:

- **`assets/manifest.json`** — PWA manifest with app metadata, icons, and display settings
- **`assets/sw.js`** — Service Worker for offline caching of assets
- **`assets/pwa-install.js`** — Install prompt handling for Android Chrome
- **`index.html`** — Includes PWA meta tags, manifest link, and service worker registration
- **`assets/icon-192.png` and `assets/icon-512.png`** — App icons (placeholders, replace with custom designs)

### How Users Install It

- **Android Chrome** — use Menu (⋮) → "Add to Home screen"
- **iOS Safari** — Share (↑) → "Add to Home Screen"
- **Desktop Chrome/Edge** — Address bar install icon or Menu → "Install Zsozso Wallet"

### Offline Support

The service worker caches critical assets on first visit, allowing the app to work offline. When online, it automatically fetches fresh content while serving cached versions as fallback.
