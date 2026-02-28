use super::DbI18n;

pub struct HungarianDb;

impl DbI18n for HungarianDb {
    fn read_error(&self, error: &str) -> String { format!("DB olvasási hiba: {:?}", error) }
    fn write_error(&self, error: &str) -> String { format!("DB írási hiba: {:?}", error) }
    fn subscribe_error(&self, error: &str) -> String { format!("DB feliratkozási hiba: {:?}", error) }
    fn connection_error(&self, error: &str) -> String { format!("DB kapcsolódási hiba: {:?}", error) }
}
