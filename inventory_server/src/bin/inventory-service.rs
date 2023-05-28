use axum::{
    routing::get,
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
    consul_api::consul::*,
    consul_api::model::*,
};


#[path = "../db_access/mod.rs"]
mod db_access;
#[path = "../handlers/mod.rs"]
mod handlers;

#[path = "../models/mod.rs"]
mod models;

#[path = "../multiplex_service.rs"]
mod multiplexservice;


#[path = "../consul_api/mod.rs"]
mod consul_api;

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
        .route("/query_inventory", get(query_inventory))
        .route("/query_inventory_change", get(query_inventory_change_history))
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
    tokio::spawn(register(&addr));

    axum::Server::bind(&addr.parse().unwrap())
        // .serve(rest.into_make_service())
        .serve(tower::make::Shared::new(service))
        .await
        .unwrap();
}


/**
 * TODO 实现consul的健康检查服务
 */
async fn register(addr: &str) {
    println!("register consul doing...");
    let addrs: Vec<&str> = addr.split(":").collect();
    let addr = addrs[0];
    let port: i32 = addrs[1].parse().unwrap();
    let opt = consul_api::model::ConsulOption::default();
    let cs = consul_api::consul::Consul::new(opt).unwrap();

    //register consul name as inventory-srv.
    let reg = consul_api::model::Registration::simple("inventory-srv", addr, port);
    cs.register(&reg).await.unwrap();
    println!("register consul done.");
}