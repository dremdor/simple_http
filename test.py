import aiohttp
import asyncio

# Асинхронная функция для отправки post запросов через единую сессию
async def post_order(session, order_id):
    url = "http://localhost:3000/orders"
    json_data = {
        "order_uid": f"b563feb7b2b84b6test{order_id}",
        "track_number": "WBILMTESTTRACK",
        "entry": "WBIL",
        "delivery": {
            "name": "Test Testov",
            "phone": "+9720000000",
            "zip": "2639809",
            "city": "Kiryat Mozkin",
            "address": "Ploshad Mira 15",
            "region": "Kraiot",
            "email": "test@gmail.com",
        },
        "payment": {
            "transaction": f"b563feb7b2b84b6test{order_id}",
            "request_id": "",
            "currency": "USD",
            "provider": "wbpay",
            "amount": 1817,
            "payment_dt": 1637907727,
            "bank": "alpha",
            "delivery_cost": 1500,
            "goods_total": 317,
            "custom_fee": 0,
        },
        "items": [
            {
                "chrt_id": 9934930,
                "track_number": "WBILMTESTTRACK",
                "price": 453,
                "rid": f"ab4219087a764ae0btest{order_id}",
                "name": "Mascaras",
                "sale": 30,
                "size": "0",
                "total_price": 317,
                "nm_id": 2389212,
                "brand": "Vivienne Sabo",
                "status": 202,
            }
        ],
        "locale": "en",
        "internal_signature": "",
        "customer_id": f"test{order_id}",
        "delivery_service": "meest",
        "shardkey": "9",
        "sm_id": 99,
        "date_created": "2021-11-26T06:22:19Z",
        "oof_shard": "1",
    }

    async with session.post(url, json=json_data) as response:
        if response.status == 200:
            print(f"Order {order_id} posted successfully")
        else:
            print(f"Failed to post order {order_id}, status code: {response.status}")

# Асинхронная функция для отправки get запросов через единую сессию
async def get_order(session, order_id):
    url = f"http://localhost:3000/orders/b563feb7b2b84b6test{order_id}"

    async with session.get(url) as response:
        if response.status == 200:
            order_data = await response.json()
            print(f"Order {order_id} data: {order_data}")
        else:
            print(f"Failed to get order {order_id}, status code: {response.status}")


async def main():
    # Открытие единой http сессии
    async with aiohttp.ClientSession() as session:
        post_tasks = [post_order(session, i) for i in range(1, 10001)]
        await asyncio.gather(*post_tasks)
        # Использование gather для параллельного запуска всех задач
        get_tasks = [get_order(session, i) for i in range(1, 10001)]
        await asyncio.gather(*get_tasks)


if __name__ == "__main__":
    asyncio.run(main())
