use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Order {
    pub id: i32,

    pub items_id: Vec<String>,
    pub price: Vec<i32>,
    pub total_price: i32,
    pub currency: String,

    pub sub_time: i64,
    pub pay_time: i64,

    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddOrder {
    pub items_id: Vec<String>,
    pub price: Vec<i32>,
    pub total_price: i32,
    pub currency: String,

    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddOrderResult {
    pub description: String,
}
