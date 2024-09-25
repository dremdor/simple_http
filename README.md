# Simple HTTP Server with PostgreSQL and Axum

## Установка
###1
  - Установка зависимостей:
    -- Rust и менеджер пакетов Cargo
    -- PostgreSQL должен быть открыт и запущен
    -- необходимые зависимости
  ```
  cargo install sqlx-cli --no-default-features --features native-tls,postgres
  ```
###2
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

###3 
  - Запуск сервера, обработка двух аргументов командной строки,
  db_url базы данных и адрес сервера addr
  ```

  ``` 

