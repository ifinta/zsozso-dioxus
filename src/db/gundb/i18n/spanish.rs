use super::DbI18n;

pub struct SpanishDb;

impl DbI18n for SpanishDb {
    fn read_error(&self, error: &str) -> String { format!("Error de lectura de BD: {:?}", error) }
    fn write_error(&self, error: &str) -> String { format!("Error de escritura de BD: {:?}", error) }
    fn subscribe_error(&self, error: &str) -> String { format!("Error de suscripción de BD: {:?}", error) }
    fn connection_error(&self, error: &str) -> String { format!("Error de conexión de BD: {:?}", error) }
    fn sea_error(&self, error: &str) -> String { format!("Error criptográfico SEA: {:?}", error) }
}
