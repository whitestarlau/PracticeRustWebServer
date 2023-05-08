#[macro_use]
extern crate num_derive;

use axum::{
    routing::{get, post},
    Router,
};
use chrono::Utc;
use dotenv::dotenv;
use idgenerator::{IdGeneratorOptions, IdInstance};
use std::{env, sync::Arc};
use std::{net::SocketAddr, thread};
use tokio_cron_scheduler::{Job, JobScheduler};

use sqlx::postgres::PgPoolOptions;

use crate::{
    handlers::grpc::*,
    handlers::{corn::poll_inventory_state_order_from_db, rest::*},
    models::state::AppState,
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

fn main() {
    thread::spawn(|| {
        //定时任务，用于定时轮询本地消息列表中有没有失败的任务没有处理
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            println!("start corn sched in main.");
            corn_aysnc().await;
        });
    });

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        println!("start web_server in main.");
        web_server().await;
    });
}

async fn web_server() {
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
}

async fn corn_aysnc() {
    let mut sched = JobScheduler::new().await.unwrap();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let local_database_url =
        env::var("DATABASE_URL_LOCAL").expect("DATABASE_URL_LOCAL should be set.");

    let db_pool_arc = Arc::new(PgPoolOptions::new().connect(&database_url).await.unwrap());
    let local_db_pool_arc = Arc::new(
        PgPoolOptions::new()
            .connect(&local_database_url)
            .await
            .unwrap(),
    );

    let job = Job::new("1/10 * * * * *", move |uuid, l| {
        let now = Utc::now().timestamp_millis();

        println!("I run every 10 seconds ts:{}", now);
        //TODO 进行定时任务

        let db_pool = db_pool_arc.clone();
        let local_db_pool = local_db_pool_arc.clone();
        poll_inventory_state_order_from_db(
            &db_pool,
            &local_db_pool,
            "https://127.0.0.1:3001".to_string(),
        );
    })
    .unwrap();

    //定时任务必须要启动成功，不能失败，所以直接用unwarp
    sched.add(job).await.unwrap();
    sched.start().await.unwrap();

    println!("start corn sched.");

    loop {
        //一直等待，这里的定时任务要永久执行下去。
        if let Ok(Some(it)) = sched.time_till_next_job().await {
            println!("time_till_next_job {:?}", it);
            tokio::time::sleep(it).await;
        };
    }
}
