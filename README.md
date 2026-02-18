# Zsozso

The main desktop application for the [Iceberg Protocol](https://ifinta.github.io/zsozso-webpage/) ecosystem, built with Rust and [Dioxus](https://dioxuslabs.com/).

ZSOZSO is the utility token fueling the Iceberg Protocol — a decentralized hierarchical MLM infrastructure and message-bus architecture on the [Stellar](https://stellar.org/) blockchain. This app provides a native desktop wallet for key management, transaction signing, and network interaction.

## Features

- **Key Management** — Generate new Stellar keypairs or import existing secret keys
- **Secure Storage** — Secret keys are stored in the OS-level credential manager (GNOME Keyring, macOS Keychain, Windows Credential Manager)
- **Network Switching** — Toggle between Stellar Mainnet and Testnet with a single click
- **Language Switching** — Toggle between English and Hungarian UI at runtime
- **Transaction Signing** — Build and sign XDR transaction envelopes locally
- **Transaction Submission** — Submit signed transactions directly to Stellar Horizon
- **Testnet Faucet** — Activate test accounts via Friendbot (Testnet only)
- **Clipboard Security** — Secret keys are auto-cleared from clipboard after 30 seconds

## Architecture

The application is designed with trait-based abstraction layers so that no module depends on blockchain or platform specifics directly:

```
src/
├── main.rs                  # Application entry point (launches Dioxus desktop)
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
│   ├── keyring.rs           # OS keyring implementation (GNOME/macOS/Windows)
│   └── i18n/
│       ├── mod.rs           # StoreI18n trait — storage error messages
│       ├── english.rs       # English implementation
│       └── hungarian.rs     # Hungarian implementation
└── ui/
    ├── mod.rs               # Dioxus UI components and application logic
    ├── clipboard.rs         # Clipboard copy with timed auto-clear
    └── i18n/
        ├── mod.rs           # UiI18n trait — all UI-facing strings
        ├── english.rs       # English implementation
        └── hungarian.rs     # Hungarian implementation
```

### Core Traits

| Trait | Purpose | Current Implementation |
|-------|---------|----------------------|
| `Ledger` | Blockchain operations (keygen, signing, submitting) | `StellarLedger` (Stellar Horizon + XDR) |
| `Store` | Secure secret persistence | `KeyringStore` (OS credential manager) |

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

| Platform | Status |
|----------|--------|
| Linux (Debian/Ubuntu) | ✅ Supported (current) |
| macOS | 🔜 Planned |
| Windows | 🔜 Planned |
| Web (WASM) | 🔜 Planned |
| iOS | 🔜 Planned |
| Android | 🔜 Planned |

## Prerequisites

### Linux (Debian/Ubuntu)

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

## Build & Run

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

## Related Repositories

- [zsozso-webpage](https://github.com/ifinta/zsozso-webpage) — Project website & whitepaper
- [ZSOZSO on Stellar Expert](https://stellar.expert/explorer/public/asset/ZSOZSO-GDZKLEYJ54QUIEYE4DUUOCIJDUS7R5MDW5MCAB3XTUGPJ3C7SSZJRQUC) — ZSOZSO asset on mainnet

## License

See repository for license details.