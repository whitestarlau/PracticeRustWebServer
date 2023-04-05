use serde::Deserialize;

use crate::order_proto::{order_service_client::OrderServiceClient, GetOrderRequest};


mod order_proto {
    tonic::include_proto!("order");
}


#[tokio::main]
async fn main() {
    println!("test grpc");
   let get_order_result =  grpc_get_order(1.to_string()).await;
   eprintln!("grpc_get_order result: {:?}", get_order_result);
}

async fn grpc_get_order(user_id: String) -> Result<String, String> {
    let addr = "http://127.0.0.1:3000";
    eprintln!("grpc_get_order on : {}", addr);
    
    
    let mut client = OrderServiceClient::connect(addr)
        .await
        .map_err(|err| err.to_string())?;

    eprintln!("grpc_get_order client success.");

    
    let req = tonic::Request::new(GetOrderRequest {
        user_id: user_id,
    });
    let getOrderRespone = client
        .get_orders(req)
        .await
        .map_err(|err| err.to_string())?
        .into_inner();

    eprintln!("grpc_get_order result: {:?}", getOrderRespone);
    Ok("".to_string())
}
