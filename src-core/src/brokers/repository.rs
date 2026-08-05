use chrono::Utc;
use diesel::prelude::*;

use crate::db::{get_connection, DbPool};
use crate::errors::{AppError, DatabaseError};
use crate::schema::broker_connections::dsl::*;
use crate::schema::accounts;

use super::models::BrokerConnection;

pub struct BrokerConnectionsRepository;

impl BrokerConnectionsRepository {
    pub fn get_by_account_id(
        pool: &DbPool,
        acc_id: &str,
        profile_id_param: &str,
    ) -> Result<Option<BrokerConnection>, AppError> {
        let mut conn = get_connection(pool)?;
        let result = broker_connections
            .inner_join(accounts::table.on(account_id.eq(accounts::id)))
            .filter(account_id.eq(acc_id))
            .filter(accounts::profile_id.eq(profile_id_param))
            .select(broker_connections::all_columns())
            .first::<BrokerConnection>(&mut conn)
            .optional()
            .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))?;
        Ok(result)
    }

    pub fn insert_or_update(
        pool: &DbPool,
        connection: BrokerConnection,
        profile_id: &str,
    ) -> Result<BrokerConnection, AppError> {
        let mut conn = get_connection(pool)?;
        let now = Utc::now().naive_utc();

        let existing = Self::get_by_account_id(pool, &connection.account_id, profile_id)?;
        if let Some(mut ext) = existing {
            ext.access_token = connection.access_token;
            ext.refresh_token = connection.refresh_token;
            ext.expires_at = connection.expires_at;
            ext.updated_at = now;

            let updated = diesel::update(broker_connections.filter(id.eq(&ext.id)))
                .set(&ext)
                .get_result(&mut conn)
                .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))?;
            Ok(updated)
        } else {
            let inserted = diesel::insert_into(broker_connections)
                .values(&connection)
                .get_result(&mut conn)
                .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))?;
            Ok(inserted)
        }
    }

    pub fn delete_by_account_id(pool: &DbPool, acc_id: &str, profile_id: &str) -> Result<(), AppError> {
        let mut conn = get_connection(pool)?;
        if let Some(existing) = Self::get_by_account_id(pool, acc_id, profile_id)? {
            diesel::delete(broker_connections.filter(id.eq(existing.id)))
                .execute(&mut conn)
                .map_err(|e| AppError::Database(DatabaseError::QueryFailed(e)))?;
        }
        Ok(())
    }
}
