use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use binance::config::Config;
use binance::websockets::WebSockets;
use binance::ws_model::{CombinedStreamEvent, WebsocketEventUntag};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{error, info};

/// Thin wrapper around the binance-rs-async websocket client to stream kline data.
pub struct MarketStream {
    keep_running: Arc<AtomicBool>,
}

impl MarketStream {
    pub fn new() -> Self {
        Self {
            keep_running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn stop(&self) {
        self.keep_running.store(false, Ordering::Relaxed);
    }

    /// Start a combined websocket stream for the provided symbols (1m klines).
    pub async fn start_stream(
        &self,
        symbols: Vec<String>,
        tx: UnboundedSender<CombinedStreamEvent<WebsocketEventUntag>>,
    ) {
        self.keep_running.store(true, Ordering::Relaxed);
        let keep_running = self.keep_running.clone();
        let conf = websocket_config_from_env();
        let ws_base = conf.ws_endpoint.clone();
        let streams: Vec<String> = symbols
            .into_iter()
            .map(|s| format!("{}@kline_1m", s.to_lowercase()))
            .collect();

        tokio::spawn(async move {
            let mut web_socket: WebSockets<'static, CombinedStreamEvent<WebsocketEventUntag>> =
                WebSockets::new_with_options(
                    move |event: CombinedStreamEvent<WebsocketEventUntag>| {
                        if let Err(send_err) = tx.send(event) {
                            error!("Failed to forward websocket event: {}", send_err);
                        }
                        Ok(())
                    },
                    conf,
                );

            info!(
                "Connecting to Binance websockets: {:?} (base: {})",
                streams, ws_base
            );
            if let Err(e) = web_socket.connect_multiple(streams).await {
                error!("WebSocket connection error: {:?}", e);
                return;
            }

            if let Err(e) = web_socket.event_loop(&keep_running).await {
                error!("WebSocket event loop error: {:?}", e);
            }

            if let Err(e) = web_socket.disconnect().await {
                error!("WebSocket disconnect error: {:?}", e);
            }
            info!("WebSocket disconnected");
        });
    }
}

fn websocket_config_from_env() -> Config {
    let mut conf = Config::default();
    if let Ok(custom) = std::env::var("BINANCE_WS_ENDPOINT") {
        conf = conf.set_ws_endpoint(custom);
    } else if std::env::var("BINANCE_US").is_ok() {
        conf = conf.set_ws_endpoint("wss://stream.binance.us:9443");
    }
    conf
}
