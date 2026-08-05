use std::sync::Arc;

use crate::context::ServiceContext;
use log::debug;
use tauri::State;
use investwise_core::profiles::profiles_model::Profile;

#[tauri::command]
pub async fn get_profiles(state: State<'_, Arc<ServiceContext>>) -> Result<Vec<Profile>, String> {
    debug!("Fetching profiles...");
    state
        .profile_service()
        .list_profiles()
        .map_err(|e| format!("Failed to load profiles: {}", e))
}

#[tauri::command]
pub async fn get_profile(
    profile_id: String,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Profile, String> {
    debug!("Fetching profile {}...", profile_id);
    state
        .profile_service()
        .get_profile(&profile_id)
        .map_err(|e| format!("Failed to load profile: {}", e))
}

#[tauri::command]
pub async fn create_profile(
    name: String,
    password: Option<String>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Profile, String> {
    debug!("Creating new profile {}...", name);
    state
        .profile_service()
        .create_profile(
            &name,
            password.as_deref(),
            None,
            false,
            None,
        )
        .await
        .map_err(|e| format!("Failed to create profile: {}", e))
}
