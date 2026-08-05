use std::sync::Arc;

use crate::{error::ApiResult, main_lib::AppState};
use axum::{extract::{Query, State}, routing::post, Json, Router};
use investwise_core::{
    accounts::AccountServiceTrait,
    portfolio::{
        income::IncomeSummary,
        performance::{PerformanceMetrics, SimplePerformanceMetrics},
    },
};

#[derive(serde::Deserialize, Default)]
struct ProfileQuery {
    #[serde(rename = "profile_id")]
    profile_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct AccountsSimplePerfBody {
    #[serde(rename = "accountIds")]
    account_ids: Option<Vec<String>>,
}

async fn calculate_accounts_simple_performance(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
    Json(body): Json<AccountsSimplePerfBody>,
) -> ApiResult<Json<Vec<SimplePerformanceMetrics>>> {
    let profile_id = query.profile_id.as_deref().unwrap_or("default_profile");
    let ids = if let Some(ids) = body.account_ids {
        ids
    } else {
        state
            .account_service
            .get_active_accounts(profile_id)?
            .into_iter()
            .map(|a| a.id)
            .collect()
    };
    if ids.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let metrics = state
        .performance_service
        .calculate_accounts_simple_performance(&ids, profile_id)?;
    Ok(Json(metrics))
}

#[derive(serde::Deserialize)]
struct PerfBody {
    #[serde(rename = "itemType")]
    item_type: String,
    #[serde(rename = "itemId")]
    item_id: String,
    #[serde(rename = "startDate")]
    start_date: Option<String>,
    #[serde(rename = "endDate")]
    end_date: Option<String>,
}

async fn calculate_performance_history(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
    Json(body): Json<PerfBody>,
) -> ApiResult<Json<PerformanceMetrics>> {
    let start = match &body.start_date {
        Some(s) => Some(
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid startDate: {}", e))?,
        ),
        None => None,
    };
    let end = match &body.end_date {
        Some(s) => Some(
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid endDate: {}", e))?,
        ),
        None => None,
    };
    let profile_id = query.profile_id.as_deref().unwrap_or("default_profile");
    let metrics = state
        .performance_service
        .calculate_performance_history(&body.item_type, &body.item_id, start, end, profile_id)
        .await?;
    Ok(Json(metrics))
}

async fn calculate_performance_summary(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
    Json(body): Json<PerfBody>,
) -> ApiResult<Json<PerformanceMetrics>> {
    let start = match &body.start_date {
        Some(s) => Some(
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid startDate: {}", e))?,
        ),
        None => None,
    };
    let end = match &body.end_date {
        Some(s) => Some(
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| anyhow::anyhow!("Invalid endDate: {}", e))?,
        ),
        None => None,
    };
    let profile_id = query.profile_id.as_deref().unwrap_or("default_profile");
    let metrics = state
        .performance_service
        .calculate_performance_summary(&body.item_type, &body.item_id, start, end, profile_id)
        .await?;
    Ok(Json(metrics))
}

async fn get_income_summary(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ProfileQuery>,
) -> ApiResult<Json<Vec<IncomeSummary>>> {
    let profile_id = query.profile_id.as_deref().unwrap_or("default_profile");
    let items = state.income_service.get_income_summary(profile_id)?;
    Ok(Json(items))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/performance/accounts/simple",
            post(calculate_accounts_simple_performance),
        )
        .route("/performance/history", post(calculate_performance_history))
        .route("/performance/summary", post(calculate_performance_summary))
        .route("/income/summary", axum::routing::get(get_income_summary))
}
