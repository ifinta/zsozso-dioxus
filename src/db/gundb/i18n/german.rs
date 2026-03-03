use super::DbI18n;

pub struct GermanDb;

impl DbI18n for GermanDb {
    fn read_error(&self, error: &str) -> String { format!("DB-Lesefehler: {:?}", error) }
    fn write_error(&self, error: &str) -> String { format!("DB-Schreibfehler: {:?}", error) }
    fn subscribe_error(&self, error: &str) -> String { format!("DB-Abonnementfehler: {:?}", error) }
    fn connection_error(&self, error: &str) -> String { format!("DB-Verbindungsfehler: {:?}", error) }
    fn sea_error(&self, error: &str) -> String { format!("SEA-Kryptographiefehler: {:?}", error) }
}
