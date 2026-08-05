use crate::market_data::market_data_constants::{
    DATA_SOURCE_ALPACA, DATA_SOURCE_ALPHA_VANTAGE, DATA_SOURCE_MANUAL,
    DATA_SOURCE_MARKET_DATA_APP, DATA_SOURCE_METAL_PRICE_API, DATA_SOURCE_POLYGON,
    DATA_SOURCE_YAHOO,
};
use crate::schema::quotes;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{expression::AsExpression, sql_types::Text};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(
    Queryable, Identifiable, Selectable, Debug, Clone, Serialize, Deserialize, PartialEq, Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub id: String,
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub adjclose: Decimal,
    pub volume: Decimal,
    pub currency: String,
    pub data_source: DataSource,
    pub created_at: DateTime<Utc>,
}

#[derive(
    Queryable,
    Identifiable,
    Selectable,
    Insertable,
    AsChangeset,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    QueryableByName,
)]
#[diesel(table_name = quotes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(rename_all = "camelCase")]
pub struct QuoteDb {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub symbol: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub timestamp: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub open: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub high: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub low: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub close: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub adjclose: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub volume: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub currency: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub data_source: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub created_at: String,
}

// Conversion implementations
impl From<QuoteDb> for Quote {
    fn from(db: QuoteDb) -> Self {
        let parse_datetime = |s: &str| -> DateTime<Utc> {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now())
        };

        Quote {
            id: db.id,
            symbol: db.symbol,
            timestamp: parse_datetime(&db.timestamp),
            open: Decimal::from_str(&db.open).unwrap_or_default(),
            high: Decimal::from_str(&db.high).unwrap_or_default(),
            low: Decimal::from_str(&db.low).unwrap_or_default(),
            close: Decimal::from_str(&db.close).unwrap_or_default(),
            adjclose: Decimal::from_str(&db.adjclose).unwrap_or_default(),
            volume: Decimal::from_str(&db.volume).unwrap_or_default(),
            data_source: DataSource::from(db.data_source.as_ref()),
            created_at: parse_datetime(&db.created_at),
            currency: db.currency,
        }
    }
}

impl From<&Quote> for QuoteDb {
    fn from(quote: &Quote) -> Self {
        QuoteDb {
            id: quote.id.clone(),
            symbol: quote.symbol.clone(),
            timestamp: quote.timestamp.to_rfc3339(),
            open: quote.open.to_string(),
            high: quote.high.to_string(),
            low: quote.low.to_string(),
            close: quote.close.to_string(),
            adjclose: quote.adjclose.to_string(),
            volume: quote.volume.to_string(),
            currency: quote.currency.clone(),
            data_source: quote.data_source.as_str().to_string(),
            created_at: quote.created_at.to_rfc3339(),
        }
    }
}

impl From<&LivePrice> for Quote {
    fn from(live_price: &LivePrice) -> Self {
        Quote {
            id: format!("live_{}", live_price.symbol.to_ascii_lowercase()),
            symbol: live_price.symbol.clone(),
            timestamp: live_price.timestamp,
            open: live_price.price,
            high: live_price.price,
            low: live_price.price,
            close: live_price.price,
            adjclose: live_price.price,
            volume: Decimal::ZERO,
            currency: "USD".to_string(),
            data_source: live_price.provider.clone(),
            created_at: live_price.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_provider_ids_map_to_their_data_sources() {
        assert_eq!(DataSource::from("ALPACA").as_str(), DATA_SOURCE_ALPACA);
        assert_eq!(DataSource::from("POLYGON").as_str(), DATA_SOURCE_POLYGON);
        assert_eq!(DataSource::from("YAHOO").as_str(), DATA_SOURCE_YAHOO);
    }

    #[test]
    fn live_price_converts_to_quote_without_historical_fallback() {
        let live_price = LivePrice {
            symbol: "AAPL".to_string(),
            price: Decimal::from_str("187.25").unwrap(),
            timestamp: Utc::now(),
            provider: DataSource::Alpaca,
        };

        let quote = Quote::from(&live_price);

        assert_eq!(quote.symbol, live_price.symbol);
        assert_eq!(quote.close, live_price.price);
        assert_eq!(quote.data_source, DataSource::Alpaca);
        assert_eq!(quote.currency, "USD");
    }
}

/// Summary model for quote search results
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSummary {
    pub exchange: String,
    pub short_name: String,
    pub quote_type: String,
    pub symbol: String,
    pub index: String,
    pub score: f64,
    pub type_display: String,
    pub long_name: String,
}

#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub symbol: String,
    pub data_source: DataSource,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsExpression, Default)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "UPPERCASE")]
pub enum DataSource {
    Yahoo,
    MarketDataApp,
    AlphaVantage,
    MetalPriceApi,
    Alpaca,
    Polygon,
    #[default]
    Manual,
}

impl DataSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataSource::Yahoo => DATA_SOURCE_YAHOO,
            DataSource::MarketDataApp => DATA_SOURCE_MARKET_DATA_APP,
            DataSource::AlphaVantage => DATA_SOURCE_ALPHA_VANTAGE,
            DataSource::MetalPriceApi => DATA_SOURCE_METAL_PRICE_API,
            DataSource::Alpaca => DATA_SOURCE_ALPACA,
            DataSource::Polygon => DATA_SOURCE_POLYGON,
            DataSource::Manual => DATA_SOURCE_MANUAL,
        }
    }
}

impl From<DataSource> for String {
    fn from(source: DataSource) -> Self {
        source.as_str().to_string()
    }
}

impl From<&str> for DataSource {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            DATA_SOURCE_YAHOO => DataSource::Yahoo,
            DATA_SOURCE_MARKET_DATA_APP => DataSource::MarketDataApp,
            DATA_SOURCE_ALPHA_VANTAGE => DataSource::AlphaVantage,
            DATA_SOURCE_METAL_PRICE_API => DataSource::MetalPriceApi,
            DATA_SOURCE_ALPACA => DataSource::Alpaca,
            DATA_SOURCE_POLYGON => DataSource::Polygon,
            _ => DataSource::Manual,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LatestQuotePair {
    pub latest: Quote,
    pub previous: Option<Quote>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MarketDataProviderInfo {
    pub id: String,
    pub name: String,
    pub logo_filename: String,
    pub last_synced_date: Option<chrono::DateTime<chrono::Utc>>,
}

// --- Added for MarketDataProviderSetting ---

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Queryable,
    Identifiable,
    Selectable,
    Insertable,
    AsChangeset,
)]
#[diesel(table_name = crate::schema::market_data_providers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[serde(rename_all = "camelCase")]
pub struct MarketDataProviderSetting {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub logo_filename: Option<String>,
    pub last_synced_at: Option<String>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::market_data_providers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UpdateMarketDataProviderSetting {
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

// --- Quote Import Models ---

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteImport {
    pub symbol: String,
    pub date: String, // ISO format YYYY-MM-DD
    pub open: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    pub close: Decimal, // Required field
    pub volume: Option<Decimal>,
    pub currency: String,
    pub validation_status: ImportValidationStatus,
    pub error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ImportValidationStatus {
    Valid,
    Warning(String),
    Error(String),
}

// --- Live Market Data Models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePrice {
    pub symbol: String,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub provider: DataSource,
}

#[derive(
    Queryable,
    Identifiable,
    Selectable,
    Insertable,
    AsChangeset,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = crate::schema::live_prices_cache)]
#[diesel(primary_key(symbol))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LivePriceDb {
    pub symbol: String,
    pub price: String,
    pub timestamp: String,
    pub provider: String,
}

impl From<LivePriceDb> for LivePrice {
    fn from(db: LivePriceDb) -> Self {
        let parse_datetime = |s: &str| -> DateTime<Utc> {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now())
        };
        LivePrice {
            symbol: db.symbol,
            price: Decimal::from_str(&db.price).unwrap_or_default(),
            timestamp: parse_datetime(&db.timestamp),
            provider: DataSource::from(db.provider.as_ref()),
        }
    }
}

impl From<&LivePrice> for LivePriceDb {
    fn from(live_price: &LivePrice) -> Self {
        LivePriceDb {
            symbol: live_price.symbol.clone(),
            price: live_price.price.to_string(),
            timestamp: live_price.timestamp.to_rfc3339(),
            provider: live_price.provider.as_str().to_string(),
        }
    }
}
