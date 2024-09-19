use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use std::net::SocketAddr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;

//Структура описывающаяя Json
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
    // Подключение переменных окружения
    dotenv::dotenv().ok();
    //Установка подписчика для логирования
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let pool = PgPool::connect("postgres://db_admin:12345@localhost/order_table")
        .await
        .expect("Failed to connect to database");

    let shared_pool = Arc::new(pool);

    //иницализация маршрутов
    let app = Router::new()
        .route("/orders/:order_uid", get(get_order))
        .layer(Extension(shared_pool));
    // инициализация адреса
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    // логирование о начале работы сервера
    info!("Listening on {}", addr);
    // запуск сервера
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_order(
    Path(order_uid): Path<String>,
    Extension(pool): Extension<Arc<PgPool>>,
)-> Result<Json<Order>, (StatusCode, String)> {
    // логирование при получении запроса
    info!("Received request for order: {}", order_uid);
    // получение данных из базы данных по ключу
    let row = sqlx::query!(
        r#"SELECT data FROM orders_json WHERE order_uid = $1"#,
        order_uid
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
        warn!("Database query failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?;

    // парсинг полученных данных в json
    let order: Order = serde_json::from_value(row.data.unwrap_or_default()).map_err(|e| {
        warn!("Failed to parse JSON {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?;
    //логирование при успехе
    info!("Successfully retrieved data for order: {}", order_uid);
    // отправление json по запросу
    Ok(Json(order))
}
