use std::f32::consts::E;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    Json,
};

use common_lib::{internal_error, internal_error_dyn, validate_payload};

use sqlx::PgPool;
use tracing::{info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    db_access::db::{query_goods_summary_list, query_goods_detail},
    models::goods::{GoodsDetail, GoodsSummary, QueryRequest, QueryDetailRequest},
};

#[instrument]
pub async fn health_handler() -> Html<&'static str> {
    println!("some one call health check api.");
    Html("<h1>Goods server health ok.</h1>")
}

#[instrument]
pub async fn get_goods_summary(
    State(pool): State<PgPool>,
    Query(query_params): Query<QueryRequest>,
) -> Result<axum::Json<Vec<GoodsSummary>>, (StatusCode, String)> {
    return query_goods_summary_list(&pool, query_params.page, query_params.page_size)
        .await
        .map(map_ok_result);
}

/**
 *
 */
#[instrument]
pub async fn get_goods_detail(
    State(pool): State<PgPool>,
    Query(query_params): Query<QueryDetailRequest>,
) -> Result<axum::Json<GoodsDetail>, (StatusCode, String)> {
    return query_goods_detail(&pool, query_params.goods_id)
        .await
        .map(map_ok_result);
}

pub fn map_ok_result<T>(r: T) -> axum::Json<T> {
    axum::Json(r)
}