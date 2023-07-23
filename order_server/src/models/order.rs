use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct GetOrderParams {
    pub user_id: Uuid,
    pub page: i64,
    pub page_size: i64,
}

/**
 * inventory_success 库存是否扣减成功
 */
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Order {
    pub id: i32,

    pub user_id: Uuid,

    pub item_id: i32,
    pub price: i32,
    pub count : i32,
    pub currency: String,

    pub sub_time: i64,
    pub pay_time: i64,

    pub inventory_state: i32,

    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddOrder {
    pub items_id: i32,
    pub price: i32,
    pub count : i32,
    pub currency: String,

    pub description: Option<String>,

    pub token: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddOrderResult {
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewOrderToken {
    pub token: i64,
}



#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OrderDeInventoryMsg {
    pub id: i32,
    pub user_id: Uuid,

    pub order_id: i32,
}
