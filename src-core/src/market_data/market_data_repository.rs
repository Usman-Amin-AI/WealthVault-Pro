use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use log::{debug, error};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::market_data_errors::MarketDataError;
use super::market_data_model::{
    LatestQuotePair, MarketDataProviderSetting, Quote, QuoteDb, UpdateMarketDataProviderSetting,
};
use super::market_data_traits::MarketDataRepositoryTrait;
use crate::db::{get_connection, write_actor::{spawn_writer, WriteHandle}};
use crate::errors::Result;
use crate::schema::quotes::dsl::{quotes, symbol, timestamp};
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel::sqlite::Sqlite;

// Import for daily_account_valuation table
use super::market_data_constants::{DATA_SOURCE_MANUAL, DATA_SOURCE_YAHOO};
use crate::schema::daily_account_valuation::dsl as dav_dsl;
use crate::schema::market_data_providers::dsl as market_data_providers_dsl;

pub struct MarketDataRepository {
    pool: Arc<Pool<ConnectionManager<SqliteConnection>>>,
    writer: WriteHandle,
}

impl MarketDataRepository {
    pub fn new(pool: Arc<Pool<ConnectionManager<SqliteConnection>>>, writer: WriteHandle) -> Self {
        Self { pool, writer }
    }
}

#[async_trait]
impl MarketDataRepositoryTrait for MarketDataRepository {
    fn get_all_historical_quotes(&self) -> Result<Vec<Quote>> {
        let mut conn = get_connection(&self.pool)?;

        Ok(quotes
            .order(timestamp.desc())
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?
            .into_iter()
            .map(Quote::from)
            .collect())
    }

    fn get_historical_quotes_for_symbol(&self, input_symbol: &str) -> Result<Vec<Quote>> {
        let mut conn = get_connection(&self.pool)?;

        Ok(quotes
            .filter(symbol.eq(input_symbol))
            .order(timestamp.desc())
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?
            .into_iter()
            .map(Quote::from)
            .collect())
    }

    async fn save_quotes(&self, input_quotes: &[Quote]) -> Result<()> {
        if input_quotes.is_empty() {
            return Ok(());
        }
        let quotes_owned: Vec<Quote> = input_quotes.to_vec();
        let db_rows: Vec<QuoteDb> = quotes_owned.iter().map(QuoteDb::from).collect();

        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<()> {
                for chunk in db_rows.chunks(1_000) {
                    diesel::replace_into(quotes)
                        .values(chunk)
                        .execute(conn)
                        .map_err(MarketDataError::DatabaseError)?;
                }
                Ok(())
            })
            .await
    }

    async fn save_quote(&self, quote: &Quote) -> Result<Quote> {
        let quote_cloned = quote.clone();
        let save_result = self.save_quotes(std::slice::from_ref(&quote_cloned)).await;
        save_result?;
        Ok(quote_cloned)
    }

    async fn delete_quote(&self, quote_id: &str) -> Result<()> {
        let id_to_delete = quote_id.to_string();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<()> {
                diesel::delete(quotes.filter(crate::schema::quotes::dsl::id.eq(id_to_delete)))
                    .execute(conn)
                    .map_err(MarketDataError::DatabaseError)?;
                Ok(())
            })
            .await
    }

    async fn delete_quotes_for_symbols(&self, symbols_to_delete: &[String]) -> Result<()> {
        let symbols_owned = symbols_to_delete.to_vec();

        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<()> {
                diesel::delete(quotes.filter(symbol.eq_any(symbols_owned)))
                    .execute(conn)
                    .map_err(MarketDataError::DatabaseError)?;
                Ok(())
            })
            .await
    }

    fn get_quotes_by_source(&self, input_symbol: &str, source: &str) -> Result<Vec<Quote>> {
        let mut conn = get_connection(&self.pool)?;
        use crate::schema::quotes::dsl::symbol as symbol_col;

        Ok(quotes
            .filter(symbol_col.eq(input_symbol))
            .filter(crate::schema::quotes::dsl::data_source.eq(source))
            .order(timestamp.asc())
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?
            .into_iter()
            .map(Quote::from)
            .collect())
    }

    fn get_latest_quote_for_symbol(&self, input_symbol: &str) -> Result<Quote> {
        let mut conn = get_connection(&self.pool)?;

        let query_result = quotes
            .filter(symbol.eq(input_symbol))
            .order(timestamp.desc())
            .first::<QuoteDb>(&mut conn)
            .optional();

        match query_result {
            Ok(Some(quote_db)) => Ok(Quote::from(quote_db)),
            Ok(None) => Err(MarketDataError::NotFound(format!(
                "No quote found in database for symbol: {}",
                input_symbol
            ))
            .into()),
            Err(diesel_err) => Err(MarketDataError::DatabaseError(diesel_err).into()),
        }
    }

    fn get_latest_quotes_for_symbols(
        &self,
        input_symbols: &[String],
    ) -> Result<HashMap<String, Quote>> {
        if input_symbols.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = get_connection(&self.pool)?;

        let placeholders = input_symbols
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "WITH RankedQuotes AS ( \
                SELECT \
                    q.*, \
                    ROW_NUMBER() OVER (PARTITION BY q.symbol ORDER BY q.timestamp DESC) as rn \
                FROM quotes q WHERE q.symbol IN ({}) \
            ) \
            SELECT * FROM RankedQuotes WHERE rn = 1 \
            ORDER BY symbol",
            placeholders
        );

        let mut query_builder = Box::new(sql_query(sql)).into_boxed::<Sqlite>();

        for symbol_val in input_symbols {
            query_builder = query_builder.bind::<Text, _>(symbol_val);
        }

        let ranked_quotes_db: Vec<QuoteDb> = query_builder
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?;

        let result: HashMap<String, Quote> = ranked_quotes_db
            .into_iter()
            .map(|quote_db| (quote_db.symbol.clone(), quote_db.into()))
            .collect();

        Ok(result)
    }

    fn get_latest_quotes_pair_for_symbols(
        &self,
        input_symbols: &[String],
    ) -> Result<HashMap<String, LatestQuotePair>> {
        if input_symbols.is_empty() {
            return Ok(HashMap::new());
        }

        let mut conn = get_connection(&self.pool)?;

        let placeholders = input_symbols
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "WITH RankedQuotes AS ( \
                SELECT \
                    q.*, \
                    ROW_NUMBER() OVER (PARTITION BY q.symbol ORDER BY q.timestamp DESC) as rn \
                FROM quotes q  WHERE q.symbol IN ({}) \
            ) \
            SELECT * \
            FROM RankedQuotes \
            WHERE rn <= 2 \
            ORDER BY symbol, rn",
            placeholders
        );

        let mut query_builder = Box::new(sql_query(sql)).into_boxed::<Sqlite>();

        for symbol_val in input_symbols {
            query_builder = query_builder.bind::<Text, _>(symbol_val);
        }

        let ranked_quotes_db: Vec<QuoteDb> = query_builder
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?;

        let mut result_map: HashMap<String, LatestQuotePair> = HashMap::new();
        let mut current_symbol_quotes: Vec<Quote> = Vec::new();

        for quote_db in ranked_quotes_db {
            let quote = Quote::from(quote_db);

            if current_symbol_quotes.is_empty() || quote.symbol == current_symbol_quotes[0].symbol {
                current_symbol_quotes.push(quote);
            } else {
                if !current_symbol_quotes.is_empty() {
                    let latest_quote = current_symbol_quotes.remove(0);
                    let previous_quote = if !current_symbol_quotes.is_empty() {
                        Some(current_symbol_quotes.remove(0))
                    } else {
                        None
                    };
                    result_map.insert(
                        latest_quote.symbol.clone(),
                        LatestQuotePair {
                            latest: latest_quote,
                            previous: previous_quote,
                        },
                    );
                }
                current_symbol_quotes.clear();
                current_symbol_quotes.push(quote);
            }
        }

        if !current_symbol_quotes.is_empty() {
            let latest_quote = current_symbol_quotes.remove(0);
            let previous_quote = if !current_symbol_quotes.is_empty() {
                Some(current_symbol_quotes.remove(0))
            } else {
                None
            };
            result_map.insert(
                latest_quote.symbol.clone(),
                LatestQuotePair {
                    latest: latest_quote,
                    previous: previous_quote,
                },
            );
        }

        Ok(result_map)
    }

    fn get_historical_quotes_for_symbols_in_range(
        &self,
        input_symbols: &HashSet<String>,
        start_date_naive: NaiveDate,
        end_date_naive: NaiveDate,
    ) -> Result<Vec<Quote>> {
        if input_symbols.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = get_connection(&self.pool)?;

        let start_datetime_utc: DateTime<Utc> =
            Utc.from_utc_datetime(&start_date_naive.and_hms_opt(0, 0, 0).unwrap());
        let end_datetime_utc: DateTime<Utc> =
            Utc.from_utc_datetime(&end_date_naive.and_hms_opt(23, 59, 59).unwrap());

        let start_str = start_datetime_utc.to_rfc3339();
        let end_str = end_datetime_utc.to_rfc3339();

        let symbols_vec: Vec<String> = input_symbols.iter().cloned().collect();

        Ok(quotes
            .filter(symbol.eq_any(symbols_vec))
            .filter(timestamp.ge(start_str))
            .filter(timestamp.le(end_str))
            .order(timestamp.asc())
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?
            .into_iter()
            .map(Quote::from)
            .collect())
    }

    fn get_all_historical_quotes_for_symbols(
        &self,
        symbols: &HashSet<String>,
    ) -> Result<Vec<Quote>> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = get_connection(&self.pool)?;
        let symbols_vec: Vec<String> = symbols.iter().cloned().collect();

        Ok(quotes
            .filter(symbol.eq_any(symbols_vec))
            .order(timestamp.asc())
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?
            .into_iter()
            .map(Quote::from)
            .collect())
    }

    fn get_all_historical_quotes_for_symbols_by_source(
        &self,
        symbols: &HashSet<String>,
        source: &str,
    ) -> Result<Vec<Quote>> {
        if symbols.is_empty() {
            return Ok(Vec::new());
        }
        let mut conn = get_connection(&self.pool)?;
        let symbols_vec: Vec<String> = symbols.iter().cloned().collect();

        Ok(quotes
            .filter(symbol.eq_any(symbols_vec))
            .filter(crate::schema::quotes::dsl::data_source.eq(source))
            .order(timestamp.asc())
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?
            .into_iter()
            .map(Quote::from)
            .collect())
    }

    fn get_latest_sync_dates_by_source(&self) -> Result<HashMap<String, Option<NaiveDateTime>>> {
        let mut conn = get_connection(&self.pool)?;

        let latest_calculated_at_str: Option<String> = dav_dsl::daily_account_valuation
            .select(diesel::dsl::max(dav_dsl::calculated_at))
            .first::<Option<String>>(&mut conn)
            .optional()?
            .flatten();

        let latest_sync_naive_datetime: Option<NaiveDateTime> =
            latest_calculated_at_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt_utc| dt_utc.naive_utc())
            });

        let mut sync_dates_map: HashMap<String, Option<NaiveDateTime>> = HashMap::new();

        sync_dates_map.insert(DATA_SOURCE_YAHOO.to_string(), latest_sync_naive_datetime);
        sync_dates_map.insert(DATA_SOURCE_MANUAL.to_string(), latest_sync_naive_datetime);

        Ok(sync_dates_map)
    }

    fn get_all_providers(&self) -> Result<Vec<MarketDataProviderSetting>> {
        let mut conn = get_connection(&self.pool)?;
        market_data_providers_dsl::market_data_providers
            .order(market_data_providers_dsl::priority.desc())
            .select(MarketDataProviderSetting::as_select())
            .load::<MarketDataProviderSetting>(&mut conn)
            .map_err(|e| MarketDataError::DatabaseError(e).into())
    }

    fn get_provider_by_id(&self, provider_id_input: &str) -> Result<MarketDataProviderSetting> {
        let mut conn = get_connection(&self.pool)?;
        market_data_providers_dsl::market_data_providers
            .find(provider_id_input)
            .select(MarketDataProviderSetting::as_select())
            .first::<MarketDataProviderSetting>(&mut conn)
            .map_err(|e| MarketDataError::DatabaseError(e).into())
    }

    async fn update_provider_settings(
        &self,
        provider_id_input: String,
        changes: UpdateMarketDataProviderSetting,
    ) -> Result<MarketDataProviderSetting> {
        self.writer
            .exec(
                move |conn: &mut SqliteConnection| -> Result<MarketDataProviderSetting> {
                    diesel::update(
                        market_data_providers_dsl::market_data_providers.find(&provider_id_input),
                    )
                    .set(&changes)
                    .get_result(conn)
                    .map_err(|e| MarketDataError::DatabaseError(e).into())
                },
            )
            .await
    }

    // --- Quote Import Methods ---

    async fn bulk_insert_quotes(&self, quote_records: Vec<QuoteDb>) -> Result<usize> {
        if quote_records.is_empty() {
            return Ok(0);
        }

        let quotes_owned = quote_records.clone();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<usize> {
                let mut total_inserted = 0;
                for chunk in quotes_owned.chunks(1000) {
                    total_inserted += diesel::insert_into(quotes)
                        .values(chunk)
                        .execute(conn)
                        .map_err(MarketDataError::DatabaseError)?;
                }
                Ok(total_inserted)
            })
            .await
    }

    async fn bulk_update_quotes(&self, quote_records: Vec<QuoteDb>) -> Result<usize> {
        if quote_records.is_empty() {
            return Ok(0);
        }

        let quotes_owned = quote_records.clone();
        self.writer
            .exec(move |conn: &mut SqliteConnection| -> Result<usize> {
                let mut total_updated = 0;
                for chunk in quotes_owned.chunks(1000) {
                    total_updated += diesel::replace_into(quotes)
                        .values(chunk)
                        .execute(conn)
                        .map_err(MarketDataError::DatabaseError)?;
                }
                Ok(total_updated)
            })
            .await
    }

    async fn bulk_upsert_quotes(&self, quote_records: Vec<Quote>) -> Result<usize> {
        debug!(
            "🚀 REPOSITORY: bulk_upsert_quotes called with {} quotes",
            quote_records.len()
        );

        if quote_records.is_empty() {
            debug!("⚠️ No quotes to insert, returning 0");
            return Ok(0);
        }

        debug!(
            "🔄 Converting {} Quote structs to QuoteDb structs",
            quote_records.len()
        );
        let quotes_owned = quote_records.clone();
        let db_rows: Vec<QuoteDb> = quotes_owned.iter().map(QuoteDb::from).collect();
        debug!("✅ Converted to {} QuoteDb records", db_rows.len());
        debug!(
            "🎯 Sample QuoteDb: id={}, symbol={}, timestamp={}, data_source={}",
            db_rows[0].id, db_rows[0].symbol, db_rows[0].timestamp, db_rows[0].data_source
        );

        debug!("💾 Executing database transaction...");
        let result = self
            .writer
            .exec(move |conn: &mut SqliteConnection| -> Result<usize> {
                debug!("🔄 Inside database transaction");
                let mut total_upserted = 0;
                let chunk_size = 1000;
                let total_chunks = db_rows.len().div_ceil(chunk_size);

                debug!(
                    "📦 Processing {} quotes in {} chunks of {}",
                    db_rows.len(),
                    total_chunks,
                    chunk_size
                );

                for (chunk_index, chunk) in db_rows.chunks(chunk_size).enumerate() {
                    debug!(
                        "💾 Processing chunk {}/{} with {} quotes",
                        chunk_index + 1,
                        total_chunks,
                        chunk.len()
                    );

                    let count = diesel::replace_into(quotes)
                        .values(chunk)
                        .execute(conn)
                        .map_err(|e| {
                            error!(
                                "❌ Database error in chunk {}/{}: {}",
                                chunk_index + 1,
                                total_chunks,
                                e
                            );
                            MarketDataError::DatabaseError(e)
                        })?;

                    debug!(
                        "✅ Chunk {}/{} inserted {} records",
                        chunk_index + 1,
                        total_chunks,
                        count
                    );
                    total_upserted += count;
                }

                debug!(
                    "✅ Transaction completed successfully, total upserted: {}",
                    total_upserted
                );
                Ok(total_upserted)
            })
            .await;

        match result {
            Ok(count) => {
                debug!(
                    "✅ REPOSITORY: bulk_upsert_quotes completed successfully, upserted {} quotes",
                    count
                );
                Ok(count)
            }
            Err(e) => {
                error!("❌ REPOSITORY: bulk_upsert_quotes failed: {}", e);
                Err(e)
            }
        }
    }

    fn quote_exists(&self, symbol_param: &str, date: &str) -> Result<bool> {
        let mut conn = get_connection(&self.pool)?;

        let count: i64 = quotes
            .filter(crate::schema::quotes::dsl::symbol.eq(symbol_param))
            .filter(crate::schema::quotes::dsl::timestamp.like(format!("{}%", date)))
            .count()
            .get_result(&mut conn)
            .map_err(MarketDataError::DatabaseError)?;

        Ok(count > 0)
    }

    fn get_existing_quotes_for_period(
        &self,
        symbol_param: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<Quote>> {
        let mut conn = get_connection(&self.pool)?;

        Ok(quotes
            .filter(crate::schema::quotes::dsl::symbol.eq(symbol_param))
            .filter(crate::schema::quotes::dsl::timestamp.ge(start_date))
            .filter(crate::schema::quotes::dsl::timestamp.le(end_date))
            .load::<QuoteDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?
            .into_iter()
            .map(Quote::from)
            .collect())
    }

    async fn save_live_prices(&self, prices: &[crate::market_data::market_data_model::LivePrice]) -> Result<()> {
        if prices.is_empty() {
            return Ok(());
        }
        use crate::schema::live_prices_cache;
        let db_records: Vec<crate::market_data::market_data_model::LivePriceDb> = prices.iter().map(|p| p.into()).collect();

        self.writer.exec(move |conn: &mut SqliteConnection| -> Result<()> {
            diesel::replace_into(live_prices_cache::table)
                .values(&db_records)
                .execute(conn)
                .map_err(MarketDataError::DatabaseError)?;
            Ok(())
        }).await
    }

    fn get_live_prices(&self, symbols_list: &[String]) -> Result<HashMap<String, crate::market_data::market_data_model::LivePrice>> {
        if symbols_list.is_empty() {
            return Ok(HashMap::new());
        }
        let mut conn = get_connection(&self.pool)?;
        use crate::schema::live_prices_cache::dsl::*;

        let records: Vec<crate::market_data::market_data_model::LivePriceDb> = live_prices_cache
            .filter(symbol.eq_any(symbols_list.to_vec()))
            .load::<crate::market_data::market_data_model::LivePriceDb>(&mut conn)
            .map_err(MarketDataError::DatabaseError)?;

        let mut map = HashMap::new();
        for record in records {
            map.insert(record.symbol.clone(), record.into());
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::write_actor::WriteHandle;
    use chrono::Utc;
    use diesel::r2d2::{ConnectionManager, Pool};
    use diesel::sqlite::SqliteConnection;
    use rust_decimal_macros::dec;
    use std::sync::Arc;
    use tempfile::tempdir;

    async fn create_test_repository(
    ) -> Result<(
        MarketDataRepository,
        Pool<ConnectionManager<SqliteConnection>>,
        tempfile::TempDir,
    )> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db_path_str = db_path.to_str().unwrap().to_string();

        let manager = ConnectionManager::<SqliteConnection>::new(&db_path_str);
        let pool = Pool::builder().max_size(4).build(manager)?;
        crate::db::run_migrations(&pool)?;
        let writer = spawn_writer(pool.clone());
        let repository = MarketDataRepository::new(Arc::new(pool.clone()), writer);

        Ok((repository, pool, temp_dir))
    }

    #[tokio::test]
    async fn test_save_and_get_live_prices_persists_to_db() -> Result<()> {
        let (repository, _, _temp_dir) = create_test_repository().await?;

        let live_price = crate::market_data::market_data_model::LivePrice {
            symbol: "AAPL".to_string(),
            price: dec!(173.45),
            timestamp: Utc::now(),
            provider: crate::market_data::market_data_model::DataSource::Yahoo,
        };

        repository.save_live_prices(&[live_price.clone()]).await?;

        let symbols = vec!["AAPL".to_string()];
        let live_prices = repository.get_live_prices(&symbols)?;

        assert_eq!(live_prices.len(), 1);
        let loaded_price = live_prices.get("AAPL").expect("Live price not found");
        assert_eq!(loaded_price.symbol, live_price.symbol);
        assert_eq!(loaded_price.provider, live_price.provider);
        assert_eq!(loaded_price.price, live_price.price);
        assert!(loaded_price.timestamp <= Utc::now());

        Ok(())
    }

    #[tokio::test]
    async fn test_save_live_prices_upserts_existing_entry() -> Result<()> {
        let (repository, _, _temp_dir) = create_test_repository().await?;

        let first_price = crate::market_data::market_data_model::LivePrice {
            symbol: "MSFT".to_string(),
            price: dec!(300.15),
            timestamp: Utc::now(),
            provider: crate::market_data::market_data_model::DataSource::Yahoo,
        };

        repository.save_live_prices(&[first_price.clone()]).await?;

        let updated_price = crate::market_data::market_data_model::LivePrice {
            symbol: "MSFT".to_string(),
            price: dec!(305.20),
            timestamp: Utc::now(),
            provider: crate::market_data::market_data_model::DataSource::Alpaca,
        };

        repository.save_live_prices(&[updated_price.clone()]).await?;

        let symbols = vec!["MSFT".to_string()];
        let live_prices = repository.get_live_prices(&symbols)?;
        let loaded_price = live_prices.get("MSFT").expect("Live price not found after update");

        assert_eq!(loaded_price.price, updated_price.price);
        assert_eq!(loaded_price.provider, updated_price.provider);
        assert!(loaded_price.timestamp >= first_price.timestamp);

        Ok(())
    }
}
