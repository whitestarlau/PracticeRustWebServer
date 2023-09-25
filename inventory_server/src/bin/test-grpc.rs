// use crate::order_proto::{order_service_client::OrderServiceClient, GetOrderRequest};

use proto::{DeductionInventoryRequest, DeductionInventoryRespone};

use crate::proto::inventory_service_client::InventoryServiceClient;

mod proto {
    tonic::include_proto!("inventory");
}

#[tokio::main]
async fn main() {
    println!("test grpc");

    let addr = "http://127.0.0.1:3001".to_string();

    let result = deduction_inventory_call(addr, 0, 1, 1).await;
    println!("grpc deduction_inventory_call result: {:?}", result);
}

pub async fn deduction_inventory_call(
    addr: String,
    inventory_id: i32,
    deduction_count: i32,
    order_id: i32,
) -> Result<DeductionInventoryRespone, String> {
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
