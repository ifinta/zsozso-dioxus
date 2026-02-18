mod keyring;

pub use self::keyring::KeyringStore;

/// Titkos kulcs biztonságos tárolásának absztrakt interfésze.
/// A UI ezt a trait-et használja — nem tud róla, hogy keyring, fájl, vagy böngésző localStorage van mögötte.
pub trait Store {
    /// Titok mentése a biztonságos tárolóba.
    fn save(&self, secret: &str) -> Result<(), String>;

    /// Titok betöltése a biztonságos tárolóból.
    fn load(&self) -> Result<String, String>;
}
