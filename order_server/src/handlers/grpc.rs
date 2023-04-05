use sqlx::PgPool;

use crate::{
    db_access::db::{add_new_order_from_db, get_all_orders_from_db},
    models::order::AddOrder,
};

use self::order_proto::order_service_server::{OrderService, OrderServiceServer};


mod order_proto {
    tonic::include_proto!("order");
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
impl OrderService for GrpcServiceImpl {
    async fn get_orders(
        &self,
        request: tonic::Request<order_proto::GetOrderRequest>,
    ) -> Result<tonic::Response<order_proto::GetOrderRespone>, tonic::Status> {
        let db = get_all_orders_from_db(&self.pool).await;

        let mut responseDatas: Vec<order_proto::Order> = Vec::new();
        if let Ok(data_json) = db {
            let data = data_json.0;
            for order in data {
                let proto_order = order_proto::Order {
                    user_id: 1.to_string(),
                    items_id: "".to_string(),
                    price: "".to_string(),
                    total_price: 100,
                    currency: "CNY".to_string(),
                    description: "".to_string(),
                };
                responseDatas.push(proto_order);
            }
        }

        let response = order_proto::GetOrderRespone {
            orders: responseDatas,
        };

        println!("GrpcServiceImpl get_orders result: {:?}", response);
        Ok(tonic::Response::new(response))
    }

    async fn add_order(
        &self,
        request: tonic::Request<order_proto::AddOrderRequest>,
    ) -> Result<tonic::Response<order_proto::AddOrderRespone>, tonic::Status> {
        let request_data = request.into_inner();

        let add = AddOrder {
            items_id: Vec::new(),
            price: Vec::new(),
            total_price: request_data.total_price,
            currency: request_data.currency,
            description: Option::Some(request_data.description),
        };
        let db_result = add_new_order_from_db(&self.pool, add).await;

        let response = match db_result {
            Ok(_) => order_proto::AddOrderRespone { result: 0 },
            Err(_) => order_proto::AddOrderRespone { result: 1 },
        };

        Ok(tonic::Response::new(response))
    }
}

pub fn get_grpc_router(pgPool: PgPool) -> OrderServiceServer<GrpcServiceImpl> {
    OrderServiceServer::new(GrpcServiceImpl::new(pgPool))
}