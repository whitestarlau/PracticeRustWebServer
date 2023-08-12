use axum::{async_trait, extract::FromRequest};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
// use validator::Validate;

use axum::{extract::FromRequestParts, http::HeaderMap};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GoodsSummary {
    pub id: i32,
    pub goods_name: String,
    pub goods_image: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct QueryRequest {
    pub page_size: i64,
    pub page: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct QueryDetailRequest {
    pub goods_id: i32,
}

/**
 * unit_price 单价。单位分。
 */
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GoodsDetail {
    pub id: i32,
    pub goods_name: String,
    pub goods_image: String,
    pub unit_price: i32,
    pub goods_des: String,
    pub inventory_count: i32,
}
