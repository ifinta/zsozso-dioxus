use super::DbI18n;

pub struct FrenchDb;

impl DbI18n for FrenchDb {
    fn read_error(&self, error: &str) -> String { format!("Erreur de lecture DB : {:?}", error) }
    fn write_error(&self, error: &str) -> String { format!("Erreur d'écriture DB : {:?}", error) }
    fn subscribe_error(&self, error: &str) -> String { format!("Erreur d'abonnement DB : {:?}", error) }
    fn connection_error(&self, error: &str) -> String { format!("Erreur de connexion DB : {:?}", error) }
    fn sea_error(&self, error: &str) -> String { format!("Erreur cryptographique SEA : {:?}", error) }
}
