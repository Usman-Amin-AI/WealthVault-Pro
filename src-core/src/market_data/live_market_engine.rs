use crate::market_data::market_data_model::LivePrice;
use dashmap::DashMap;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use super::providers::yahoo_websocket::YahooWebsocket;
use super::providers::alpaca_websocket::AlpacaWebsocket;
use super::providers::polygon_websocket::PolygonWebsocket;
use crate::market_data::market_data_model::LivePriceDb;
use crate::market_data::market_data_traits::MarketDataRepositoryTrait;

#[derive(Clone)]
pub struct LiveMarketEngine {
    pub prices: Arc<DashMap<String, LivePrice>>,
    tx: mpsc::Sender<LivePrice>,
    pub broadcast_tx: tokio::sync::broadcast::Sender<LivePrice>,
}

impl LiveMarketEngine {
    pub fn new(repository: Arc<dyn MarketDataRepositoryTrait + Send + Sync>) -> Self {
        let (tx, mut rx) = mpsc::channel::<LivePrice>(1000);
        let (broadcast_tx, _) = tokio::sync::broadcast::channel(100);
        let prices = Arc::new(DashMap::new());
        let prices_clone = prices.clone();
        let btx = broadcast_tx.clone();

        // Background task to update dashmap and SQLite
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            while let Some(live_price) = rx.recv().await {
                prices_clone.insert(live_price.symbol.clone(), live_price.clone());
                buffer.push(live_price.clone());
                
                // Broadcast to subscribers (like Tauri)
                let _ = btx.send(live_price);

                // Simple batching/flushing mechanism (every 50 messages)
                if buffer.len() >= 50 {
                    if let Err(e) = repository.save_live_prices(&buffer).await {
                        error!("Failed to flush live prices to SQLite: {}", e);
                    }
                    buffer.clear();
                }
            }
        });

        Self { prices, tx, broadcast_tx }
    }

    pub fn start_yahoo(&self, symbols: Vec<String>) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let ws = YahooWebsocket::new(tx, symbols);
            ws.run().await;
        });
    }

    pub fn start_alpaca(&self, symbols: Vec<String>, api_key: String, api_secret: String) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let ws = AlpacaWebsocket::new(tx, symbols, api_key, api_secret);
            ws.run().await;
        });
    }

    pub fn start_polygon(&self, symbols: Vec<String>, api_key: String) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let ws = PolygonWebsocket::new(tx, symbols, api_key);
            ws.run().await;
        });
    }
}
