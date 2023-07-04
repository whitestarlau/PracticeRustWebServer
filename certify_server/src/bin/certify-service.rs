use std::env;

#[macro_use]
extern crate lazy_static;

use axum::{Router, routing::{get, post}};
use sqlx::postgres::PgPoolOptions;
use dotenv::dotenv;

use crate::{models::state::AppState, handlers::rest::{health_handler, sign_up, sign_in, verify}};

#[path = "../models/mod.rs"]
mod models;

#[path = "../handlers/mod.rs"]
mod handlers;

#[path = "../utils/mod.rs"]
mod utils;

#[path = "../config/mod.rs"]
mod config;

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

    let app_state = AppState {
        pool: db_pool,
    };

    let health_check_path = "/health_check";

    // build our application with a route
    let rest = Router::new()
        .route(health_check_path, get(health_handler))
        .route("/sign_up", post(sign_up))
        .route("/sign_in", post(sign_in))
        .route("/verify", post(verify).get(verify))
        .with_state(app_state);

    // run it
    let addr = "127.0.0.1:3003";
    println!("listening on {}", addr);

    //向consul中心注册自己
    // tokio::spawn(register_consul(&addr, health_check_path));

    axum::Server::bind(&addr.parse().unwrap())
        .serve(rest.into_make_service())
        .await
        .unwrap();
    
}
