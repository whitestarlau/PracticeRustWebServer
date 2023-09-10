use std::f32::consts::E;

use axum::http::StatusCode;
use common_lib::{internal_error, internal_error_dyn};
use sqlx::postgres::PgPool;
use tracing::info;

use crate::models::goods::{self, GoodsDetail, GoodsSummary};

pub async fn query_goods_summary_list(
    pool: &PgPool,
    page: i64,
    page_size: i64,
) -> Result<Vec<GoodsSummary>, (StatusCode, String)> {
    let offset = page_size * page;

    let goods = sqlx::query!(
        "SELECT * FROM goods_summary LIMIT $1 OFFSET $2",
        page_size,
        offset
    )
    .map({
        |row| GoodsSummary {
            id: row.id,
            goods_name: row.name.unwrap_or_default(),
            goods_image: row.image.unwrap_or_default(),
        }
    })
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    // info!("get_user size: {}", users);

    Ok(goods)
}

pub async fn query_goods_detail(
    pool: &PgPool,
    goods_id: i32,
) -> Result<GoodsDetail, (StatusCode, String)> {
    println!("query_goods_detail id: {}", goods_id);

    let goods_detail = sqlx::query!("SELECT * FROM goods_detail where id = $1", goods_id)
        .map({
            |row| {
                let price = row.unit_price;
                GoodsDetail {
                    id: row.id,
                    goods_name: row.name.unwrap_or_default(),
                    goods_image: row.image.unwrap_or_default(),
                    unit_price: price,
                    goods_des: row.des.unwrap_or_default(),
                    inventory_count: 0,
                }
            }
        })
        .fetch_one(pool)
        .await
        .map_err(internal_error)?;

    Ok(goods_detail)
}
