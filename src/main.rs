use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_embed::ServeEmbed;
use futures::stream::StreamExt;
use futures::SinkExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};

mod database;
mod influxdb;
mod models;
mod prober;
mod routes;

#[derive(rust_embed::Embed, Clone)]
#[folder = "frontend/dist"]
struct Frontend;

#[derive(Clone)]
struct AppState {
    db: Arc<DatabaseConnection>,
    influx_client: Arc<influxdb2::Client>,
    influx_config: Arc<influxdb::InfluxConfig>,
    tx: Arc<broadcast::Sender<String>>,
}

#[tokio::main]
async fn main() {
    println!("Starting smokeping-rs application...");

    let db = database::setup_database().await.unwrap_or_else(|e| {
        eprintln!("Failed to setup database: {:?}", e);
        std::process::exit(1);
    });
    let db = Arc::new(db);

    let (influx_client, influx_config) = influxdb::setup_influxdb().await.unwrap_or_else(|e| {
        eprintln!("Failed to setup InfluxDB: {:?}", e);
        std::process::exit(1);
    });
    let influx_client = Arc::new(influx_client);
    let influx_config = Arc::new(influx_config);

    let (tx, _) = broadcast::channel(100);
    let tx = Arc::new(tx);

    let state = AppState {
        db: db.clone(),
        influx_client: influx_client.clone(),
        influx_config: influx_config.clone(),
        tx: tx.clone(),
    };

    let targets = models::target::Entity::find()
        .filter(models::target::Column::IsActive.eq(true))
        .all(db.as_ref())
        .await
        .unwrap();

    for target in targets {
        let prober_state = state.clone();
        tokio::spawn(prober::run_prober(
            target,
            prober_state.influx_client.as_ref().clone(),
            prober_state.influx_config.bucket.clone(),
            prober_state.tx.as_ref().clone(),
        ));
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_router = Router::new()
        .route("/targets", get(routes::targets::list_targets).post(routes::targets::create_target))
        .route("/targets/:id", get(routes::targets::get_target).put(routes::targets::update_target).delete(routes::targets::delete_target))
        .route("/targets/:id/data", get(routes::targets::get_probe_data));

    let app = Router::new()
        .nest("/api", api_router)
        .route("/ws", get(ws_handler))
        .fallback_service(ServeEmbed::<Frontend>::new())
        .layer(cors)
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&bind_addr).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state.tx))
}

async fn handle_socket(socket: WebSocket, tx: Arc<broadcast::Sender<String>>) {
    let (mut sender, _) = socket.split();
    let mut rx = tx.subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });
}