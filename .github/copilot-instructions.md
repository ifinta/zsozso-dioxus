# Copilot Instructions — Zsozso

## Build & Run

```bash
# Dev server with hot-reload
dx serve --platform web

# Release build (stamps CACHE_NAME into sw.js, builds, stages to dist/app/)
./build.sh

# Dry run — show CACHE_NAME without building
./build.sh --dry
```

Prerequisites: `rustup target add wasm32-unknown-unknown` and `cargo install dioxus-cli`.

No tests exist in the codebase. No linter configuration beyond default `cargo check`.

## Architecture

Zsozso is the **main PWA app for the [Iceberg Protocol](https://zsozso.info)** — a decentralized hierarchical MLM infrastructure and message-bus on the Stellar blockchain. Built with **Dioxus 0.7** (Rust → WASM), it runs as a PWA only — no native/desktop target.

### Ecosystem

- **Live app**: https://zsozso.info/app/ — deployed on an nginx server, tested primarily on iPhone (Safari PWA) and by testers on various devices
- **Project website**: https://zsozso.info — whitepaper and protocol documentation ([source](https://github.com/ifinta/zsozso-webpage))
- **Smart contracts**: developed separately in https://github.com/ifinta/zsozso-sc (Soroban, first contract deployed on Stellar Testnet)
- **Remote log upload**: the server at zsozso.info accepts log uploads from any app user to `/app/upload_log` for remote debugging

### Module layout

- **`src/ui/`** — Dioxus components, state, controller, tabs
- **`src/ledger/`** — Blockchain abstraction (`Ledger` trait) + Stellar implementation + Soroban smart contract bindings
- **`src/store/`** — Secret persistence (`Store` trait) — IndexedDB with passkey-derived AES-GCM encryption
- **`src/db/`** — Decentralized graph database (`Db` trait) — GUN.js via JS bridge
- **`src/i18n.rs`** — `Language` enum (English, Hungarian, French, German, Spanish)
- **`assets/`** — JS bridge files, service worker, PWA manifest, icons
- **`server/`** — nginx config + log upload helper (not part of the WASM build)

### Core traits

| Trait | Module | Implementation | Purpose |
|-------|--------|----------------|---------|
| `Ledger` | `ledger/mod.rs` | `StellarLedger` | Keygen, signing, tx building, Horizon API |
| `SmartContract` | `ledger/sc/mod.rs` | `ZsozsoSc` | Soroban contract invocation |
| `Store` | `store/mod.rs` | `IndexedDbStore` | Encrypted secret persistence |
| `Db` | `db/mod.rs` | `GunDb` | GUN decentralised database |
| `Sea` | `db/sea.rs` | `GunSea` | GUN SEA crypto operations |

All trait async methods use `#[allow(async_fn_in_trait)]`. Errors are `Result<T, String>` throughout — no custom error types.

### JS bridges

WASM communicates with browser APIs through `window.__*_bridge` objects:

| Bridge | JS file | Rust module |
|--------|---------|-------------|
| `__gun_bridge` | `assets/gun_bridge.js` | `db::gundb` |
| `__sea_bridge` | `assets/sea_bridge.js` | `db::sea` |
| `__passkey_bridge` | `assets/passkey_bridge.js` | `store::passkey` |
| `__qr_scanner_bridge` | `assets/qr_scanner_bridge.js` | `ui::qr_scanner` |
| `__zsozso_log` | `assets/log_bridge.js` | `ui::tabs::log` |

### State management

`WalletState` (in `ui/state.rs`) is a struct of Dioxus `Signal<T>` fields — one signal per piece of reactive state. Initialized via `use_wallet_state()` hook.

`AppController` (in `ui/controller.rs`) bridges state and actions:
- **Sync methods** mutate signals directly (e.g. `generate_key()`, `import_key()`)
- **Async methods** use `spawn(async move { ... })` and follow the `*_action()` naming suffix (e.g. `submit_transaction_action()`, `ping_contract_action()`)

Authentication uses the `AuthState` enum: `Pending → Authenticating → Authenticated | Failed`.

### Network graph (`db/network.rs`)

The `NetworkGraph` trait abstracts the Iceberg Protocol hierarchy stored in GUN DB:
- Each node is identified by its Stellar public key
- Data model: one parent per node, N workers, and an optional nickname
- Ancestry chain traversed up to 6 levels for the UI
- `GunNetworkGraph` implements the trait using `GunDb` under `network/<pubkey>/` paths
- Write operations will use SEA authentication; `request_modify()` is a placeholder for environment-approved changes

### Service worker & deployment

`build.sh` stamps a date+commit CACHE_NAME into `assets/sw.js` and `index.html` before building. Never deploy manually — always use `./build.sh`. The SW uses `skipWaiting()` + `clients.claim()` for immediate activation; `index.html` auto-reloads on `controllerchange`.

## Key Conventions

### Component pattern

Tab components are free functions returning `Element`:
```rust
pub fn render_home_tab(i18n: &dyn UiI18n) -> Element { rsx! { ... } }
pub fn render_settings_tab(s: WalletState, ctrl: AppController, i18n: &dyn UiI18n) -> Element { rsx! { ... } }
```
Components receive `(WalletState, AppController, &dyn UiI18n)` as needed — not all three are always required.

### I18n pattern

Every user-facing string is behind an i18n trait method. Each module owns its own i18n layer (`UiI18n`, `LedgerI18n`, `StoreI18n`, `ScI18n`, `DbI18n`). Factory functions select the implementation:
```rust
pub fn ui_i18n(lang: Language) -> Box<dyn UiI18n> { ... }
```

Method naming: `btn_*()` for buttons, `lbl_*()` for labels, `fmt_*()` for format helpers returning `String`, plain names for status messages returning `&'static str`.

**Adding a new language** requires:
1. Add variant to `Language` enum in `src/i18n.rs`
2. Add implementation file in each `i18n/` directory (5 modules)
3. Register in each factory function

### Security

Secret keys are always wrapped in `Zeroizing<String>` (from the `zeroize` crate). Seed bytes are zeroed immediately after use. PRF keys for IndexedDB encryption are obtained lazily via passkey authentication.

### Styling

All styles are inline CSS via Rust format strings in `rsx!` — there are no CSS files or CSS-in-Rust frameworks.

### Web base path

The app is served under `/app/` — configured via `base_path = "app"` in `Dioxus.toml`.
