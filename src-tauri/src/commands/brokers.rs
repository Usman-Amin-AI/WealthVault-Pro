use investwise_core::activities::ActivityImport;
use investwise_core::brokers::models::BrokerConnectionDto;
use investwise_core::brokers::service::BrokerService;
use std::sync::Arc;

#[tauri::command]
pub async fn get_auth_url(
    provider: String,
    profile_id: Option<String>,
    state: tauri::State<'_, Arc<crate::context::ServiceContext>>,
) -> Result<String, String> {
    let pid = profile_id.as_deref().unwrap_or_default();
    state.broker_service().get_auth_url(&provider, pid).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn handle_oauth_callback(
    account_id: String,
    provider: String,
    profile_id: Option<String>,
    state: tauri::State<'_, Arc<crate::context::ServiceContext>>,
) -> Result<BrokerConnectionDto, String> {
    let pid = profile_id.as_deref().unwrap_or_default();
    state.broker_service().handle_oauth_callback(&account_id, &provider, pid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_broker_connection(
    account_id: String,
    profile_id: Option<String>,
    state: tauri::State<'_, Arc<crate::context::ServiceContext>>,
) -> Result<Option<BrokerConnectionDto>, String> {
    let pid = profile_id.as_deref().unwrap_or_default();
    state.broker_service().get_connections_by_account(&account_id, pid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn fetch_broker_activities(
    account_id: String,
    profile_id: Option<String>,
    state: tauri::State<'_, Arc<crate::context::ServiceContext>>,
) -> Result<Vec<ActivityImport>, String> {
    let pid = profile_id.as_deref().unwrap_or_default();
    state.broker_service().fetch_broker_activities(&account_id, pid).await.map_err(|e| e.to_string())
}
