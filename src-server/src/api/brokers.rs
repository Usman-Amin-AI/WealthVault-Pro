use axum::{
    extract::{Path, State, Query},
    routing::{get, post},
    Json, Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use investwise_core::brokers::models::BrokerConnectionDto;
use investwise_core::activities::ActivityImport;
use crate::main_lib::AppState;

pub fn brokers_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth-url/{provider}", get(get_auth_url))
        .route("/callback/{account_id}/{provider}", post(handle_oauth_callback))
        .route("/connections/{account_id}", get(get_broker_connection))
        .route("/broker-activities/{account_id}", post(fetch_broker_activities))
}

async fn get_auth_url(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<String>, String> {
    let profile_id = query.get("profile_id").map(|s| s.as_str()).unwrap_or_default();
    state.broker_service.get_auth_url(&provider, profile_id).await.map(Json).map_err(|e| e.to_string())
}

async fn handle_oauth_callback(
    State(state): State<Arc<AppState>>,
    Path((account_id, provider)): Path<(String, String)>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<BrokerConnectionDto>, String> {
    let profile_id = query.get("profile_id").map(|s| s.as_str()).unwrap_or_default();
    state.broker_service.handle_oauth_callback(&account_id, &provider, profile_id)
        .await
        .map(Json)
        .map_err(|e| e.to_string())
}

async fn get_broker_connection(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Option<BrokerConnectionDto>>, String> {
    let profile_id = query.get("profile_id").map(|s| s.as_str()).unwrap_or_default();
    state.broker_service.get_connections_by_account(&account_id, profile_id)
        .map(Json)
        .map_err(|e| e.to_string())
}

async fn fetch_broker_activities(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Vec<ActivityImport>>, String> {
    let profile_id = query.get("profile_id").map(|s| s.as_str()).unwrap_or_default();
    state.broker_service.fetch_broker_activities(&account_id, profile_id)
        .await
        .map(Json)
        .map_err(|e| e.to_string())
}
