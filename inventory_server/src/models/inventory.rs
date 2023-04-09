use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Inventory {
    pub id: i32,

    pub count: i32,

    pub description: Option<String>,
}

/**
 * 库存增减的表，记录是谁操作的
 */
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InventoryChange {
    pub id: i32,

    pub count: i32,
    pub inventory_id: i32,
    pub deduction_order_id: Option<i32>,

    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeducteInventoryRequest {
    pub inventory_id : i32,
    
    pub count: i32,
    pub order_id: i32,

    pub description: Option<String>,
}

/**
 * TODO 可能需要添加一些约束，比如权限校验？
 */
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddInventoryRequest {
    pub inventory_id : i32,
    pub count: i32,

    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChangeInventoryResult {
    pub result: i32,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct QueryRequest {
    pub id: i32,
}