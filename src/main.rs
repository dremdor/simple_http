use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use std::net::SocketAddr;
use tracing::{info, warn};
use tracing_subscriber::FmtSubscriber;
use clap::Parser;

// Структура описывающаяя данные в формате Json, для каждого заказа
// Добавление атрибутов: 
//  Debug - для форматирования значений
//  Serialize - для конвертирования из структуры в Json
//  Deserialize - для конвертирования из json в структуру
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
// Структура для парсинга аргументов командой строки,
//  в нашем случае это url базы данных с логином, паролем и адресом
//  и адрес самого сервера
// Добавление атрибутов для 
#[derive(Parser, Debug)]
struct Args {
    // Атрибуты флагов:
    //  - short, можно использовать как -- -d <>
    //  - long, можно использовать как -- --db_url <>
    #[arg(short, long)]
    db_url: String,

    //  - short, можно использовать как -- -a <>
    //  - long, можно использовать как -- --addr <>
    #[arg(short, long, default_value = "127.0.0.1:3000")]
    addr: String,
}
// Добавление атрибута для поддержки выполнения асинхронного кодa
#[tokio::main]
async fn main() {
    // Создание подписчика для логирования
    // c параметром логирования на уровне INFO и ниже(Warn, error)
    // логи можно настроить под разные нужды,
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    // Инициализация нашего подписчика как глобального в программе
    // Аналогичный вариант это tracing_subscriber::fmt::init()
    // Использующий по умолчанию максимальный уровень INFO,
    // простые логи для логирования в консоль
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    // обработка аргументов командной строки, имеет встроенный парсер
    // сохраняющий все поля в отдельную структуру
    let args = Args::parse();

    // логирование о попытке подключения к БД 
    info!("Connecting to database: {}", args.db_url);

    // создание объекта , содержащего пул соединений к БД
    // позволяет поддерживать несколько активных соединений к БД
    // по умолчанию 10, на сам расширяется при необходимости
    let pool = PgPool::connect(&args.db_url)
        .await
        .expect("Failed to connect to the database");

    // пул наших соединений оборачивается в Arc,
    // атомарный счетчик ссылок, гарантирует, что объект,
    // в нашем случае соединение будет активно, пока на него есть 
    // ссылка, и что когда ссылок не будет оно станет неактивным
    // обеспечивает неизменяемость
    let shared_pool = Arc::new(pool);

    // иницализация маршрутов
    // каждый маршрут имеет доступ к общему пулу соединений,
    // и внутри каждого обработчика этот пул можно извлечь
    // через параметр Extension(pool)
    let app = Router::new()
        .route("/orders/:order_uid", get(get_order))
        .route("/orders", post(post_order))
        .layer(Extension(shared_pool));
    // инициализация адреса через парсинг аргументов командной строки
    let addr: SocketAddr = args.addr.parse().expect("Invalid address");
    // логирование о начале работы сервера
    info!("Listening on {}", addr);
    // запуск сервера
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Асинхронная функция для получения заказа по уникальному айди
// Первый аргумент передается через Extension обернутый в Arc
// А второй аргумент мы получаем из пути по которому идет запрос,
// замечу, что порядок аргументов важен для обработчиков, т.к.
// внутреннее устройство запускает извлечение параметров слева направо
// что может спровоцировать Handler error
async fn get_order(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(order_uid): Path<String>,
)-> Result<Json<Order>, (StatusCode, String)> {
    // логирование при получении запроса
    info!("Received get request for order: {}", order_uid);
    // получение данных из базы данных по ключу
    // реализация запроса сделана не через макрос, чтобы иметь
    // в случае чего возможность использовать данный сервис и для
    // других баз данных, надо лишь добавить их десериализацию через
    // структуры
    let row = sqlx::query(
        r#"SELECT data FROM orders_json WHERE order_uid = $1"#,
    )
    .bind(&order_uid)
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
        warn!("Database query failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?;
    // попытка получить данные из тела запроса
    let data: serde_json::Value = row
        .try_get("data")
        .unwrap_or_default();
    // парсинг полученных данных в json
    let order: Order = serde_json::from_value(data).map_err(|e| {
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

// Асинхронная функция для post запросов, пул соединений передается
// аналогично предыдущей функции, и также передается Json который на
// этапе захода в функцию десериализуется в структуру, Т.к. сервис
// достаточно простой, было выбрано решение хранить всю Json целиком,
// а в качестве ключа использовать order_uid, в дальнейшем можно использовать
// redis для кеширования наиболее часто запрашиваемых полей
async fn post_order(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(order): Json<Order>,
    ) -> Result<(StatusCode, String), (StatusCode, String)> {
    // логирование получения post запроса 
    info!("Received post request for order: {}", order.order_uid);
    // попытка вставки в бд т.к. я не использую виртуальное окружение
    // каждый аргумент биндим в аргументы запроса, а также сериализуем нашу
    // структуру обратно в json, т.к. тип данных в БД jsonb
    sqlx::query(
        r#"INSERT INTO orders_json (order_uid, data)
        VALUES ($1, $2);"#,
    )
    .bind(&order.order_uid)
    .bind(serde_json::to_value(&order).unwrap())
    .execute(&*pool)
    .await
    .map_err(|e| {
        warn!("Failed to post order {}: {}", order.order_uid, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to insert order".to_string(),
        )
    })?;
    // Вывод дополнительного сообщения помимо кода успеха
    // в случае успешной записи в БД
    info!("Order {} posted successfully", order.order_uid);
    Ok((StatusCode::OK, format!("Order {} posted successfully", order.order_uid)))
}
