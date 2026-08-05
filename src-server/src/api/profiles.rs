use std::sync::Arc;

use crate::{error::{ApiError, ApiResult}, main_lib::AppState};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use investwise_core::{
    errors::{AppError, ValidationError},
    profiles::profiles_model::{ProfileShare, ProfileSharingRule},
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRequest {
    pub name: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub sharing_rule: Option<String>,
    #[serde(default)]
    pub family_mode_enabled: bool,
    #[serde(default)]
    pub encryption_key: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileShareRequest {
    pub owner_profile_id: String,
    pub shared_profile_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub permissions: String,
}

async fn list_profiles(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<investwise_core::profiles::profiles_model::Profile>>> {
    let profiles = state.profile_service.list_profiles()?;
    Ok(Json(profiles))
}

async fn create_profile(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProfileRequest>,
) -> ApiResult<Json<investwise_core::profiles::profiles_model::Profile>> {
    if payload.name.trim().is_empty() {
        return Err(ApiError::Core(AppError::Validation(ValidationError::InvalidInput(
            "Profile name cannot be empty".to_string(),
        ))));
    }

    let result = state
        .profile_service
        .create_profile(
            &payload.name,
            payload.password.as_deref(),
            payload.sharing_rule.as_deref(),
            payload.family_mode_enabled,
            payload.encryption_key.as_deref(),
        )
        .await?;

    info!("Created profile {} with family_mode_enabled={}", result.id, result.family_mode_enabled);
    Ok(Json(result))
}

async fn get_profile(
    Path(profile_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<investwise_core::profiles::profiles_model::Profile>> {
    let profile = state.profile_service.get_profile(&profile_id)?;
    Ok(Json(profile))
}

async fn update_profile_sharing(
    Path(profile_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProfileRequest>,
) -> ApiResult<Json<investwise_core::profiles::profiles_model::Profile>> {
    let normalized_rule = ProfileSharingRule::from_str(
        payload.sharing_rule.as_deref().unwrap_or("private"),
    )
    .as_str()
    .to_string();
    let updated = state
        .profile_service
        .update_profile_sharing(
            &profile_id,
            Some(&normalized_rule),
            payload.family_mode_enabled,
            payload.encryption_key.as_deref(),
        )
        .await?;

    info!("Updated profile sharing for {} to {}", profile_id, normalized_rule);
    Ok(Json(updated))
}

async fn create_share(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProfileShareRequest>,
) -> ApiResult<Json<ProfileShare>> {
    let share = state
        .profile_service
        .create_share(
            &payload.owner_profile_id,
            &payload.shared_profile_id,
            &payload.resource_type,
            &payload.resource_id,
            &payload.permissions,
        )
        .await?;

    info!("Created profile share {} for {} -> {}", share.id, payload.owner_profile_id, payload.shared_profile_id);
    Ok(Json(share))
}

async fn list_shares(
    Path(profile_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<ProfileShare>>> {
    let shares = state.profile_service.list_shares_for_profile(&profile_id)?;
    Ok(Json(shares))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/profiles", get(list_profiles).post(create_profile))
        .route("/profiles/{profile_id}", get(get_profile).put(update_profile_sharing))
        .route("/profiles/{profile_id}/shares", get(list_shares).post(create_share))
}
