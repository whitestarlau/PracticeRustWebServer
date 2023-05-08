use axum::{
    extract::State,
    http::StatusCode,
    response::Html, Json,
};

use sqlx::PgPool;

use crate::{models::order::{AddOrder, AddOrderResult, Order}, db_access::db::{get_all_orders_from_db, add_new_order_from_db}};

pub async fn health_handler() -> Html<&'static str> {
    Html("<h1>Order server health ok.</h1>")
}

pub async fn query_inventory(
    State(pool): State<PgPool>,
) -> Result<axum::Json<Vec<Order>>, (StatusCode, String)> {
    get_all_orders_from_db(&pool).await
}

pub async fn add_new_order(
    State(pool): State<PgPool>,
    Json(data): Json<AddOrder>,
) -> Result<axum::Json<AddOrderResult>, (StatusCode, String)> {
    //TODO 此处插入数据合法性校验
    add_new_order_from_db(&pool,data).await
}