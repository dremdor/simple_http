# Simple HTTP Server with PostgreSQL and Axum

## Установка
### 1
  - Установка зависимостей:\
    -- Rust и менеджер пакетов Cargo\
    -- PostgreSQL должен быть открыт и запущен\
    -- необходимые зависимости\
  ```
  cargo install sqlx-cli --no-default-features --features native-tls,postgres
  ```
### 2
  - Создание базы данных через cli
  ```
  CREATE DATABASE order_db;
  ```
  - Подключение и создание таблицы
  ```
  psql -U <ваш_пользователь> -d order_db

  ```
  ```
  CREATE TABLE orders_json (
      order_uid VARCHAR PRIMARY KEY,
      data JSONB
  );
  ```

### 3 
  - Запуск сервера
  ```
  cargo run -- --db-url postgres://db_admin:12345@localhost/order_db --addr 127.0.0.1:3000
  ``` 
  Параметры:
    --db-url: строка подключения к базе данных PostgreSQL.
    --addr: IP-адрес и порт, на которых сервер будет слушать входящие запросы.
### 4
  - Примеры запросов
  get-запрос по uid заказа:
  ```
  curl http://127.0.0.1:3000/orders/<order_uid>
  ```
  post-запрос c json в теле:
  ```
  curl -X POST http://127.0.0.1:3000/orders \
  -H "Content-Type: application/json" \
  -d '{"order_uid": "123", "track_number": "TR12345", "entry": "web", ...}'

  ```
### 5
  - Запуск тестового скрипта:
  ```
  source venv/bin/activate
  ```
  ```
  python3 test.py
  ```
