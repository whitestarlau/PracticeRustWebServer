use std::env;

// #[macro_use]
// extern crate lazy_static;

use axum::{Router, routing::{get, post}};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;
use tower_http::cors::CorsLayer;

use crate::{models::state::AppState, handlers::rest::{health_handler, get_goods_summary, get_goods_detail}};

#[path = "../models/mod.rs"]
mod models;

#[path = "../handlers/mod.rs"]
mod handlers;

#[path = "../db_access/mod.rs"]
mod db_access;


fn main() {
    
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        println!("start web_server in main.");
        web_server().await;
    });
}

async fn web_server() {
    dotenv().ok();

    //初始化tracing
    let file_appender = tracing_appender::rolling::hourly("./axum_log", "prefix.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    // tracing_subscriber::registry().with(fmt::layer()).init();
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    //set database pool
    //设置数据库连接池。restful和grpc服务各用一个，不用考虑生命周期标注会比较简单。
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let db_pool = PgPoolOptions::new().connect(&database_url).await.unwrap();

    // let app_state = AppState {
    //     pool: db_pool,
    // };

    let health_check_path = "/health_check";

    // build our application with a route
    let rest = Router::new()
        .route(health_check_path, get(health_handler))
        .route("/goods_list", post(get_goods_summary).get(get_goods_summary))
        .route("/goods_detail", post(get_goods_detail).get(get_goods_detail))
        .layer(CorsLayer::permissive())
        .with_state(db_pool);

    // run it
    let addr = "127.0.0.1:3004";
    println!("listening on {}", addr);

    //向consul中心注册自己
    // tokio::spawn(register_consul(&addr, health_check_path));

    axum::Server::bind(&addr.parse().unwrap())
        .serve(rest.into_make_service())
        .await
        .unwrap();
    
}