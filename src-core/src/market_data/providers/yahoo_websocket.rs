use crate::market_data::market_data_model::{DataSource, LivePrice};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc, TimeZone};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use prost::Message;
use rust_decimal::Decimal;
use std::str::FromStr;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

pub mod yahoo_proto {
    include!(concat!(env!("OUT_DIR"), "/yahoo.rs"));
}

pub struct YahooWebsocket {
    tx: mpsc::Sender<LivePrice>,
    symbols: Vec<String>,
}

impl YahooWebsocket {
    pub fn new(tx: mpsc::Sender<LivePrice>, symbols: Vec<String>) -> Self {
        Self { tx, symbols }
    }

    pub async fn run(&self) {
        let url = "wss://streamer.finance.yahoo.com";
        let (ws_stream, _) = match connect_async(url).await {
            Ok(s) => s,
            Err(e) => {
                error!("Yahoo Websocket connection failed: {}", e);
                return;
            }
        };

        info!("Connected to Yahoo WebSocket");
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to symbols
        let subscribe_msg = format!("{{\"subscribe\": {:?}}}", self.symbols);
        if let Err(e) = write.send(WsMessage::Text(subscribe_msg.into())).await {
            error!("Failed to send Yahoo subscribe message: {}", e);
            return;
        }

        while let Some(msg) = read.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    // Yahoo sends base64 encoded protobufs
                    let bytes = match general_purpose::STANDARD.decode(text.trim()) {
                        Ok(b) => b,
                        Err(e) => {
                            error!("Yahoo Websocket Base64 decode error: {}", e);
                            continue;
                        }
                    };

                    match yahoo_proto::PricingData::decode(&*bytes) {
                        Ok(data) => {
                            let dt = Utc.timestamp_millis_opt(data.time).single().unwrap_or(Utc::now());
                            let price = match Decimal::from_f32_retain(data.price) {
                                Some(p) => p,
                                None => Decimal::from_str(&data.price.to_string()).unwrap_or_default(),
                            };

                            let live_price = LivePrice {
                                symbol: data.id,
                                price,
                                timestamp: dt,
                                provider: DataSource::Yahoo,
                            };

                            debug!("Yahoo live price: {:?}", live_price);
                            let _ = self.tx.send(live_price).await;
                        }
                        Err(e) => {
                            error!("Yahoo Websocket Protobuf decode error: {}", e);
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Yahoo Websocket closed");
                    break;
                }
                Err(e) => {
                    error!("Yahoo Websocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }
}
