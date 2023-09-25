use sqlx::PgPool;

use crate::{
    db_access::db::{de_inventory_from_db},
    models::inventory::{DeducteInventoryRequest},
};

use self::proto::inventory_service_server::{InventoryService, InventoryServiceServer};

mod proto {
    tonic::include_proto!("inventory");
}

pub struct GrpcServiceImpl {
    pool: PgPool,
}

impl GrpcServiceImpl {
    pub fn new(pg_pool: PgPool) -> GrpcServiceImpl {
        return GrpcServiceImpl { pool: pg_pool };
    }
}

#[tonic::async_trait]
impl InventoryService for GrpcServiceImpl {
    async fn deduction_inventory(
        &self,
        request: tonic::Request<proto::DeductionInventoryRequest>,
    ) -> Result<tonic::Response<proto::DeductionInventoryRespone>, tonic::Status> {
        println!("GrpcServiceImpl deduction_inventory call.");

        
        let request_data = request.into_inner();
        let request_data_inner = DeducteInventoryRequest {
            inventory_id: request_data.inventory_id.into(),
            count: request_data.deduction_count.into(),
            order_id: request_data.orders_id.into(),
            description: Some("from grpc.".to_string()),
        };

        let _db_result = de_inventory_from_db(&self.pool, request_data_inner).await;


        let response = proto::DeductionInventoryRespone {
            result : 200,
        };

        println!("GrpcServiceImpl get_orders result: {:?}", response);
        Ok(tonic::Response::new(response))
    }

}

pub fn get_grpc_router(pg_pool: PgPool) -> InventoryServiceServer<GrpcServiceImpl> {
    InventoryServiceServer::new(GrpcServiceImpl::new(pg_pool))
}
