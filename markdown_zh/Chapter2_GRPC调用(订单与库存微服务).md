# 库存微服务和GRPC

在上一篇文章中我们通过Axum构建了一个简单的http服务器。以为我们的构建微服务的目标，我们需要多个服务。这里我们还是以电商场景做目标，构建一个简单库存服务，然后通过RPC调用来连接两个服务。

## 1、库存微服务

### 1.1 数据结构和定义
和订单微服务一样，我们单独创建一个微服务，名为```inventory_server```，目录结构如下：

```
.
├── Cargo.lock
├── Cargo.toml
├── db_new.sql
├── readme.md
└── src
    ├── bin
    │   └── inventory-service.rs
    ├── db_access
    │   ├── db.rs
    │   └── mod.rs
    ├── handlers
    │   ├── mod.rs
    │   └── general.rs
    └── models
        ├── error.rs
        ├── inventory.rs
        └── mod.rs
```

因为和上一章的结构基本一样，我们不再一一介绍那些基本一样的代码，只介绍库存的主要逻辑实现。

首先是数据结构，我们的库存服务包括了两张表，一张记录当前的库存状态，一张记录库存的变化情况。表的具体字段如下：

``` sql
create table inventory (
       id serial primary key,
       
       count INT not null,

       description varchar(140)
);
create table inventory_change (
       id serial primary key,
       
       count INT not null,

       inventory_id INT not null,
       
       deduction_order_id INT,

       description varchar(140)
);
```

为了尽量简化，我们的库存只有库存id、库存数量两个字段。而库存扣减表包含了扣减数量、关联库存id、关联订单id、描述这几个关键字段。

因为在这一章中，我们计划库存的扣减只能使用GRPC来调用，所以http部分的服务只实现查询部分api。

首先我们还是要定义一下查询和返回的数据结构，需要和数据库表字段相吻合。这部分定义在models/inventory.rs中。

``` Rust
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct QueryRequest {
    pub id: i32,
}
```

### 1.2 查询逻辑实现

然后我们就需要实现具体的查询逻辑，这部分代码还是在```db_access/db.rs```中进行实现：

``` Rust
pub async fn query_inventory_from_db(
    pool: &PgPool,
    inventoey_id: i32,
) -> Result<Vec<Inventory>, (StatusCode, String)> {
    let inventory: Vec<Inventory> =
        sqlx::query!("SELECT * FROM inventory WHERE id = $1", inventoey_id)
            .map({
                |row| Inventory {
                    id: row.id,
                    count: row.count,
                    description: row.description,
                }
            })
            .fetch_all(pool)
            .await
            .map_err(internal_error)?;

    println!("query_orders_from_db size: {}", inventory.len());

    Ok(inventory)
}

pub async fn query_inventory_change_from_db(
    pool: &PgPool,
    inventoey_id: i32,
) -> Result<Vec<InventoryChange>, (StatusCode, String)> {
    let inventory: Vec<InventoryChange> =
        sqlx::query!("SELECT * FROM inventory_change WHERE id = $1", inventoey_id)
            .map({
                |row| InventoryChange {
                    id: row.id,
                    inventory_id: row.inventory_id,
                    deduction_order_id: row.deduction_order_id,
                    count: row.count,
                    description: row.description,
                }
            })
            .fetch_all(pool)
            .await
            .map_err(internal_error)?;

    println!("query_orders_from_db size: {}", inventory.len());

    Ok(inventory)
}
```
最后我们需要将sql的实现进行一层包装，实现Axum所需的函数，这部分在rest.rs中

``` rust
pub async fn query_inventory(
    State(pool): State<PgPool>,
    Query(query_params): Query<QueryRequest>,
) -> Result<axum::Json<Inventory>, (StatusCode, String)> {
    // println!("get_all_orders user_id: {}", query_params.user_id);
    let db_result = query_inventory_from_db(&pool, query_params.id).await;

    match db_result {
        Ok(mut result_vec) => {
            return result_vec
                .pop()
                .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "err".to_string()))
                .map(map_ok_result);
        }
        Err(err) => {
            return Err(err);
        }
    }
}

/**
 * 查询库存改变的记录
 */
pub async fn query_inventory_change_history(
    State(pool): State<PgPool>,
    Query(query_params): Query<QueryRequest>,
) -> Result<axum::Json<Vec<InventoryChange>>, (StatusCode, String)> {
    // println!("get_all_orders user_id: {}", query_params.user_id);
    let result = query_inventory_change_from_db(&pool, query_params.id)
        .await
        .map(map_ok_result);

    return result;
}
```

你可能注意到了，查询库存的时候我们的数据库查询返回的是```Vec```，但是根据业务逻辑来说，库存应该是唯一的，所以在最后返回的时候我们使用了pop来获取这个唯一的元素。

## 2、集成GRPC

### 2.1 GRPC引入和结构定义

因为我们的服务是多个微服务，相互之间运行在不同的进程乃至不同的服务器、网络节点上。要让不同的微服务之间实现沟通，通常使用的是远程过程调用（Remote Procedure Call）。RPC有很多种实现方案，在这里我们选用GRPC，首先是因为在Rust生态中比较流行的是GRPC，有很完善的开源组件，其次是因为GRPC基于http协议，所以可以和其他语言实现的跨语言的沟通。

在Rust中GRPC流行的库名为tonic，我们进行引入：
``` toml
[dependencies]
# ...
# grpc
tonic = "0.8"
# 序列化反序列化proto使用的库
prost = "0.11"
hyper = "0.14.19"
futures = "0.3"

[build-dependencies]
tonic-build = "0.8"
```

除了引入直接的库依赖外，我们还需要引入一个```tonic-build```的库。

为了方便在不同的微服务之间共享proto，我们将proto定义放在父文件夹的子文件夹中。即文件夹的结构如下：
```
.
├── inventory_server
├── order_server
└── proto
    └── inventory.proto
```

inventory.proto文件的定义如下：

``` proto
syntax = "proto3";
package inventory;

message DeductionInventoryRequest {
  int32 inventoryId = 1;
  int32 deductionCount = 2;
  int32 ordersId = 3;
}

message DeductionInventoryRespone {
  int32 result = 1; 
}

service InventoryService {
  rpc deductionInventory(DeductionInventoryRequest) returns (DeductionInventoryRespone);
}
```

我们暂时只实现扣减库存的接口，定义了一个请求，一个应答。

### 2.2 GRPC编译配置及生成的代码

在定义proto文件后，我们需要在我们的项目中新增一个```build.rs```文件，用于在编译前执行GRPC微服务代码的生成。
``` Rust
use std::fs;

/**
 * 生成gRPC的文件
 */
fn main() {
    let proto_path = "../proto";
    let mut proto_files = vec![];
    for entry in fs::read_dir(proto_path).unwrap() {
        let entry = entry.unwrap();
        let md = entry.metadata().unwrap();
        if md.is_file() && entry.path().extension().unwrap() == "proto" {
            proto_files.push(entry.path().as_os_str().to_os_string())
        }
    }

    tonic_build::configure()
        // .out_dir("src") // 生成代码的存放目录，可以指定src文件夹来看tonic生成的代码是怎么样的
        .compile(
            proto_files.as_slice(), // 欲生成的 proto 文件列表
            &[proto_path],          // proto 依赖所在的根目录
        )
        .unwrap();
}
```

如果不指定生成代码路径，生成的代码将默认生成到target文件夹中。比如这里将生成一个200多行的inventory.rs文件。

我将一部分代码摘抄在这里，可以看到Rust生成了出入参的结构体，然后分别生成了一个客户端代码和一个服务端代码，并且内部是通过http进行通讯的。

``` Rust
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeductionInventoryRequest {
    #[prost(int32, tag = "1")]
    pub inventory_id: i32,
    #[prost(int32, tag = "2")]
    pub deduction_count: i32,
    #[prost(int32, tag = "3")]
    pub orders_id: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeductionInventoryRespone {
    #[prost(int32, tag = "1")]
    pub result: i32,
}
/// Generated client implementations.
pub mod inventory_service_client {
    //...
                                http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
    //...
}

pub mod inventory_service_server {
    //...
}
```

### 2.3 代码业务实现

在库存这边，我们先要实现服务端的代码。首先我们需要声明一个结构体，然后为其实现InventoryService这个特性：

``` rust
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
```

 然后我们将这个实现注入到InventoryServiceServer中，实例化的InventoryServiceServer将在未来交给Axum：

``` Rust
pub fn get_grpc_router(pg_pool: PgPool) -> InventoryServiceServer<GrpcServiceImpl> {
    InventoryServiceServer::new(GrpcServiceImpl::new(pg_pool))
}

```

### 2.4 让GRPC服务和restful服务共存

然后，然后我们就会遇到一个问题，如何将一个GRPC的服务扔给Axum，让其执行并和restful的http服务共存呢？

我们从Axum官方的例子中找到了答案。我们需要实现了一个tower中间件的Service，就叫MultiplexService吧，然后，通过识别请求头来将不同的请求分发给restful或者grpc服务。关键代码如下：

``` Rust
/**
 * 拷贝自axum官方 rest-grpc-multiplex 示例
 */

use axum::{body::BoxBody, http::header::CONTENT_TYPE, response::IntoResponse};
use futures::{future::BoxFuture, ready};
use hyper::{Body, Request, Response};
use std::{
    convert::Infallible,
    task::{Context, Poll},
};
use tower::Service;

impl<A, B> Service<Request<Body>> for MultiplexService<A, B>
where
    A: Service<Request<Body>, Error = Infallible>,
    A::Response: IntoResponse,
    A::Future: Send + 'static,
    B: Service<Request<Body>, Error = Infallible>,
    B::Response: IntoResponse,
    B::Future: Send + 'static,
{
    type Response = Response<BoxBody>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    //...

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        //...
        // if we get a grpc request call the grpc service, otherwise call the rest service
        // when calling a service it becomes not-ready so we have drive readiness again
        if is_grpc_request(&req) {
            self.grpc_ready = false;
            let future = self.grpc.call(req);
            Box::pin(async move {
                let res = future.await?;
                Ok(res.into_response())
            })
        } else {
            self.rest_ready = false;
            let future = self.rest.call(req);
            Box::pin(async move {
                let res = future.await?;
                Ok(res.into_response())
            })
        }
    }
}

fn is_grpc_request<B>(req: &Request<B>) -> bool {
    req.headers()
        .get(CONTENT_TYPE)
        .map(|content_type| content_type.as_bytes())
        .filter(|content_type| content_type.starts_with(b"application/grpc"))
        .is_some()
}

```

通过这个Service我们就可以将两个不同类型的服务组合到一起。请注意，这里我们新建了一个单独的数据库连接池给GRPC使用，这是因为在Rust的所有权系统中共享数据库连接池对象会相当麻烦，我们干脆建立两个对象，这样会更加简单和方便。

``` Rust
#[tokio::main]
async fn main() {
    dotenv().ok();

    //set database pool
    //设置数据库连接池。restful和grpc服务各用一个，不用考虑生命周期标注会比较简单。
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let db_pool = PgPoolOptions::new().connect(&database_url).await.unwrap();
    let db_pool2 = PgPoolOptions::new().connect(&database_url).await.unwrap();

    // build our application with a route
    let rest = Router::new()
        .route("/", get(health_handler))
        .route("/query_inventory", get(query_inventory))
        .route("/query_inventory_change", get(query_inventory_change_history))
        .with_state(db_pool);

    let grpc = get_grpc_router(db_pool2);

    // combine them into one service 
    // 将rest和grpc两种路由合并到一起
    let service = MultiplexService::new(rest, grpc);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        // .serve(rest.into_make_service())
        .serve(tower::make::Shared::new(service))
        .await
        .unwrap();
}
```

这样我们就实现了一个简单的restful+grpc的微服务了。

当然，我们还需要测试一下我们的grpc服务是否能够正常运行。简单的测试代码如下。测试代码位于```/bin/test-grpc.rs```下

``` Rust
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

```

然后我们先使用```cargo run```运行库存微服务，然后通过```cargo run --bin test-grpc```来执行我们的测试代码，可以得到如下结果，证明我们的GRPC已经可以服务了：

```
test grpc
grpc deduction_inventory_call result: Ok(DeductionInventoryRespone { result: 200 })
```

## 3、总结

在本章中，我们搭建了一个简单的库存微服务，然后引入了GRPC并为库存微服务实践了GRPC的服务端。在下一章中我们将在订单微服务中通过GRPC调用库存的微服务，来实现订单生成时的库存校验和自动扣减功能。同时我们将通过一个简单的“分布式事务”来保证两个微服务的数据一致性。