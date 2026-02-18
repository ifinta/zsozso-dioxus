# Zsozso

The main application for the [Iceberg Protocol](https://ifinta.github.io/zsozso-webpage/) ecosystem, built with Rust and [Dioxus](https://dioxuslabs.com/). Runs as a native desktop app or in the browser via WebAssembly.

ZSOZSO is the utility token fueling the Iceberg Protocol — a decentralized hierarchical MLM infrastructure and message-bus architecture on the [Stellar](https://stellar.org/) blockchain. This app provides a wallet for key management, transaction signing, and network interaction.

## Features

- **Key Management** — Generate new Stellar keypairs or import existing secret keys
- **Secure Storage** — Desktop: OS credential manager (GNOME Keyring, macOS Keychain, Windows Credential Manager); Web: browser localStorage
- **Network Switching** — Toggle between Stellar Mainnet and Testnet with a single click
- **Language Switching** — Toggle between English and Hungarian UI at runtime
- **Transaction Signing** — Build and sign XDR transaction envelopes locally
- **Transaction Submission** — Submit signed transactions directly to Stellar Horizon
- **Testnet Faucet** — Activate test accounts via Friendbot (Testnet only)
- **Clipboard Security** — Desktop: secret keys auto-cleared from clipboard after 30s; Web: uses navigator.clipboard API

## Architecture

The application is designed with trait-based abstraction layers so that no module depends on blockchain or platform specifics directly. Platform differences are handled via `#[cfg]` gates and Cargo feature flags (`desktop` / `web`).

```
src/
├── main.rs                  # Entry point — desktop or web launch via cfg
├── i18n.rs                  # Language enum (English, Hungarian)
├── ledger/
│   ├── mod.rs               # Ledger trait — abstract blockchain interface
│   ├── stellar.rs           # Stellar implementation (Horizon API, XDR, ed25519)
│   └── i18n/
│       ├── mod.rs           # LedgerI18n trait — ledger error/status messages
│       ├── english.rs       # English implementation
│       └── hungarian.rs     # Hungarian implementation
├── store/
│   ├── mod.rs               # Store trait — abstract secret storage interface
│   ├── keyring.rs           # Desktop: OS keyring (GNOME/macOS/Windows)
│   ├── local_storage.rs     # Web: browser localStorage
│   └── i18n/
│       ├── mod.rs           # StoreI18n trait — storage error messages
│       ├── english.rs       # English implementation
│       └── hungarian.rs     # Hungarian implementation
└── ui/
    ├── mod.rs               # Dioxus UI components and application logic
    ├── clipboard.rs         # Clipboard copy — arboard (desktop) / navigator.clipboard (web)
    └── i18n/
        ├── mod.rs           # UiI18n trait — all UI-facing strings
        ├── english.rs       # English implementation
        └── hungarian.rs     # Hungarian implementation
```

### Core Traits

| Trait | Purpose | Desktop Implementation | Web Implementation |
|-------|---------|----------------------|-------------------|
| `Ledger` | Blockchain operations (keygen, signing, submitting) | `StellarLedger` | `StellarLedger` (same) |
| `Store` | Secure secret persistence | `KeyringStore` (OS credential manager) | `LocalStorageStore` (browser localStorage) |

### Internationalization (i18n) Traits

Every user-facing string in the application is abstracted behind i18n traits, with factory functions selecting the correct implementation based on the active `Language`. Each module owns its own i18n layer:

| Trait | Module | Purpose | Implementations |
|-------|--------|---------|-----------------|
| `UiI18n` | `ui/i18n` | All UI-facing strings — button labels, status messages, placeholders, format helpers | `EnglishUi`, `HungarianUi` |
| `LedgerI18n` | `ledger/i18n` | Blockchain operation messages — faucet, Horizon, XDR, and transaction errors/statuses | `EnglishLedger`, `HungarianLedger` |
| `StoreI18n` | `store/i18n` | Secret storage messages — keyring save/load/storage errors | `EnglishStore`, `HungarianStore` |

**Adding a new language** requires three steps:

1. Add a variant to the `Language` enum in `src/i18n.rs`
2. Create a new implementation file in each `i18n/` directory (`ui/i18n/`, `ledger/i18n/`, `store/i18n/`)
3. Register the new implementation in each factory function (`ui_i18n()`, `ledger_i18n()`, `store_i18n()`)

## Target Platforms

| Platform | Status | Feature Flag |
|----------|--------|-------------|
| Linux (Debian/Ubuntu) | ✅ Supported | `desktop` (default) |
| macOS | 🔜 Planned | `desktop` |
| Windows | 🔜 Planned | `desktop` |
| Web (WASM) | ✅ Supported | `web` |
| iOS / Android (Capacitor) | 📦 Packagable | `web` (wrapped in native WebView) |
| iOS / Android (PWA) | 📲 Installable | `web` (no app store needed) |

## Prerequisites

### Linux (Debian/Ubuntu) — Desktop

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# System dependencies for Dioxus desktop and keyring
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  libdbus-1-dev \
  pkg-config \
  libssl-dev
```

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

### Desktop (default)

```bash
# Clone the repository
git clone https://github.com/ifinta/zsozso-dioxus.git
cd zsozso-dioxus

# Development build
cargo build
cargo run

# Release build (optimized, smaller binary)
cargo build --release
./target/release/zsozso
```

### Web (WASM)

```bash
# Development server with hot-reload
dx serve --platform web

# Release build
dx build --release --platform web
# Output in target/dx/zsozso/release/web/public/

# Serve the release build locally
python3 -m http.server 8080 -d target/dx/zsozso/release/web/public/
# Or with npx:
npx serve target/dx/zsozso/release/web/public/ -l 8080
```

### Feature Flags

| Flag | Description |
|------|-------------|
| `desktop` | Native desktop app (default) — Dioxus desktop, arboard clipboard, tokio runtime, OS keyring |
| `web` | Browser app via WASM — Dioxus web, navigator.clipboard, gloo-timers, browser localStorage |

> **Note:** The `desktop` and `web` features are mutually exclusive. Use `--no-default-features --features web` if building manually with `cargo` instead of `dx`.

## Mobile Deployment

The WASM web build can be deployed to iOS and Android without modifying any Rust code. Two approaches are available:

### Option A: Progressive Web App (PWA)

A PWA allows users to "install" the app directly from the browser to their home screen — no app store required. It works on iOS Safari, Android Chrome, and desktop browsers.

#### Setup

The project includes PWA support out of the box via the following files:

- **`assets/manifest.json`** — PWA manifest with app metadata, icons, and display settings
- **`assets/sw.js`** — Service Worker for offline caching of assets
- **`assets/pwa-install.js`** — Install prompt handling for Android Chrome
- **`index.html`** — Includes PWA meta tags, manifest link, and service worker registration
- **`assets/icon-192.png` and `assets/icon-512.png`** — App icons (placeholders, replace with custom designs)

The web UI includes a **📲 Install** button (visible only on compatible browsers when the app can be installed) and an iOS hint for Safari users.

#### How Users Install It

- **Android Chrome** — Automatic "Install app" banner or click the **📲 Install** button in the app, or use Menu (⋮) → "Add to Home screen"
- **iOS Safari** — Share (↑) → "Add to Home Screen" (iOS doesn't support automatic install prompts, but the app shows a hint)
- **Desktop Chrome/Edge** — Address bar install icon or Menu → "Install Zsozso Wallet"

#### Offline Support

The service worker caches critical assets on first visit, allowing the app to work offline. When online, it automatically fetches fresh content while serving cached versions as fallback.

### Option B: Capacitor (Native App Store Package)

[Capacitor](https://capacitorjs.com/) wraps the web build in a native WebView, producing a real `.apk` (Android) or `.ipa` (iOS) that can be submitted to the App Store / Play Store.

#### Prerequisites

```bash
# Node.js (v18+) and npm
# https://nodejs.org/

# Android Studio (for Android builds)
# https://developer.android.com/studio

# Xcode (for iOS builds, macOS only)
# https://developer.apple.com/xcode/
```

#### Setup & Build

```bash
# 1. Build the WASM release
dx build --release --platform web

# 2. Copy the output to a dedicated dist/ directory
cp -r target/dx/zsozso/release/web/public dist

# 3. Initialize Capacitor
npm init -y
npm install @capacitor/core @capacitor/cli
npx cap init zsozso com.ifinta.zsozso --web-dir dist

# 4. Add platforms
npx cap add android
npx cap add ios

# 5. Sync the web build into the native projects
npx cap sync

# 6. Open in native IDE
npx cap open android   # Opens Android Studio
npx cap open ios       # Opens Xcode (macOS only)
```

#### Update Workflow

After making changes and rebuilding the WASM output:

```bash
dx build --release --platform web
cp -r target/dx/zsozso/release/web/public dist
npx cap sync
```

> **Note:** Native device APIs (e.g. secure storage, biometrics) can be added via [Capacitor plugins](https://capacitorjs.com/docs/plugins). The browser `localStorage` store will work out of the box inside the Capacitor WebView.

### Comparison

| | PWA | Capacitor |
|---|---|---|
| **App Store / Play Store** | ❌ Not needed | ✅ Yes |
| **Offline support** | ✅ Service worker | ✅ Built-in |
| **Native APIs** | ⚠️ Limited | ✅ Plugins |
| **Installation** | "Add to Home Screen" | Download from store |
| **Updates** | Automatic | Store update |
| **Build complexity** | Very low | Moderate |
| **Rust code changes** | Not needed | Not needed |

## Related Repositories

- [zsozso-webpage](https://github.com/ifinta/zsozso-webpage) — Project website & whitepaper
- [ZSOZSO on Stellar Expert](https://stellar.expert/explorer/public/asset/ZSOZSO-GDZKLEYJ54QUIEYE4DUUOCIJDUS7R5MDW5MCAB3XTUGPJ3C7SSZJRQUC) — ZSOZSO asset on mainnet

## License

See repository for license details.