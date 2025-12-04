use tokio::sync::OnceCell;
use reqwest::Client;

use crate::infrastructure::constants::BITCOIN_EXPLORER_URL;
use super::errors::explorer_errors::ExplorerError;

/// Клиент для работы с Bitcoin Explorer API
pub struct ExplorerClient {
    http: Client,
    base_url: String,
}

impl ExplorerClient {
    /// Создает новый клиент для работы с Bitcoin Explorer API
    ///
    /// # Arguments
    /// * `base_url` - Базовый URL API explorer'а
    ///
    /// # Returns
    /// `Result<Self, ExplorerError>` - клиент или ошибка создания
    pub fn new(base_url: impl Into<String>) -> Result<Self, ExplorerError> {
        let http = Client::builder()
            .pool_max_idle_per_host(8)
            .build()
            .map_err(|e| ExplorerError::ClientBuildError(e.to_string()))?;
        
        Ok(ExplorerClient {
            http,
            base_url: base_url.into(),
        })
    }
    
    /// Получает информацию о транзакции по её ID
    ///
    /// # Arguments
    /// * `txid` - ID транзакции в формате hex string
    ///
    /// # Returns
    /// `Result<serde_json::Value, ExplorerError>` - JSON данные транзакции или ошибка
    pub async fn get_transaction(&self, txid: &str) -> Result<serde_json::Value, ExplorerError> {
        let url = format!("{}/tx/{}", self.base_url, txid);
        let resp = self
            .http
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(resp)
    }
}

pub static BITCOIN_EXPLORER_CLIENT: OnceCell<ExplorerClient> = OnceCell::const_new();

/// Получает или инициализирует глобальный клиент Bitcoin Explorer
///
/// # Returns
/// `Result<&'static ExplorerClient, ExplorerError>` - ссылка на клиент или ошибка инициализации
pub async fn get_or_init_client() -> Result<&'static ExplorerClient, ExplorerError> {
    BITCOIN_EXPLORER_CLIENT
        .get_or_try_init(|| async {
            ExplorerClient::new(&BITCOIN_EXPLORER_URL.to_string())
        })
        .await
}