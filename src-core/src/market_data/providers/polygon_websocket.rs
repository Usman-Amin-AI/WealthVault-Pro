use crate::market_data::market_data_model::{DataSource, LivePrice};
use chrono::{DateTime, Utc, TimeZone};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

pub struct PolygonWebsocket {
    tx: mpsc::Sender<LivePrice>,
    symbols: Vec<String>,
    api_key: String,
}

impl PolygonWebsocket {
    pub fn new(tx: mpsc::Sender<LivePrice>, symbols: Vec<String>, api_key: String) -> Self {
        Self { tx, symbols, api_key }
    }

    pub async fn run(&self) {
        let url = "wss://delayed.polygon.io/stocks";
        let (ws_stream, _) = match connect_async(url).await {
            Ok(s) => s,
            Err(e) => {
                error!("Polygon Websocket connection failed: {}", e);
                return;
            }
        };

        info!("Connected to Polygon WebSocket");
        let (mut write, mut read) = ws_stream.split();

        // Auth
        let auth_msg = format!("{{\"action\":\"auth\",\"params\":\"{}\"}}", self.api_key);
        if let Err(e) = write.send(WsMessage::Text(auth_msg.into())).await {
            error!("Failed to send Polygon auth message: {}", e);
            return;
        }

        // Subscribe to Trades (T.*)
        let mut subs = Vec::new();
        for sym in &self.symbols {
            subs.push(format!("T.{}", sym));
        }
        let subscribe_msg = format!("{{\"action\":\"subscribe\", \"params\":\"{}\"}}", subs.join(","));

        if let Err(e) = write.send(WsMessage::Text(subscribe_msg.into())).await {
            error!("Failed to send Polygon subscribe message: {}", e);
            return;
        }

        while let Some(msg) = read.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    let parsed: Result<Value, _> = serde_json::from_str(&text);
                    if let Ok(Value::Array(items)) = parsed {
                        for item in items {
                            if let Some(msg_type) = item.get("ev").and_then(|t| t.as_str()) {
                                if msg_type == "T" { // Trade message
                                    if let (Some(sym), Some(price_val), Some(ts_millis)) = (
                                        item.get("sym").and_then(|s| s.as_str()),
                                        item.get("p").and_then(|p| p.as_f64()),
                                        item.get("t").and_then(|t| t.as_i64())
                                    ) {
                                        let dt = Utc.timestamp_millis_opt(ts_millis).single().unwrap_or(Utc::now());

                                        let price = match Decimal::from_f64_retain(price_val) {
                                            Some(p) => p,
                                            None => Decimal::from_str(&price_val.to_string()).unwrap_or_default(),
                                        };

                                        let live_price = LivePrice {
                                            symbol: sym.to_string(),
                                            price,
                                            timestamp: dt,
                                            provider: DataSource::Polygon,
                                        };

                                        debug!("Polygon live price: {:?}", live_price);
                                        let _ = self.tx.send(live_price).await;
                                    }
                                } else if msg_type == "status" {
                                    if let Some(status) = item.get("status").and_then(|s| s.as_str()) {
                                        if status == "auth_failed" {
                                            error!("Polygon Websocket auth failed");
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Polygon Websocket closed");
                    break;
                }
                Err(e) => {
                    error!("Polygon Websocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }
}
