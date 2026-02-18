mod keyring;
pub mod i18n;

pub use self::keyring::KeyringStore;

/// Abstract interface for secure secret key storage.
/// The UI uses this trait — it doesn't know whether keyring, file, or browser localStorage is behind it.
pub trait Store {
    /// Save a secret to the secure store.
    fn save(&self, secret: &str) -> Result<(), String>;

    /// Load a secret from the secure store.
    fn load(&self) -> Result<String, String>;
}
