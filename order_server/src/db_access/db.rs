use axum::http::StatusCode;
use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;

use crate::models::{
    error::internal_error,
    order::{AddOrder, AddOrderResult, Order},
};

pub async fn get_all_orders_from_db(
    pool: &PgPool,
    user_id: i64,
    page: i64,
    page_size: i64,
) -> Result<Vec<Order>, (StatusCode, String)> {
    let offset = page_size * page;
    let orders = sqlx::query!(
        "SELECT * FROM orders WHERE user_id = $1 LIMIT $2 OFFSET $3",
        user_id,
        page_size,
        offset
    )
    .map({
        |row| Order {
            id: row.id,
            user_id: row.user_id,
            items_id: serde_json::from_str(row.items_id.as_str()).unwrap_or_default(),
            price: serde_json::from_str(row.price.as_str()).unwrap_or_default(),
            total_price: row.total_price,
            currency: row.currency.unwrap_or_default(),
            sub_time: NaiveDateTime::from(row.sub_time.unwrap()).timestamp_millis(),
            pay_time: NaiveDateTime::from(row.pay_time.unwrap()).timestamp_millis(),
            description: row.description,
        }
    })
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    println!("get_all_orders_from_db size: {}", orders.len());

    Ok(orders)
}

pub async fn add_new_order_from_db(
    pool: &PgPool,
    data: AddOrder,
) -> Result<AddOrderResult, (StatusCode, String)> {
    let des = data.description.unwrap_or_default();
    let price = data.price;

    println!("add_new_order des: {}", des);

    let item_ids_str = serde_json::to_string(&data.items_id).unwrap_or_default();
    let price_json = serde_json::to_string(&price).unwrap_or_default();

    let ts_1970 = NaiveDateTime::from_timestamp_opt(0, 0).unwrap_or_default();

    let _rows = sqlx::query!("INSERT INTO orders (user_id, items_id, price, total_price, currency, pay_time, description) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        data.user_id,
        item_ids_str, price_json,
        data.total_price, data.currency,
        ts_1970, des)
        .fetch_one(pool)
        .await
        .map_err(internal_error);

    let result = AddOrderResult {
        description: "add successed.".to_string(),
    };
    Ok(result)
}
