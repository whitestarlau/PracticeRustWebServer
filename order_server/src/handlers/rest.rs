use std::f32::consts::E;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    Json,
};

use futures::TryFutureExt;
use idgenerator::IdInstance;

use crate::{
    consul_api,
    db_access::db::{add_new_order_from_db, get_all_orders_from_db},
    models::{
        order::{AddOrder, AddOrderResult, GetOrderParams, NewOrderToken, Order},
        state::AppState,
    },
};

pub async fn health_handler() -> Html<&'static str> {
    Html("<h1>Order server health ok.</h1>")
}

pub async fn get_all_orders(
    State(state): State<AppState>,
    Query(query_params): Query<GetOrderParams>,
) -> Result<axum::Json<Vec<Order>>, (StatusCode, String)> {
    // println!("get_all_orders user_id: {}", query_params.user_id);
    get_all_orders_from_db(
        &state.pool,
        query_params.user_id,
        query_params.page,
        query_params.page_size,
    )
    .await
    .map(map_ok_result)
}

/**
 * TODO
 * 生成一个新的token，存入数据库，然后在addOrder的时候我们会校验这个token是否使用过
 */
pub async fn request_new_order_token(
    State(_pool): State<AppState>,
) -> Result<axum::Json<NewOrderToken>, (StatusCode, String)> {
    let id = IdInstance::next_id();
    println!("request_new_order_token: {}", id);
    Ok(axum::Json(NewOrderToken { token: id }))
}

pub async fn add_new_order(
    State(state): State<AppState>,
    Json(data): Json<AddOrder>,
) -> Result<axum::Json<AddOrderResult>, (StatusCode, String)> {
    //TODO 此处插入数据合法性校验

    //从consul获取库存微服务的地址
    let cs = consul_api::consul::Consul::newDefault().map_err(map_consult_error)?;
    let filter = consul_api::model::Filter::ID(state.inventory_srv_id);
    let srv_option = cs.get_service(&filter).await.map_err(map_consult_error)?;
    
    if let Some(srv) = srv_option {
        let inventory_addr = srv.address;
        add_new_order_from_db(&state.pool, &state.local_pool, inventory_addr, data)
        .await
        .map(map_ok_result)
    }else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "cannot found inventory_srv from consul.".to_string(),
        ));
    }
}

pub fn map_ok_result<T>(r: T) -> axum::Json<T> {
    axum::Json(r)
}

pub fn map_consult_error(err: reqwest::Error) -> (StatusCode, String) {
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        "consul error.".to_string(),
    );
}
