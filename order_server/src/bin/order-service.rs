use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use dotenv::dotenv;
use std::env;

use sqlx::postgres::{PgPoolOptions};

use crate::{handlers::general::*};

#[path = "../db_access/mod.rs"]
mod db_access;
#[path = "../handlers/mod.rs"]
mod handlers;

#[path = "../models/mod.rs"]
mod models;

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    //set database pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let db_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .unwrap();
    
    // build our application with a route
    let app = Router::new()
        .route("/", get(health_handler))
        .route("/orders", get(get_all_orders))
        .route("/add_order", post(add_new_order))
        .with_state(db_pool);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

