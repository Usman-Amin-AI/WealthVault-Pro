use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;

use crate::errors::AppError;
use crate::activities::activities_model::ActivityImport;
use crate::assets::assets_model::Asset;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

use super::{BrokerIntegration, BrokerTokenResponse};

pub struct RobinhoodIntegration {
    client_id: String,
    client_secret: String,
    http_client: Client,
}

impl RobinhoodIntegration {
    pub fn new(client_id: &str, client_secret: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            http_client: Client::new(),
        }
    }
}

#[derive(Deserialize)]
struct RobinhoodTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
}

fn parse_string(value: Option<&Value>) -> Option<String> {
    value.and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

fn parse_decimal(value: Option<&Value>) -> Decimal {
    value
        .and_then(|v| match v {
            Value::String(s) => Decimal::from_str(s).ok().or_else(|| {
                s.parse::<f64>()
                    .ok()
                    .and_then(Decimal::from_f64)
            }),
            Value::Number(n) => n.as_f64().and_then(Decimal::from_f64),
            _ => None,
        })
        .unwrap_or_else(|| Decimal::ZERO)
}

fn parse_date(value: Option<&Value>) -> String {
    value
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc).format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| value.and_then(|v| v.as_str()).unwrap_or_default().to_string())
}

fn map_activity_type(raw: Option<&Value>) -> String {
    let text = raw
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_uppercase();

    match text.as_str() {
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

fn build_activity_import(record: &Value, account_id: &str) -> ActivityImport {
    let symbol = parse_string(record.get("symbol")).unwrap_or_else(|| "$CASH-USD".to_string());
    let symbol_name = parse_string(record.get("symbol_name")).or_else(|| parse_string(record.get("description")));
    let account_name = parse_string(record.get("account_name")).or_else(|| parse_string(record.get("account")));

    ActivityImport {
        id: parse_string(record.get("id")).or_else(|| parse_string(record.get("activity_id"))),
        date: parse_date(record.get("date")),
        symbol,
        activity_type: map_activity_type(record.get("type")),
        quantity: parse_decimal(record.get("quantity")),
        unit_price: parse_decimal(record.get("price")).max(parse_decimal(record.get("unit_price"))),
        currency: parse_string(record.get("currency")).unwrap_or_else(|| "USD".to_string()),
        fee: parse_decimal(record.get("fees")).max(parse_decimal(record.get("fee"))),
        amount: parse_string(record.get("amount")).and_then(|value| Decimal::from_str(&value).ok()),
        comment: parse_string(record.get("description")),
        account_id: Some(account_id.to_string()),
        account_name,
        symbol_name,
        errors: None,
        is_draft: false,
        is_valid: true,
        line_number: None,
    }
}

#[async_trait]
impl BrokerIntegration for RobinhoodIntegration {
    fn get_authorization_url(&self, redirect_uri: &str) -> Result<String, AppError> {
        let auth_url = format!(
            "https://api.robinhood.com/oauth2/authorize?response_type=code&client_id={}&redirect_uri={}&scope=read",
            self.client_id,
            redirect_uri
        );
        Ok(auth_url)
    }

    async fn exchange_code_for_token(
        &self,
        code: &str,
        redirect_uri: &str,
    ) -> Result<BrokerTokenResponse, AppError> {
        let token_endpoint = "https://api.robinhood.com/oauth2/token/";
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
            .json::<RobinhoodTokenResponse>()
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
                "https://api.robinhood.com/accounts/{}/activities/",
                account_id
            ))
            .bearer_auth(access_token);

        if let Some(start) = start_date {
            request = request.query(&[("begin_date", &start.format("%Y-%m-%d").to_string())]);
        }
        if let Some(end) = end_date {
            request = request.query(&[("end_date", &end.format("%Y-%m-%d").to_string())]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| AppError::Unexpected(e.to_string()))?;

        let payload = response
            .json::<Value>()
            .await
            .map_err(|e| AppError::Unexpected(e.to_string()))?;

        let results = payload
            .get("results")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_else(Vec::new);

        Ok(results
            .iter()
            .map(|record| build_activity_import(record, account_id))
            .collect())
    }

    async fn fetch_assets(&self, _access_token: &str) -> Result<Vec<Asset>, AppError> {
        Ok(vec![])
    }
}
