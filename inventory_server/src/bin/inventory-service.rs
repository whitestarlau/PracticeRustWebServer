use axum::{routing::get, Router};
use dotenv::dotenv;
use std::env;
use std::net::SocketAddr;

use sqlx::postgres::PgPoolOptions;

use crate::{
    handlers::grpc::*, handlers::rest::*,
    multiplexservice::MultiplexService,
};

use log;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

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

    // tracing_subscriber::registry()
    //     .with(fmt::layer())
    //     .init();
    // tracing::info!("Hello from tracing");

    //set database pool
    //设置数据库连接池。restful和grpc服务各用一个，不用考虑生命周期标注会比较简单。
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let db_pool = PgPoolOptions::new().connect(&database_url).await.unwrap();
    let db_pool2 = PgPoolOptions::new().connect(&database_url).await.unwrap();

    let health_check_path = "/health_check";

    // build our application with a route
    let rest = Router::new()
        .route(health_check_path, get(health_handler))
        .route("/query_inventory", get(query_inventory))
        .route(
            "/query_inventory_change",
            get(query_inventory_change_history),
        )
        .with_state(db_pool);

    let grpc = get_grpc_router(db_pool2);

    // combine them into one service
    // 将rest和grpc两种路由合并到一起
    let service = MultiplexService::new(rest, grpc);

    // run it
    let addr = "127.0.0.1:3001";
    // let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("listening on {}", addr);

    //向consul中心注册自己
    tokio::spawn(register_consul(addr, health_check_path));

    axum::Server::bind(&addr.parse().unwrap())
        // .serve(rest.into_make_service())
        .serve(tower::make::Shared::new(service))
        .await
        .unwrap();
}

/**
 * 注册到consul中，并实现http健康检查
 */
async fn register_consul(addr: &str, health_check_path: &str) {
    println!("register consul doing...");
    let addrs: Vec<&str> = addr.split(":").collect();
    let addr = addrs[0];
    let port: i32 = addrs[1].parse().unwrap();
    let opt = consul_reg_lib::model::ConsulOption::default();
    let cs = consul_reg_lib::consul::Consul::new(opt).unwrap();

    let health_check_url = format!("http://{}:{}{}", addr, port, health_check_path);

    let health_check = consul_reg_lib::model::HealthCheck::new(health_check_url.to_string());

    println!("register consul health_check params:{:?}", health_check);

    //register consul name as inventory-srv.
    let reg = consul_reg_lib::model::Registration::simple_with_health_check(
        "inventory-srv",
        addr,
        port,
        health_check,
    );

    cs.register(&reg).await.unwrap();
    println!("register consul done.");
}
