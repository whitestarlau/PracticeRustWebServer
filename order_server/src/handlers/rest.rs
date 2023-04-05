use axum::{extract::{State, Path}, http::StatusCode, response::Html, Json};

use sqlx::PgPool;

use crate::{
    db_access::db::{add_new_order_from_db, get_all_orders_from_db},
    models::order::{AddOrder, AddOrderResult, Order},
};

pub async fn health_handler() -> Html<&'static str> {
    Html("<h1>Order server health ok.</h1>")
}

pub async fn get_all_orders(
    State(pool): State<PgPool>,
    Path(user_id): Path<u32>,
) -> Result<axum::Json<Vec<Order>>, (StatusCode, String)> {
    println!("get_all_orders user_id: {}", user_id);
    get_all_orders_from_db(&pool).await
}

pub async fn add_new_order(
    State(pool): State<PgPool>,
    Json(data): Json<AddOrder>,
) -> Result<axum::Json<AddOrderResult>, (StatusCode, String)> {
    //TODO 此处插入数据合法性校验
    add_new_order_from_db(&pool, data).await
}
