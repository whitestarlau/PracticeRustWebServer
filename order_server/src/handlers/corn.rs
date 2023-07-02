use chrono::NaiveDateTime;
use sqlx::PgPool;
use tracing::instrument;

use crate::{
    db_access::{repo::deduction_inventory_call, db::deduction_inventory},
    models::{
        error::internal_error,
        order::{Order, OrderDeInventoryMsg},
        state::{InventoryResult, InventoryState},
    },
};

pub async fn poll_inventory_state_order_from_db(
    pool: &PgPool,
    local_pool: &PgPool,
    inventory_addr: String,
) {
    let orders_msg: Result<Vec<OrderDeInventoryMsg>, _> =
        sqlx::query!("SELECT * FROM orders_de_inventory_msg",)
            .map({
                |row| OrderDeInventoryMsg {
                    id: row.id,
                    user_id: row.user_id,
                    order_id: row.order_id,
                }
            })
            .fetch_all(local_pool)
            .await
            .map_err(internal_error);


    match orders_msg {
        Ok(msg_list) => for msg in msg_list {
            try_de_inventory(pool, local_pool, inventory_addr.clone(), msg);
        },
        Err(e) => {
            //print error msg;
        }
    }
}

async fn try_de_inventory(
    pool: &PgPool,
    local_pool: &PgPool,
    inventory_addr: String,
    msg: OrderDeInventoryMsg,
) {
    let orders = sqlx::query!("SELECT * FROM orders WHERE id = $1", msg.order_id,)
        .map({
            |row| Order {
                id: row.id,
                user_id: row.user_id,
                item_id: row.item_id,
                price: row.price,
                count: row.count,
                currency: row.currency.unwrap_or_default(),
                sub_time: NaiveDateTime::from(row.sub_time.unwrap()).timestamp_millis(),
                pay_time: NaiveDateTime::from(row.pay_time.unwrap()).timestamp_millis(),
                description: row.description,
                inventory_state: row.inventory_state,
            }
        })
        .fetch_one(pool)
        .await
        .map_err(internal_error);

    if let Ok(order) = orders {
        deduction_inventory(
            pool,
            local_pool,
            inventory_addr,
            order.item_id,
            order.count,
            msg.order_id,
        );
    }
}