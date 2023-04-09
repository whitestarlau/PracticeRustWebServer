use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
};

use sqlx::PgPool;

use crate::{
    db_access::db::{query_inventory_change_from_db, query_inventory_from_db},
    models::inventory::{Inventory, InventoryChange, QueryRequest},
};

pub async fn health_handler() -> Html<&'static str> {
    Html("<h1>Order server health ok.</h1>")
}

pub async fn query_inventory(
    State(pool): State<PgPool>,
    Query(query_params): Query<QueryRequest>,
) -> Result<axum::Json<Inventory>, (StatusCode, String)> {
    // println!("get_all_orders user_id: {}", query_params.user_id);
    let db_result = query_inventory_from_db(&pool, query_params.id).await;

    match db_result {
        Ok(mut result_vec) => {
            return result_vec
                .pop()
                .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "err".to_string()))
                .map(map_ok_result);
        }
        Err(err) => {
            return Err(err);
        }
    }
}

/**
 * 查询库存改变的记录
 */
pub async fn query_inventory_change_history(
    State(pool): State<PgPool>,
    Query(query_params): Query<QueryRequest>,
) -> Result<axum::Json<Vec<InventoryChange>>, (StatusCode, String)> {
    // println!("get_all_orders user_id: {}", query_params.user_id);
    let result = query_inventory_change_from_db(&pool, query_params.id)
        .await
        .map(map_ok_result);

    return result;
}

pub fn map_ok_result<T>(r: T) -> axum::Json<T> {
    axum::Json(r)
}
