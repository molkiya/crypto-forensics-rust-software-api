use std::env;

/// Конфигурация приложения
pub struct Config {
    /// ID тестовой транзакции для демонстрации
    pub test_tx_id: String,
    /// Путь к директории с данными
    pub data_dir: String,
    /// Имя папки по умолчанию для данных
    pub default_data_folder: String,
}

impl Config {
    /// Создает конфигурацию из переменных окружения или использует значения по умолчанию
    pub fn from_env() -> Self {
        Self {
            test_tx_id: env::var("TEST_TX_ID")
                .unwrap_or_else(|_| "d6176384de4c0b98702eccb97f3ad6670bc8410d9da715fe5b49462d3e603993".to_string()),
            data_dir: env::var("DATA_DIR")
                .unwrap_or_else(|_| "./src/data".to_string()),
            default_data_folder: env::var("DEFAULT_DATA_FOLDER")
                .unwrap_or_else(|_| "1111DAYXhoxZx2tsRnzimfozo783x1yC2".to_string()),
        }
    }
}

