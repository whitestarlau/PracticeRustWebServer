use std::f32::consts::E;

use axum::http::StatusCode;
use chrono::NaiveDateTime;
use common_lib::internal_error;
use sqlx::postgres::PgPool;
use tracing::info;

use crate::{
    db_access::repo::deduction_inventory_call,
    models::{
        order::{AddOrder, AddOrderResult, Order},
        state::{InventoryResult, InventoryState},
    },
};

mod inventory_proto {
    tonic::include_proto!("inventory");
}

pub async fn get_all_orders_from_db(
    pool: &PgPool,
    user_id: i64,
    page: i64,
    page_size: i64,
) -> Result<Vec<Order>, (StatusCode, String)> {
    let offset = page_size * page;
    let orders = sqlx::query!(
        "SELECT * FROM orders WHERE user_id = $1 LIMIT $2 OFFSET $3",
        user_id,
        page_size,
        offset
    )
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
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    info!("get_all_orders_from_db size: {}", orders.len());

    Ok(orders)
}

pub async fn add_new_order_from_db(
    pool: &PgPool,
    local_pool: &PgPool,
    inventory_addr: String,
    data: AddOrder,
) -> Result<AddOrderResult, (StatusCode, String)> {
    let des = data.description.unwrap_or_default();
    let price = data.price;

    println!("add_new_order des: {}", des);

    //本地订单插入
    // let item_ids_str = serde_json::to_string(&data.items_id).unwrap_or_default();

    let ts_1970 = NaiveDateTime::from_timestamp_opt(0, 0).unwrap_or_default();

    let insert_result :Result<i32, (StatusCode, String)> = sqlx::query!("INSERT INTO orders (user_id, item_id, price, count, currency, pay_time, description,inventory_state) VALUES ($1, $2, $3, $4, $5, $6, $7,$8) RETURNING id",
        data.user_id,
        data.items_id,
        data.price,data.count, data.currency,
        ts_1970, des,
        InventoryState::DOING as i32
    )
        .map(|row| row.id)
        .fetch_one(pool)
        .await
        .map_err(internal_error);

    match insert_result {
        Ok(order_id ) => {
            let insert_msg = sqlx::query!(
                "INSERT INTO orders_de_inventory_msg (user_id, order_id) VALUES ($1, $2) RETURNING id",
                data.user_id,
                order_id,
            )
            .map(|row| row.id)
            .fetch_one(local_pool)
            .await
            .map_err(internal_error);

            deduction_inventory(
                pool,
                local_pool,
                inventory_addr,
                data.items_id,
                data.count,
                order_id,
            )
            .await;

            let result = AddOrderResult {
                description: "add successed.".to_string(),
            };
            Ok(result)
        }
        Err(e) => Err(e),
    }
}

/**
 * 扣减库存，并更新本地数据库。
 * 没有返回，调用者不关心这个函数的执行情况，因为结果是会放到数据库中，并由定时器定期轮询检查。
 */
pub async fn deduction_inventory(
    pool: &PgPool,
    local_pool: &PgPool,
    inventory_addr: String,
    items_id: i32,
    count: i32,
    order_id: i32,
) {
    //分布式事务，扣减库存
    let deducation_resp = deduction_inventory_call(inventory_addr, items_id, count, order_id).await;
    if let Ok(resp) = deducation_resp {
        //响应为success的时候我们记录扣减库存成功
        let inventory_state = if InventoryResult::SUCCESS as i32 == resp.result {
            InventoryState::SUCCESS
        } else {
            InventoryState::FAIL
        };

        //删除消息数据库
        //注意，这一步可能写成功也可能写失败，所以可能导致deduction_inventory_call反复被调用，库存那边需要保证同一个订单id不会重复扣减。
        let _update_msg_result = sqlx::query!(
            "DELETE FROM orders_de_inventory_msg where  order_id = ($1)",
            order_id
        )
        .fetch_one(local_pool)
        .await
        .map_err(internal_error);

        //标记扣减库存成功或者失败
        let _update_result = sqlx::query!(
            "UPDATE orders SET inventory_state = ($1) where id = ($2)",
            inventory_state as i32,
            order_id
        )
        .fetch_one(pool)
        .await
        .map_err(internal_error);
    } else {
        // 远程调用失败，不代表扣减库存失败，等待定时器轮训的时候继续尝试
        //TODO 添加定时器轮询orders_de_inventory_msg表
    }
}
