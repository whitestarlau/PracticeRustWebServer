use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;

use crate::{
    handlers::rest::*,
    handlers::grpc::*,
    multiplexservice::MultiplexService,
};

#[path = "../db_access/mod.rs"]
mod db_access;
#[path = "../handlers/mod.rs"]
mod handlers;

#[path = "../models/mod.rs"]
mod models;

#[path = "../multiplex_service.rs"]
mod multiplexservice;

#[tokio::main]
async fn main() {
    dotenv().ok();

    //set database pool
    //设置数据库连接池。restful和grpc服务各用一个，不用考虑生命周期标注会比较简单。
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let db_pool = PgPoolOptions::new().connect(&database_url).await.unwrap();
    let db_pool2 = PgPoolOptions::new().connect(&database_url).await.unwrap();

    // build our application with a route
    let rest = Router::new()
        .route("/", get(health_handler))
        .route("/orders/:user_id", get(get_all_orders))
        .route("/add_order", post(add_new_order))
        .with_state(db_pool);

    let grpc = get_grpc_router(db_pool2);

    // combine them into one service 
    // 将rest和grpc两种路由合并到一起
    let service = MultiplexService::new(rest, grpc);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        // .serve(rest.into_make_service())
        .serve(tower::make::Shared::new(service))
        .await
        .unwrap();
}
