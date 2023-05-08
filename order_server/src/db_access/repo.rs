use tonic::transport::Channel;

use self::inventory_proto::{
    inventory_service_client::InventoryServiceClient, DeductionInventoryRequest,
};

mod inventory_proto {
    tonic::include_proto!("inventory");
}

/**
 * 扣减库存call
 */
pub async fn deduction_inventory_call(
    addr: String,
    inventory_id: i32,
    deduction_count: i32,
    order_id: i32,
) -> Result<inventory_proto::DeductionInventoryRespone, String> {
    let mut client = InventoryServiceClient::connect(addr)
        .await
        .map_err(|err| err.to_string())?;

    let req = tonic::Request::new(DeductionInventoryRequest {
        inventory_id: inventory_id,
        deduction_count: deduction_count,
        orders_id: order_id,
    });

    let deduction_inventory = client
        .deduction_inventory(req)
        .await
        .map_err(|err| err.to_string())?
        .into_inner();

    return Ok(deduction_inventory);
}
