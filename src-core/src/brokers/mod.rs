pub mod ibkr;
pub mod models;
pub mod oauth2;
pub mod repository;
pub mod robinhood;
pub mod schwab;
pub mod service;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use crate::errors::AppError;
use crate::activities::activities_model::ActivityImport;
use crate::assets::assets_model::Asset;

#[async_trait]
pub trait BrokerIntegration: Send + Sync {
    /// Provide the OAuth URL that the user needs to visit.
    fn get_authorization_url(&self, redirect_uri: &str) -> Result<String, AppError>;

    /// Exchange the OAuth code for an access token.
    async fn exchange_code_for_token(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<BrokerTokenResponse, AppError>;

    /// Fetch activities from the broker for a specific account.
    async fn fetch_activities(
        &self,
        access_token: &str,
        account_id: &str,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>,
    ) -> Result<Vec<ActivityImport>, AppError>;

    /// Extract assets that might not exist in the database yet.
    async fn fetch_assets(
        &self,
        access_token: &str,
    ) -> Result<Vec<Asset>, AppError>;
}

pub struct BrokerTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in_seconds: Option<i64>,
}
