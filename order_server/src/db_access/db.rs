use axum::http::StatusCode;
use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;

use crate::models::{error::internal_error, order::{Order, AddOrder, AddOrderResult}};

pub async fn get_all_orders_from_db(
    pool: &PgPool,
) -> Result<axum::Json<Vec<Order>>, (StatusCode, String)> {
    let orders = sqlx::query!(r#"SELECT * FROM orders LIMIT 10"#)
        .map(|row| Order {
            id: row.id,
            items_id: serde_json::from_str(row.items_id.as_str()).unwrap_or_default(),
            price: serde_json::from_str(row.price.as_str()).unwrap_or_default(),
            total_price: row.total_price,
            currency: row.currency.unwrap_or_default(),
            sub_time: NaiveDateTime::from(row.sub_time.unwrap()).timestamp_millis(),
            pay_time: NaiveDateTime::from(row.pay_time.unwrap()).timestamp_millis(),
            description: row.description,
        })
        .fetch_all(pool)
        .await
        .map_err(internal_error)?;

    Ok(axum::Json(orders))
}

pub async fn add_new_order_from_db(
    pool: &PgPool,
    data: AddOrder,
) -> Result<axum::Json<AddOrderResult>, (StatusCode, String)> {
    let des = data.description.unwrap_or_default();
    let price = data.price;

    println!("add_new_order des: {}", des);

    let item_ids_str = serde_json::to_string(&data.items_id).unwrap_or_default();
    let price_json = serde_json::to_string(&price).unwrap_or_default();

    let ts_1970 = NaiveDateTime::from_timestamp_opt(0, 0).unwrap_or_default();

    let _rows = sqlx::query!("INSERT INTO orders (items_id, price, total_price, currency, pay_time, description) VALUES ($1, $2, $3, $4, $5, $6)",
        item_ids_str, price_json,
        data.total_price, data.currency,
        ts_1970, des)
        .fetch_one(pool)
        .await
        .map_err(internal_error);

    let result = AddOrderResult {
        description: "add successed.".to_string(),
    };
    Ok(axum::Json(result))
}
