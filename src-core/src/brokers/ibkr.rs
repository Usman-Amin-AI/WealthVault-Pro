use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::str::FromStr;

use crate::errors::AppError;
use crate::activities::activities_model::ActivityImport;
use crate::assets::assets_model::Asset;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use super::{BrokerIntegration, BrokerTokenResponse};

pub struct IbkrIntegration {
    client_id: String,
    client_secret: String,
    http_client: Client,
}

impl IbkrIntegration {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            http_client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct IbkrTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

#[derive(Deserialize)]
struct IbkrActivityRecord {
    activity_id: Option<String>,
    activity_type: Option<String>,
    activity_date: Option<String>,
    symbol: Option<String>,
    symbol_description: Option<String>,
    quantity: Option<String>,
    price: Option<String>,
    fee: Option<String>,
    amount: Option<String>,
    currency: Option<String>,
    account_name: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
struct IbkrActivitiesResponse {
    activities: Vec<IbkrActivityRecord>,
}

fn parse_decimal(value: Option<&str>) -> Decimal {
    if let Some(s) = value {
        if let Ok(d) = Decimal::from_str(s) {
            return d;
        }
        if let Ok(f) = s.parse::<f64>() {
            if let Some(d) = Decimal::from_f64(f) {
                return d;
            }
        }
    }
    Decimal::ZERO
}

fn parse_date(value: Option<&str>) -> String {
    value
        .and_then(|date_str| DateTime::parse_from_rfc3339(date_str).ok()
            .map(|dt| dt.with_timezone(&Utc).format("%Y-%m-%d").to_string()))
        .unwrap_or_else(|| value.unwrap_or_default().to_string())
}

fn map_activity_type(raw: Option<String>) -> String {
    match raw.as_deref().unwrap_or("").to_uppercase().as_str() {
        "BUY" => "BUY".to_string(),
        "SELL" => "SELL".to_string(),
        "DIVIDEND" => "DIVIDEND".to_string(),
        "INTEREST" => "INTEREST".to_string(),
        "DEPOSIT" => "DEPOSIT".to_string(),
        "WITHDRAWAL" => "WITHDRAWAL".to_string(),
        "TRANSFER_IN" | "TRANSFERIN" => "TRANSFER_IN".to_string(),
        "TRANSFER_OUT" | "TRANSFEROUT" => "TRANSFER_OUT".to_string(),
        "FEE" => "FEE".to_string(),
        "TAX" => "TAX".to_string(),
        "SPLIT" => "SPLIT".to_string(),
        other => other.to_string(),
    }
}

fn build_activity_import(record: IbkrActivityRecord, account_id: &str) -> ActivityImport {
    ActivityImport {
        id: record.activity_id,
        date: parse_date(record.activity_date.as_deref()),
        symbol: record.symbol.unwrap_or_else(|| "$CASH-USD".to_string()),
        activity_type: map_activity_type(record.activity_type),
        quantity: parse_decimal(record.quantity.as_deref()),
        unit_price: parse_decimal(record.price.as_deref()),
        currency: record.currency.unwrap_or_else(|| "USD".to_string()),
        fee: parse_decimal(record.fee.as_deref()),
        amount: record.amount.and_then(|value| Decimal::from_str(&value).ok()),
        comment: record.description,
        account_id: Some(account_id.to_string()),
        account_name: record.account_name,
        symbol_name: record.symbol_description,
        errors: None,
        is_draft: false,
        is_valid: true,
        line_number: None,
    }
}

#[async_trait]
impl BrokerIntegration for IbkrIntegration {
    fn get_authorization_url(&self, redirect_uri: &str) -> Result<String, AppError> {
        let auth_url = format!(
            "https://api.ibkr.com/v1/api/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope=ACCOUNT",
            self.client_id, redirect_uri
        );
        Ok(auth_url)
    }

    async fn exchange_code_for_token(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<BrokerTokenResponse, AppError> {
        let token_endpoint = "https://api.ibkr.com/v1/api/oauth2/token";
        let response = self
            .http_client
            .post(token_endpoint)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("client_id", &self.client_id),
                ("client_secret", &self.client_secret),
            ])
            .send()
            .await
            .map_err(|e| AppError::Unexpected(e.to_string()))?;

        let token_response = response
            .json::<IbkrTokenResponse>()
            .await
            .map_err(|e| AppError::Unexpected(e.to_string()))?;

        Ok(BrokerTokenResponse {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_in_seconds: token_response.expires_in,
        })
    }

    async fn fetch_activities(
        &self,
        access_token: &str,
        account_id: &str,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>,
    ) -> Result<Vec<ActivityImport>, AppError> {
        let mut request = self
            .http_client
            .get(format!(
                "https://api.ibkr.com/v1/api/accounts/{}/activities",
                account_id
            ))
            .bearer_auth(access_token);

        if let Some(start) = start_date {
            request = request.query(&[("startDate", &start.format("%Y-%m-%d").to_string())]);
        }
        if let Some(end) = end_date {
            request = request.query(&[("endDate", &end.format("%Y-%m-%d").to_string())]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Unexpected(e.to_string()))?;

        let activities = response
            .json::<IbkrActivitiesResponse>()
            .await
            .map_err(|e| AppError::Unexpected(e.to_string()))?
            .activities;

        Ok(activities
            .into_iter()
            .map(|record| build_activity_import(record, account_id))
            .collect())
    }

    async fn fetch_assets(
        &self,
        access_token: &str,
    ) -> Result<Vec<Asset>, AppError> {
        let response = self
            .http_client
            .get("https://api.ibkr.com/v1/api/accounts/assets")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AppError::Unexpected(e.to_string()))?;

        let raw_assets = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AppError::Unexpected(e.to_string()))?;

        let assets = raw_assets
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|asset| {
                Some(Asset {
                    id: asset.get("symbol")?.as_str()?.to_string(),
                    isin: None,
                    name: asset
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    asset_type: asset
                        .get("assetType")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    symbol: asset.get("symbol")?.as_str()?.to_string(),
                    symbol_mapping: None,
                    asset_class: None,
                    asset_sub_class: None,
                    notes: None,
                    countries: None,
                    categories: None,
                    classes: None,
                    attributes: None,
                    created_at: chrono::Utc::now().naive_utc(),
                    updated_at: chrono::Utc::now().naive_utc(),
                    currency: asset
                        .get("currency")
                        .and_then(|v| v.as_str())
                        .unwrap_or("USD")
                        .to_string(),
                    data_source: "IBKR".to_string(),
                    sectors: None,
                    url: None,
                })
            })
            .collect();

        Ok(assets)
    }
}
