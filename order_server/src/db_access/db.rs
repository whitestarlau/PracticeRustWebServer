use std::f32::consts::E;

use axum::http::StatusCode;
use chrono::NaiveDateTime;
use common_lib::internal_error;
use sqlx::{postgres::PgPool, Acquire};
use tracing::info;
use uuid::Uuid;

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
    user_id: Uuid,
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
    inventory_addr: String,
    data: AddOrder,
    uuid: Uuid,
) -> Result<AddOrderResult, (StatusCode, String)> {
    let des = data.description.unwrap_or_default();
    let price = data.price;

    println!("add_new_order des: {}", des);

    //本地订单插入
    // let item_ids_str = serde_json::to_string(&data.items_id).unwrap_or_default();

    let ts_1970 = NaiveDateTime::from_timestamp_opt(0, 0).unwrap_or_default();

    let mut conn = pool.acquire().await.unwrap();
    let mut tx = conn.begin().await.map_err(internal_error)?;

    let insert_order =  sqlx::query!("INSERT INTO orders (user_id, item_id, price, count, currency, pay_time, description,inventory_state) VALUES ($1, $2, $3, $4, $5, $6, $7,$8) RETURNING id",uuid,data.items_id, data.price,data.count, data.currency,ts_1970, des,InventoryState::DOING as i32)
                .map(|row| row.id)
                .fetch_one(&mut tx)
                .await;

    let mut order_id_cp = -1;
    let result = match insert_order {
        Ok(order_id) => {
            order_id_cp = order_id;

            println!("insert_order suceess");

            let insert_msg = sqlx::query!(
                "INSERT INTO orders_de_inventory_msg (user_id, order_id) VALUES ($1, $2) RETURNING id",
                uuid,
                order_id,
            )
            .map(|row| row.id)
            .fetch_one(&mut tx)
            .await
            .map_err(internal_error);

            let innerResult = if let Err(e) = insert_msg {
                println!("insert_msg fail should rollback.");

                Err(e)
            } else {
                println!("insert_msg success,try rpc call.");

                
                let addResult = AddOrderResult {
                    description: "add successed.".to_string(),
                };
                Ok(addResult)
            };

            innerResult
        }
        Err(e) => {
            println!("insert_order failed should rollback.");

            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    if let Ok(_) = result {
        tx.commit().await.unwrap();
        deduction_inventory(pool, inventory_addr, data.items_id, data.count, order_id_cp).await;
    } else {
        tx.rollback().await.unwrap();
    }

    return result;
}

/**
 * 扣减库存，并更新本地数据库。
 * 没有返回，调用者不关心这个函数的执行情况，因为结果是会放到数据库中，并由定时器定期轮询检查。
 */
pub async fn deduction_inventory(
    pool: &PgPool,
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
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.map_err(internal_error)?;

        //注意，这一步可能写成功也可能写失败，所以可能导致deduction_inventory_call反复被调用，库存那边需要保证同一个订单id不会重复扣减。
        let _update_msg_result = sqlx::query!(
            "DELETE FROM orders_de_inventory_msg where  order_id = ($1)",
            order_id
        )
        .fetch_one(&mut tx)
        .await
        .map_err(internal_error);

        //标记扣减库存成功或者失败
        let _update_result = sqlx::query!(
            "UPDATE orders SET inventory_state = ($1) where id = ($2)",
            inventory_state as i32,
            order_id
        )
        .fetch_one(&mut tx)
        .await
        .map_err(internal_error);

        if let Ok(_) = _update_msg_result {
            if let Ok(a) = _update_result {
                tx.commit().await.unwrap();
            } else {
                tx.rollback().await.unwrap();
            }
        } else {
            tx.rollback().await.unwrap();
        }
    } else {
        // 远程调用失败，不代表扣减库存失败，等待定时器轮训的时候继续尝试
        //TODO 添加定时器轮询orders_de_inventory_msg表
    }
}
