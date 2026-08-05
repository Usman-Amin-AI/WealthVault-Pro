use std::collections::HashMap;
use std::sync::Arc;

use investwise_core::brokers::service::BrokerService;
use investwise_core::settings::{SettingsRepositoryTrait, Settings, SettingsUpdate};
use investwise_core::db;
use investwise_core::errors::Error as AppError;

struct DummySettingsRepository {
    values: HashMap<(String, String), String>,
}

impl DummySettingsRepository {
    fn new() -> Self {
        Self { values: HashMap::new() }
    }

    fn with(mut self, profile_id: &str, key: &str, value: &str) -> Self {
        self.values.insert((profile_id.to_string(), key.to_string()), value.to_string());
        self
    }
}

#[async_trait::async_trait]
impl SettingsRepositoryTrait for DummySettingsRepository {
    fn get_settings(&self, _profile_id: &str) -> Result<Settings, investwise_core::errors::Error> {
        Ok(Settings::default())
    }

    async fn update_settings(&self, _new_settings: &SettingsUpdate, _profile_id: &str) -> Result<(), investwise_core::errors::Error> {
        Ok(())
    }

    fn get_setting(&self, setting_key_param: &str, profile_id: &str) -> Result<String, investwise_core::errors::Error> {
        self.values
            .get(&(profile_id.to_string(), setting_key_param.to_string()))
            .cloned()
            .ok_or_else(|| investwise_core::errors::Error::MissingConfigKey(setting_key_param.to_string()))
    }

    async fn update_setting(&self, _setting_key_param: &str, _setting_value_param: &str, _profile_id: &str) -> Result<(), investwise_core::errors::Error> {
        Ok(())
    }

    fn get_distinct_currencies_excluding_base(&self, _base_currency: &str) -> Result<Vec<String>, investwise_core::errors::Error> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn test_get_auth_url_ibkr() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_db = std::env::temp_dir().join(format!("test_db_{}.db", uuid::Uuid::new_v4()));
    let db_path = tmp_db.to_str().unwrap();
    let pool = db::create_pool(db_path)?;

    let settings = DummySettingsRepository::new()
        .with("test_profile", "IBKR_CLIENT_ID", "client123")
        .with("test_profile", "IBKR_CLIENT_SECRET", "secret-xyz");

    let svc = BrokerService::new(Arc::new(settings), pool);
    let url = svc.get_auth_url("IBKR", "test_profile").await.map_err(|e| format!("err: {}", e))?;
    assert!(url.contains("client123"), "auth url should include client id");
    Ok(())
}

#[tokio::test]
async fn test_get_auth_url_unknown_provider_errors() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_db = std::env::temp_dir().join(format!("test_db_{}.db", uuid::Uuid::new_v4()));
    let db_path = tmp_db.to_str().unwrap();
    let pool = db::create_pool(db_path)?;

    // supply keys for UNKNOWN so get_api_keys succeeds, but provider is unsupported
    let settings = DummySettingsRepository::new()
        .with("test_profile", "UNSUPPORTED_CLIENT_ID", "x")
        .with("test_profile", "UNSUPPORTED_CLIENT_SECRET", "y");

    let svc = BrokerService::new(Arc::new(settings), pool);
    let res = svc.get_auth_url("UNSUPPORTED", "test_profile").await;
    assert!(res.is_err(), "unsupported provider should return error");
    Ok(())
}
#[tokio::test]
async fn test_get_auth_url_uses_profile_scoped_settings() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_db = std::env::temp_dir().join(format!("test_db_{}.db", uuid::Uuid::new_v4()));
    let db_path = tmp_db.to_str().unwrap();
    let pool = db::create_pool(db_path)?;

    let settings = DummySettingsRepository::new()
        .with("profile_a", "IBKR_CLIENT_ID", "client-a")
        .with("profile_a", "IBKR_CLIENT_SECRET", "secret-a")
        .with("profile_b", "IBKR_CLIENT_ID", "client-b")
        .with("profile_b", "IBKR_CLIENT_SECRET", "secret-b");

    let svc = BrokerService::new(Arc::new(settings), pool);
    let url_a = svc.get_auth_url("IBKR", "profile_a").await?;
    let url_b = svc.get_auth_url("IBKR", "profile_b").await?;

    assert!(url_a.contains("client-a"));
    assert!(url_b.contains("client-b"));
    assert!(!url_a.contains("client-b"));
    Ok(())
}
