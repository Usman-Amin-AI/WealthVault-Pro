use super::profiles_model::{NewProfile, Profile, ProfileShare, NewProfileShare, ProfileSharingRule};
use super::profiles_repository::ProfileRepository;
use crate::errors::{AppError, Result};
use chrono::Utc;
use log::{info, warn};
use uuid::Uuid;

pub struct ProfileService {
    repository: ProfileRepository,
}

impl ProfileService {
    pub fn new(repository: ProfileRepository) -> Self {
        Self { repository }
    }

    pub async fn create_profile(
        &self,
        name: &str,
        password: Option<&str>,
        sharing_rule: Option<&str>,
        family_mode_enabled: bool,
        encryption_key: Option<&str>,
    ) -> Result<Profile> {
        let id = Uuid::new_v4().to_string();

        let password_hash = password.map(|p| p.to_string());
        let encryption_salt = Some(Uuid::new_v4().to_string());
        let normalized_sharing_rule = ProfileSharingRule::from_str(sharing_rule.unwrap_or("private")).as_str().to_string();
        let profile_encryption_key = encryption_key.map(|key| {
            if key.trim().is_empty() {
                Uuid::new_v4().to_string()
            } else {
                key.to_string()
            }
        });
        let created_at = Utc::now().naive_utc();

        info!("Creating new profile '{}' with family_mode_enabled={} and sharing_rule={}", name, family_mode_enabled, normalized_sharing_rule);

        self.repository
            .create_profile_internal(
                id,
                name.to_string(),
                password_hash,
                encryption_salt,
                profile_encryption_key,
                normalized_sharing_rule,
                family_mode_enabled,
                created_at,
            )
            .await
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Profile> {
        self.repository.get_profile(profile_id)
    }

    pub fn list_profiles(&self) -> Result<Vec<Profile>> {
        self.repository.list_profiles()
    }

    pub async fn update_profile_sharing(
        &self,
        profile_id: &str,
        sharing_rule: Option<&str>,
        family_mode_enabled: bool,
        encryption_key: Option<&str>,
    ) -> Result<Profile> {
        let normalized_sharing_rule = ProfileSharingRule::from_str(sharing_rule.unwrap_or("private")).as_str().to_string();
        let profile_encryption_key = encryption_key.map(|key| {
            if key.trim().is_empty() {
                None
            } else {
                Some(key.to_string())
            }
        })
        .flatten();

        info!("Updating share settings for profile '{}' to family_mode_enabled={} sharing_rule={}", profile_id, family_mode_enabled, normalized_sharing_rule);

        self.repository
            .update_profile_share_settings_internal(
                profile_id.to_string(),
                normalized_sharing_rule,
                family_mode_enabled,
                profile_encryption_key,
            )
            .await
    }

    pub async fn create_share(
        &self,
        owner_profile_id: &str,
        shared_profile_id: &str,
        resource_type: &str,
        resource_id: &str,
        permissions: &str,
    ) -> Result<ProfileShare> {
        if owner_profile_id == shared_profile_id {
            warn!("Refusing to create an identity share for profile '{}'", owner_profile_id);
            return Err(AppError::Validation(crate::errors::ValidationError::InvalidInput(
                "A profile cannot share a resource with itself".to_string(),
            )));
        }

        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now().naive_utc();

        info!("Granting share from profile '{}' to '{}' for resource '{}'/'{}'", owner_profile_id, shared_profile_id, resource_type, resource_id);

        self.repository
            .create_share_internal(
                id,
                owner_profile_id.to_string(),
                shared_profile_id.to_string(),
                resource_type.to_string(),
                resource_id.to_string(),
                permissions.to_string(),
                created_at,
            )
            .await
    }

    pub fn list_shares_for_profile(&self, profile_id: &str) -> Result<Vec<ProfileShare>> {
        self.repository.get_shares_for_profile(profile_id)
    }
}
