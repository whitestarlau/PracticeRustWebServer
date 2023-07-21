use std::f32::consts::E;

use axum::http::StatusCode;
use common_lib::internal_error;
use sqlx::postgres::PgPool;

use crate::models::{
    inventory::{
        AddInventoryRequest, ChangeInventoryResult, DeducteInventoryRequest, Inventory,
        InventoryChange,
    },
};

pub async fn query_inventory_from_db(
    pool: &PgPool,
    inventoey_id: i32,
) -> Result<Vec<Inventory>, (StatusCode, String)> {
    let inventory: Vec<Inventory> =
        sqlx::query!("SELECT * FROM inventory WHERE id = $1", inventoey_id)
            .map({
                |row| Inventory {
                    id: row.id,
                    count: row.count,
                    description: row.description,
                }
            })
            .fetch_all(pool)
            .await
            .map_err(internal_error)?;

    println!("query_orders_from_db size: {}", inventory.len());

    Ok(inventory)
}

pub async fn query_inventory_change_from_db(
    pool: &PgPool,
    inventoey_id: i32,
) -> Result<Vec<InventoryChange>, (StatusCode, String)> {
    let inventory: Vec<InventoryChange> =
        sqlx::query!("SELECT * FROM inventory_change WHERE id = $1", inventoey_id)
            .map({
                |row| InventoryChange {
                    id: row.id,
                    inventory_id: row.inventory_id,
                    deduction_order_id: row.deduction_order_id,
                    count: row.count,
                    description: row.description,
                }
            })
            .fetch_all(pool)
            .await
            .map_err(internal_error)?;

    println!("query_orders_from_db size: {}", inventory.len());

    Ok(inventory)
}

/**
 * 添加库存
 */
pub async fn add_inventory_from_db(
    pool: &PgPool,
    data: AddInventoryRequest,
) -> Result<ChangeInventoryResult, (StatusCode, String)> {
    let des = data.description.unwrap_or_default();
    let count = data.count;
    let inventory_id = data.inventory_id;
    if count > 0 {
        //增加库存，目前只是简单增加，不需要额外校验

        let mut tx = pool.begin().await.unwrap();

        let update_result = sqlx::query!(
            "UPDATE inventory SET count = count + ($1) where id = ($2)",
            count,
            inventory_id
        )
        .execute(&mut tx)
        .await;

        let _insert_result = sqlx::query!(
            "insert into inventory_change (count ,inventory_id, description) values ($1,$2, $3)",
            count,
            inventory_id,
            des
        )
        .execute(&mut tx)
        .await;

        if let Err(e) = update_result {
            //插入失败了，可能没有这个库存id
            let _ = tx.rollback().await;
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("update failed,err: {:?}", e).to_string(),
            ));
        }

        let _ = tx.commit().await;

        let _rows = sqlx::query!(
            "UPDATE inventory SET count = count + ($1) where id = ($2)",
            count,
            inventory_id
        )
        .fetch_one(pool)
        .await
        .map_err(internal_error);

        return Ok(ChangeInventoryResult {
            result: 200,
            description: Some("sucess.".to_string()),
        });
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "count must > 0.".to_string(),
        ));
    }
}

/**
 * 扣减库存
 *
 * TODO 如果库存同一个订单的库存已经扣减过了，我们需要直接返回成功
 */
pub async fn de_inventory_from_db(
    pool: &PgPool,
    data: DeducteInventoryRequest,
) -> Result<ChangeInventoryResult, (StatusCode, String)> {
    let des = data.description.unwrap_or_default();
    //传进来是正的，我们扣减用负的
    let count = 0 - data.count;
    let inventory_id = data.inventory_id;
    let order_id = data.order_id;

    if count < 0 {
        //扣减库存
        let inventory_changed: Vec<InventoryChange> =
        sqlx::query!("SELECT * FROM inventory_change WHERE deduction_order_id = $1", order_id)
            .map({
                |row| InventoryChange {
                    id: row.id,
                    inventory_id: row.inventory_id,
                    deduction_order_id: row.deduction_order_id,
                    count: row.count,
                    description: row.description,
                }
            })
            .fetch_all(pool)
            .await
            .map_err(internal_error)?;

        if inventory_changed.len() >= 0 {
            //之前已经扣减过库存了，直接返回成功，避免重入。
            return Ok(ChangeInventoryResult {
                result: 200,
                description: Some("deduction already done,sucess.".to_string()),
            });
        }
       

        let mut tx = pool.begin().await.unwrap();

        let update_result = sqlx::query!(
            "UPDATE inventory SET count = count + ($1) where id = ($2)",
            count,
            inventory_id
        )
        .execute(&mut tx)
        .await;

        let _insert_result = sqlx::query!(
            "insert into inventory_change (count ,inventory_id,deduction_order_id, description) values ($1,$2, $3,$4)",
            count,
            inventory_id,
            order_id,
            des
        )
        .execute(&mut tx)
        .await;

        if let Err(e) = update_result {
            //插入失败了，可能没有这个库存id
            let _ = tx.rollback().await;
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("update failed,err: {:?}", e).to_string(),
            ));
        }

        let _ = tx.commit().await;

        let _rows = sqlx::query!(
            "UPDATE inventory SET count = count + ($1) where id = ($2)",
            count,
            inventory_id
        )
        .fetch_one(pool)
        .await
        .map_err(internal_error);

        return Ok(ChangeInventoryResult {
            result: 200,
            description: Some("sucess.".to_string()),
        });
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "count must > 0.".to_string(),
        ));
    }
}
