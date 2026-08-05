use crate::market_data::market_data_model::{DataSource, LivePrice};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

pub struct AlpacaWebsocket {
    tx: mpsc::Sender<LivePrice>,
    symbols: Vec<String>,
    api_key: String,
    api_secret: String,
}

impl AlpacaWebsocket {
    pub fn new(tx: mpsc::Sender<LivePrice>, symbols: Vec<String>, api_key: String, api_secret: String) -> Self {
        Self { tx, symbols, api_key, api_secret }
    }

    pub async fn run(&self) {
        let url = "wss://stream.data.alpaca.markets/v2/iex";
        let (ws_stream, _) = match connect_async(url).await {
            Ok(s) => s,
            Err(e) => {
                error!("Alpaca Websocket connection failed: {}", e);
                return;
            }
        };

        info!("Connected to Alpaca WebSocket");
        let (mut write, mut read) = ws_stream.split();

        // Auth
        let auth_msg = format!("{{\"action\": \"auth\", \"key\": \"{}\", \"secret\": \"{}\"}}", self.api_key, self.api_secret);
        if let Err(e) = write.send(WsMessage::Text(auth_msg.into())).await {
            error!("Failed to send Alpaca auth message: {}", e);
            return;
        }

        // Subscribe
        // "trades": ["AAPL"], "quotes": ["AMD", "CLDR"] etc.
        let subscribe_msg = serde_json::json!({
            "action": "subscribe",
            "trades": self.symbols
        });
        if let Err(e) = write.send(WsMessage::Text(subscribe_msg.to_string().into())).await {
            error!("Failed to send Alpaca subscribe message: {}", e);
            return;
        }

        while let Some(msg) = read.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    let parsed: Result<Value, _> = serde_json::from_str(&text);
                    if let Ok(Value::Array(items)) = parsed {
                        for item in items {
                            if let Some(msg_type) = item.get("T").and_then(|t| t.as_str()) {
                                if msg_type == "t" { // Trade message
                                    if let (Some(sym), Some(price_val), Some(ts)) = (
                                        item.get("S").and_then(|s| s.as_str()),
                                        item.get("p").and_then(|p| p.as_f64()),
                                        item.get("t").and_then(|t| t.as_str())
                                    ) {
                                        let dt = DateTime::parse_from_rfc3339(ts)
                                            .map(|d| d.with_timezone(&Utc))
                                            .unwrap_or_else(|_| Utc::now());

                                        let price = match Decimal::from_f64_retain(price_val) {
                                            Some(p) => p,
                                            None => Decimal::from_str(&price_val.to_string()).unwrap_or_default(),
                                        };

                                        let live_price = LivePrice {
                                            symbol: sym.to_string(),
                                            price,
                                            timestamp: dt,
                                            provider: DataSource::Alpaca,
                                        };

                                        debug!("Alpaca live price: {:?}", live_price);
                                        let _ = self.tx.send(live_price).await;
                                    }
                                } else if msg_type == "error" {
                                    error!("Alpaca Websocket returned error: {:?}", item);
                                }
                            }
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Alpaca Websocket closed");
                    break;
                }
                Err(e) => {
                    error!("Alpaca Websocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }
}
