use chrono::{Duration, Utc, NaiveDateTime};
use std::collections::HashMap;

use crate::db::DbPool;
use crate::errors::{AppError, ValidationError};
use crate::settings::SettingsRepositoryTrait;
use std::sync::Arc;

use super::ibkr::IbkrIntegration;
use super::robinhood::RobinhoodIntegration;
use super::schwab::SchwabIntegration;
use super::models::{BrokerConnection, BrokerConnectionDto};
use super::repository::BrokerConnectionsRepository;
use super::{BrokerIntegration, oauth2::start_local_oauth_server};
use uuid::Uuid;

pub struct BrokerService {
    settings_repository: Arc<dyn SettingsRepositoryTrait>,
    pool: Arc<DbPool>,
}

impl BrokerService {
    pub fn new(settings_repository: Arc<dyn SettingsRepositoryTrait>, pool: Arc<DbPool>) -> Self {
        Self { settings_repository, pool }
    }

    fn get_integration(provider: &str, client_id: &str, client_secret: &str) -> Result<Box<dyn BrokerIntegration>, AppError> {
        match provider.to_uppercase().as_str() {
            "IBKR" => Ok(Box::new(IbkrIntegration::new(client_id, client_secret))),
            "SCHWAB" => Ok(Box::new(SchwabIntegration::new(client_id, client_secret))),
            "ROBINHOOD" => Ok(Box::new(RobinhoodIntegration::new(client_id, client_secret))),
            _ => Err(AppError::Validation(ValidationError::InvalidInput(format!("Unsupported broker provider: {}", provider)))),
        }
    }

    fn get_api_keys(&self, provider: &str, profile_id: &str) -> Result<(String, String), AppError> {
        let key_prefix = format!("{}_", provider.to_uppercase());
        let client_id_key = format!("{}CLIENT_ID", key_prefix);
        let client_secret_key = format!("{}CLIENT_SECRET", key_prefix);

        let client_id = self.settings_repository.get_setting(&client_id_key, profile_id)
            .map_err(|_| AppError::Validation(ValidationError::InvalidInput(format!("Missing {} in settings", client_id_key))))?;

        let client_secret = self.settings_repository.get_setting(&client_secret_key, profile_id)
            .map_err(|_| AppError::Validation(ValidationError::InvalidInput(format!("Missing {} in settings", client_secret_key))))?;

        Ok((client_id, client_secret))
    }

    pub async fn get_auth_url(&self, provider: &str, profile_id: &str) -> Result<String, AppError> {
        let (client_id, client_secret) = self.get_api_keys(provider, profile_id)?;
        let integration = Self::get_integration(provider, &client_id, &client_secret)?;

        let port: u16 = 8484;
        let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

        integration.get_authorization_url(&redirect_uri)
    }

    pub async fn handle_oauth_callback(&self, account_id: &str, provider: &str, profile_id: &str) -> Result<BrokerConnectionDto, AppError> {
        let (client_id, client_secret) = self.get_api_keys(provider, profile_id)?;
        let integration = Self::get_integration(provider, &client_id, &client_secret)?;

        let port: u16 = 8484;
        let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

        // Start the temporary server and wait for the callback
        let code = start_local_oauth_server(port).await?;

        // Exchange code for token
        let token_resp = integration.exchange_code_for_token(&code, &redirect_uri).await?;

        let now = Utc::now().naive_utc();
        let expires_at = token_resp.expires_in_seconds.map(|secs| now + Duration::seconds(secs));

        let conn = BrokerConnection {
            id: Uuid::new_v4().to_string(),
            account_id: account_id.to_string(),
            provider: provider.to_string(),
            access_token: token_resp.access_token,
            refresh_token: token_resp.refresh_token,
            expires_at,
            created_at: now,
            updated_at: now,
        };

        let saved = BrokerConnectionsRepository::insert_or_update(&self.pool, conn, profile_id)?;
        Ok(BrokerConnectionDto::from(saved))
    }

    pub fn get_connections_by_account(&self, account_id: &str, profile_id: &str) -> Result<Option<BrokerConnectionDto>, AppError> {
        let conn = BrokerConnectionsRepository::get_by_account_id(&self.pool, account_id, profile_id)?;
        Ok(conn.map(BrokerConnectionDto::from))
    }

    pub async fn fetch_broker_activities(&self, account_id: &str, profile_id: &str) -> Result<Vec<crate::activities::activities_model::ActivityImport>, AppError> {
        let conn = BrokerConnectionsRepository::get_by_account_id(&self.pool, account_id, profile_id)?
            .ok_or_else(|| AppError::Validation(ValidationError::InvalidInput(
                "Broker connection not found for account".into(),
            )))?;

        let (client_id, client_secret) = self.get_api_keys(&conn.provider, profile_id)?;
        let integration = Self::get_integration(&conn.provider, &client_id, &client_secret)?;

        let activities = integration.fetch_activities(
            &conn.access_token,
            &conn.account_id,
            None,
            None
        ).await?;

        Ok(activities)
    }
}
