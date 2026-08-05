use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::broker_connections;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Insertable, AsChangeset)]
#[diesel(table_name = broker_connections)]
pub struct BrokerConnection {
    pub id: String,
    pub account_id: String,
    pub provider: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConnectionDto {
    pub id: String,
    pub account_id: String,
    pub provider: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<BrokerConnection> for BrokerConnectionDto {
    fn from(conn: BrokerConnection) -> Self {
        Self {
            id: conn.id,
            account_id: conn.account_id,
            provider: conn.provider,
            created_at: conn.created_at,
            updated_at: conn.updated_at,
        }
    }
}
