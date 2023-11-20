mod database;
mod server;
mod messages;

use axum::{
    routing::get,
    Router,
};
use messages::BroadcastCommand;
use sqlx::{Sqlite, Pool};
use std::{net::SocketAddr, sync::Arc, collections::HashMap};
use tower_http::{
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::log::{log, Level};
use tokio::sync::{mpsc, RwLock};
use tokio::sync::broadcast::{Sender, channel};

use crate::database::setup_database;
use crate::server::ws_handler;
use crate::messages::DirectCommand;




type SessionMap = Arc<RwLock<HashMap<String, Sender<BroadcastCommand>>>>;

#[derive(Clone)]
pub struct ServerState {
    sessions: SessionMap,
    db: Pool<Sqlite>
}

impl ServerState {
    fn new(db: Pool<Sqlite>) -> Self {
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        Self{
            sessions,
            db
        }
    }

    async fn join_session(&mut self, name: String) -> Sender<BroadcastCommand> {
        if self.sessions.read().await.contains_key(&name) {
            self.sessions.read().await.get(&name).expect("Missing session sender").clone() 
        } else {
            let (sender, _) = channel(10);
            self.sessions.write().await.insert(name, sender.clone());
            sender
        }

    }
}

#[tokio::main]
async fn main() {

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "signal_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = setup_database().await.unwrap();
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        ).with_state(ServerState::new(db));


    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();

}
