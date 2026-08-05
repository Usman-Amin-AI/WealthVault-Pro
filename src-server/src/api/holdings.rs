use std::sync::Arc;

use crate::{error::ApiResult, main_lib::AppState};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use investwise_core::portfolio::{
    holdings::holdings_model::Holding, valuation::valuation_model::DailyAccountValuation,
};

#[derive(serde::Deserialize)]
struct HoldingsQuery {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "profile_id")]
    profile_id: Option<String>,
}

async fn get_holdings(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HoldingsQuery>,
) -> ApiResult<Json<Vec<Holding>>> {
    let base = state.base_currency.read().unwrap().clone();
    let profile_id = q.profile_id.as_deref().unwrap_or("default_profile");
    let holdings = state
        .holdings_service
        .get_holdings(&q.account_id, &base, profile_id)
        .await?;
    Ok(Json(holdings))
}

#[derive(serde::Deserialize)]
struct HoldingItemQuery {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "assetId")]
    asset_id: String,
    #[serde(rename = "profile_id")]
    profile_id: Option<String>,
}

async fn get_holding(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HoldingItemQuery>,
) -> ApiResult<Json<Option<Holding>>> {
    let base = state.base_currency.read().unwrap().clone();
    let profile_id = q.profile_id.as_deref().unwrap_or("default_profile");
    let holding = state
        .holdings_service
        .get_holding(&q.account_id, &q.asset_id, &base, profile_id)
        .await?;
    Ok(Json(holding))
}

#[derive(serde::Deserialize)]
struct HistoryQuery {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "startDate")]
    start_date: Option<String>,
    #[serde(rename = "endDate")]
    end_date: Option<String>,
    #[serde(rename = "profile_id")]
    profile_id: Option<String>,
}

async fn get_historical_valuations(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> ApiResult<Json<Vec<DailyAccountValuation>>> {
    let profile_id = q.profile_id.as_deref().unwrap_or("default_profile");
    let start = match q.start_date {
        Some(s) => Some(
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid startDate: {}", e))?,
        ),
        None => None,
    };
    let end = match q.end_date {
        Some(s) => Some(
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid endDate: {}", e))?,
        ),
        None => None,
    };
    let vals = state
        .valuation_service
        .get_historical_valuations(&q.account_id, start, end, profile_id)?;
    Ok(Json(vals))
}

async fn get_latest_valuations(
    State(state): State<Arc<AppState>>,
    raw: axum::extract::RawQuery,
) -> ApiResult<Json<Vec<DailyAccountValuation>>> {
    use investwise_core::accounts::AccountServiceTrait;

    let pairs = raw
        .0
        .as_ref()
        .and_then(|qs| serde_urlencoded::from_str::<Vec<(String, String)>>(qs).ok())
        .unwrap_or_default();

    let mut ids: Vec<String> = Vec::new();
    for (k, v) in &pairs {
        if k == "accountIds" || k == "accountIds[]" {
            ids.push(v.clone());
        }
    }

    let profile_id = pairs
        .iter()
        .find(|(k, _)| k == "profile_id")
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "default_profile".to_string());

    if ids.is_empty() {
        ids = state
            .account_service
            .get_active_accounts(&profile_id)?
            .into_iter()
            .map(|a| a.id)
            .collect();
    }
    if ids.is_empty() {
        return Ok(Json(vec![]));
    }
    let vals = state.valuation_service.get_latest_valuations(&ids, &profile_id)?;
    Ok(Json(vals))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/holdings", get(get_holdings))
        .route("/holdings/item", get(get_holding))
        .route("/valuations/history", get(get_historical_valuations))
        .route("/valuations/latest", get(get_latest_valuations))
}
