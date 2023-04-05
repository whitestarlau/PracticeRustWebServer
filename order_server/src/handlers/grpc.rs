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
        let request_data = request.into_inner();
        let db = get_all_orders_from_db(
            &self.pool,
            request_data.user_id,
            request_data.page,
            request_data.page_size,
        )
        .await;

        let mut responseDatas: Vec<order_proto::Order> = Vec::new();
        if let Ok(datas) = db {
            for order in datas {
                let item_id_str = serde_json::to_string(&order.items_id).unwrap_or_default();
                let price_str = serde_json::to_string(&order.price).unwrap_or_default();
                let des = order.description.unwrap_or_default();
                let proto_order = order_proto::Order {
                    user_id: order.user_id,
                    items_id: item_id_str,
                    price: price_str,
                    total_price: order.total_price,
                    currency: order.currency,
                    description: des,
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
            user_id: request_data.user_id,
            items_id: Vec::new(),
            price: Vec::new(),
            total_price: request_data.total_price,
            currency: request_data.currency,
            description: Option::Some(request_data.description),
            token: request_data.token,
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
