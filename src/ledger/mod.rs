mod stellar;

pub use stellar::StellarLedger;

/// Kulcspár: publikus cím (G...) és titkos kulcs (S...)
pub struct KeyPair {
    pub public_key: String,
    pub secret_key: String,
}

/// Hálózat választó — a Ledger implementáció dönti el, mit jelent
#[derive(Clone, Copy, PartialEq)]
pub enum NetworkEnvironment {
    Production,
    Test,
}

/// Hálózat leírása az UI számára
pub struct NetworkInfo {
    pub name: &'static str,
    pub has_faucet: bool,
}

/// A főkönyvi rendszer absztrakt interfésze.
/// A main.rs ezt a trait-et használja — nem tud róla, hogy Stellar van mögötte.
#[allow(async_fn_in_trait)]
pub trait Ledger {
    /// Hálózat információ az UI-nak
    fn network_info(&self) -> NetworkInfo;

    /// Új kulcspár generálása
    fn generate_keypair(&self) -> KeyPair;

    /// Titkos kulcsból publikus cím kiszámítása
    /// None-t ad vissza, ha a kulcs formátuma hibás
    fn public_key_from_secret(&self, secret: &str) -> Option<String>;

    /// Teszt fiók aktiválása (faucet) — csak Test hálózaton
    async fn activate_test_account(&self, public_key: &str) -> Result<String, String>;

    /// Önmagának küldő fizetési tranzakció generálása és aláírása
    /// Visszaadja a kész, beküldésre kész XDR-t (base64) és a szekvenciaszámot
    async fn build_self_payment(
        &self,
        secret_key: &str,
        amount: i64,
    ) -> Result<(String, i64), String>;

    /// Aláírt tranzakció beküldése a hálózatra
    async fn submit_transaction(&self, xdr: &str) -> Result<String, String>;
}