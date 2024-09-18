use axum::{response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Serialize, Deserialize)]
struct Order {
    order_uid: String,
    track_number: String,
    entry: String,
    delivery: DeliveryInfo,
    payment: PaymentInfo,
    items: Vec<ItemsInfo>,
    locale: String,
    internal_signature: String,
    customer_id: String,
    delivery_service: String,
    shardkey: String,
    sm_id: u64,
    date_created: String,
    oof_shard: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct DeliveryInfo {
    name: String,
    phone: String,
    zip: String,
    city: String,
    address: String,
    region: String,
    email: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct PaymentInfo {
    transaction: String,
    request_id: String,
    currency: String,
    provider: String,
    amount: u32,
    payment_dt: i64,
    bank: String,
    delivery_cost: u32,
    goods_total: u32,
    custom_fee: u32,
}
#[derive(Debug, Serialize, Deserialize)]
struct ItemsInfo {
    chrt_id: u64,
    track_number: String,
    price: u32,
    rid: String,
    name: String,
    sale: u32,
    size: String,
    total_price: u32,
    nm_id: u64,
    brand: String,
    status: u32,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let app = Router::new().route("/", get(root_handler));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_handler() -> Result<Json<Order>, Json<serde_json::Value>> {
    info!("Received request to root handler");

    let mut file = match File::open("model.json").await {
        Ok(f) => f,
        Err(e) => {
            warn!("Cannot open file: {}", e);
            return Err(Json(json!({"error": "Cannot open file"})));
        }
    };

    let mut data = String::new();
    match file.read_to_string(&mut data).await {
        Ok(_) => {}
        Err(e) => {
            warn!("Cannot read file: {}", e);
            return Err(Json(json!({"error": "Cannot read file"})));
        }
    }

    let order: Order = match serde_json::from_str(&data) {
        Ok(o) => o,
        Err(e) => {
            warn!("Cannot parse JSON: {}", e);
            return Err(Json(json!({"error": "Cannot parse JSON"})));
        }
    };

    Ok(Json(order))
}
