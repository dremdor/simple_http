use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug)]
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
#[derive(Debug)]
struct DeliveryInfo {
    name: String,
    phone: String,
    zip: String,
    city: String,
    address: String,
    region: String,
    email: String,
}
#[derive(Debug)]
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
#[derive(Debug)]
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
    let app = Router::new().route("/", get(root_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root_handler() -> &'static str {
    "Hello world"
}
