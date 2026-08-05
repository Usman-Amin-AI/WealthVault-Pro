use super::profiles_model::{NewProfile, Profile, ProfileShare, NewProfileShare};
use crate::db::write_actor::WriteHandle;
use crate::errors::{AppError, DatabaseError, Result};
use crate::schema::{profile_shares, profiles};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use r2d2::Pool;
use diesel::r2d2::ConnectionManager;
use std::sync::Arc;

pub struct ProfileRepository {
    pool: Arc<Pool<ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl ProfileRepository {
    pub fn new(pool: Arc<Pool<ConnectionManager<SqliteConnection>>>, writer: WriteHandle) -> Self {
        Self { pool, writer }
    }

    pub async fn create_profile_internal(
        &self,
        id: String,
        name: String,
        password_hash: Option<String>,
        encryption_salt: Option<String>,
        encryption_key: Option<String>,
        sharing_rule: String,
        family_mode_enabled: bool,
        created_at: chrono::NaiveDateTime,
    ) -> Result<Profile> {
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<Profile> {
                let new_profile = NewProfile {
                    id: &id,
                    name: &name,
                    password_hash: password_hash.as_deref(),
                    encryption_salt: encryption_salt.as_deref(),
                    encryption_key: encryption_key.as_deref(),
                    sharing_rule: &sharing_rule,
                    family_mode_enabled,
                    created_at,
                };

                diesel::insert_into(profiles::table)
                    .values(&new_profile)
                    .execute(conn)
                    .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))?;

                profiles::table
                    .find(&id)
                    .first(conn)
                    .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))
            })
            .await
    }

    pub fn get_profile(&self, profile_id: &str) -> Result<Profile> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(DatabaseError::PoolCreationFailed(e)))?;

        profiles::table
            .find(profile_id)
            .first(&mut conn)
            .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))
    }

    pub fn list_profiles(&self) -> Result<Vec<Profile>> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(DatabaseError::PoolCreationFailed(e)))?;

        profiles::table
            .load(&mut conn)
            .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))
    }

    pub async fn update_profile_share_settings_internal(
        &self,
        profile_id: String,
        sharing_rule: String,
        family_mode_enabled: bool,
        encryption_key: Option<String>,
    ) -> Result<Profile> {
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<Profile> {
                diesel::update(profiles::table.find(&profile_id))
                    .set((
                        profiles::sharing_rule.eq(&sharing_rule),
                        profiles::family_mode_enabled.eq(family_mode_enabled),
                        profiles::encryption_key.eq(encryption_key.as_deref()),
                    ))
                    .execute(conn)
                    .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))?;

                profiles::table
                    .find(&profile_id)
                    .first(conn)
                    .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))
            })
            .await
    }

    pub async fn create_share_internal(
        &self,
        id: String,
        owner_profile_id: String,
        shared_profile_id: String,
        resource_type: String,
        resource_id: String,
        permissions: String,
        created_at: chrono::NaiveDateTime,
    ) -> Result<ProfileShare> {
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<ProfileShare> {
                let new_share = NewProfileShare {
                    id: &id,
                    owner_profile_id: &owner_profile_id,
                    shared_profile_id: &shared_profile_id,
                    resource_type: &resource_type,
                    resource_id: &resource_id,
                    permissions: &permissions,
                    created_at,
                };

                diesel::insert_into(profile_shares::table)
                    .values(&new_share)
                    .execute(conn)
                    .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))?;

                profile_shares::table
                    .find(&id)
                    .first(conn)
                    .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))
            })
            .await
    }

    pub fn get_shares_for_profile(&self, profile_id: &str) -> Result<Vec<ProfileShare>> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| AppError::Database(DatabaseError::PoolCreationFailed(e)))?;

        profile_shares::table
            .filter(profile_shares::shared_profile_id.eq(profile_id))
            .load(&mut conn)
            .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))
    }
}
