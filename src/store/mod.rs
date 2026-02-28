mod local_storage;
pub mod i18n;

pub use self::local_storage::LocalStorageStore;

/// Abstract interface for secure secret key storage.
/// The UI uses this trait — it doesn't know whether browser localStorage or another backend is behind it.
pub trait Store {
    /// Save a secret to the secure store.
    fn save(&self, secret: &str) -> Result<(), String>;

    /// Load a secret from the secure store.
    fn load(&self) -> Result<String, String>;
}
