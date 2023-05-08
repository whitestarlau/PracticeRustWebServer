use sqlx::{PgPool};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub local_pool: PgPool,
    pub inventory_addr: String,
}


#[derive(FromPrimitive)]
pub enum InventoryState {
    DOING = 0,
    SUCCESS = 1,
    FAIL = 2,
}

/**
 * 库存扣减结果
 */
#[derive(FromPrimitive)]
pub enum InventoryResult {
    SUCCESS = 200,
}