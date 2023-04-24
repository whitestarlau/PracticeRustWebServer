#[macro_use]
extern crate num_derive;

use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use idgenerator::{IdGeneratorOptions, IdInstance};
use std::env;
use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;

use crate::{
    handlers::grpc::*,
    handlers::rest::*,
    models::state::{AppState, InventoryState},
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

    // 雪花算法生成唯一id
    let options = IdGeneratorOptions::new().worker_id(1).worker_id_bit_len(6);
    // Initialize the id generator instance with the option.
    // Other options not set will be given the default value.
    let _ = IdInstance::init(options);
    // Call `next_id` to generate a new unique id.
    // let id = IdInstance::next_id();

    //set database pool
    //设置数据库连接池。restful和grpc服务各用一个，不用考虑生命周期标注会比较简单。
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let local_database_url =
        env::var("DATABASE_URL_LOCAL").expect("DATABASE_URL_LOCAL should be set.");
    let db_pool = PgPoolOptions::new().connect(&database_url).await.unwrap();
    let db_pool2 = PgPoolOptions::new().connect(&database_url).await.unwrap();

    let local_db_pool = PgPoolOptions::new()
        .connect(&local_database_url)
        .await
        .unwrap();

    let local_db_pool2 = PgPoolOptions::new()
        .connect(&local_database_url)
        .await
        .unwrap();

    let app_state = AppState {
        pool: db_pool,
        local_pool: local_db_pool,
        inventory_addr: "https://127.0.0.1:3001".to_string(),
    };

    // build our application with a route
    let rest = Router::new()
        .route("/", get(health_handler))
        .route("/orders", get(get_all_orders))
        .route("/add_order", post(add_new_order))
        .route("/request_order_token", get(request_new_order_token))
        .with_state(app_state);

    let grpc = get_grpc_router(db_pool2, local_db_pool2);

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

    //TODO 新增一个定时器
}
