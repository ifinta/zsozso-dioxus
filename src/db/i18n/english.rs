use super::DbI18n;

pub struct EnglishDb;

impl DbI18n for EnglishDb {
    fn read_error(&self, error: &str) -> String { format!("DB read error: {:?}", error) }
    fn write_error(&self, error: &str) -> String { format!("DB write error: {:?}", error) }
    fn subscribe_error(&self, error: &str) -> String { format!("DB subscription error: {:?}", error) }
    fn connection_error(&self, error: &str) -> String { format!("DB connection error: {:?}", error) }
    fn sea_error(&self, error: &str) -> String { format!("SEA crypto error: {:?}", error) }
}
