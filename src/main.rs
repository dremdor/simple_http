use axum::{http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
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

    //иницализация маршрутов
    let app = Router::new().route(
        "/orders/:order_uid",
        get(|order_uid| async move { get_order(pool.clone(), order_uid).await }),
    );
    //    .route("/order/:id", get(get_order_by_id));
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

// обработчик корня, возвращаемый тип данных
async fn root_handler() -> Result<Json<Order>, Json<serde_json::Value>> {
    // логирование запросов
    info!("Received request to root handler");

    // пробная обработка json из файла
    let mut file = match File::open("model.json").await {
        Ok(f) => f,
        Err(e) => {
            warn!("Cannot open file: {}", e);
            return Err(Json(json!({"error": "Cannot open file"})));
        }
    };

    // чтение даты из файла в строку
    let mut data = String::new();
    match file.read_to_string(&mut data).await {
        Ok(_) => {}
        Err(e) => {
            warn!("Cannot read file: {}", e);
            return Err(Json(json!({"error": "Cannot read file"})));
        }
    }
    // парсинг полученной строки в json
    let order: Order = match serde_json::from_str(&data) {
        Ok(o) => o,
        Err(e) => {
            warn!("Cannot parse JSON: {}", e);
            return Err(Json(json!({"error": "Cannot parse JSON"})));
        }
    };

    Ok(Json(order))
}

async fn get_order(pool: PgPool, order_uid: String) -> Result<Json<Order>, (StatusCode, String)> {
    // получение данных из базы данных по ключу
    let row = sqlx::query!(
        r#"SELECT data FROM orders_json WHERE data->>'order_uid' = $1"#,
        order_uid
    )
    .fetch_one(&pool)
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
    // отправление json по запросу
    Ok(Json(order))
}
