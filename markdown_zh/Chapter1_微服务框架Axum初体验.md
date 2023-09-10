# 微服务框架Axum初体验

Rust是近年来很火热的一个编程语言，在很多领域都出现了很优秀的解决案例。笔者自己是一位客户端程序员，对于后端开发一直比较感兴趣，出于个人探索的目的决定以Rust中的axum框架来写出一个实验性的微服务。

Rust领域有几个流行的web框架，我了解到的比较著名的有如下三个：
- Rocket框架。一个很容易上手的框架，使用宏来声明和匹配路由。
- Actix Web。一个很著名的以性能著称的框架，在社区人气很高。中间出现过原作者因个人原因放弃维护换手的事件。
- Axum框架。由rust社区的异步事实标准tokio团队开发的web框架，性能也很高，人气也在迅速攀升中。

Axum框架出自于大名鼎鼎的tokio团队，可以说发展非常迅速，而一个知名团队出品，受开发者个人因素影响的概率比较小，所以这次我选择使用Axum框架进行开发。

本文涉及的项目github地址如下： [PracticeRustWebServer](https://github.com/whitestarlau/PracticeRustWebServer)。

***笔者在后端开发和Rust开发领域也是初学者，此项目纯属实验和练手性质，肯定会存在很多不完善不合理的地方。请读者谨慎甄别。***

## 1.新建项目
我们使用Cargo新建一个项目，然后进行如下配置：

``` Rust
[package]
name = "order_server"
version = "0.1.0"
edition = "2021"
default-run = "order-service"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
axum = "0.6.10"
tokio = { version = "1.0", features = ["full"] }
tower = { version = "0.4", features = ["util"] }
dotenv = "0.15.0"
sqlx = { version = "0.6.2", features = [
    "postgres",
    "runtime-tokio-rustls",
    "macros",
    "chrono",
] }
serde = { version = "1.0.134", features = ["derive"] }
serde_json = "1.0"

chrono = { version = "0.4.19", features = ["serde"] }

[[bin]]
name = "order-service"
```

有如下的点需要说明：
1. 我们配置了bin，然后使用default-run的方式配置默认的运行路径。
2. 引入了axum、tokio、tower的依赖。这三个项目都是tokio项目组出品。
3. 引入dotenv，用于在测试环境中减少配置环境变量的麻烦。
4. 使用sqlx来进行数据库操作。
5. serde用于json的序列化和反序列化。
6. chrono库用于处理时间相关的需求。比如时间戳到时间对象的转换。

源代码的结构如下：
```
.
├── Cargo.lock
├── Cargo.toml
└── src
    ├── bin
    │   └── order-service.rs
    ├── db_access
    │   ├── db.rs
    │   └── mod.rs
    ├── handlers
    │   ├── general.rs
    │   └── mod.rs
    └── models
        ├── error.rs
        ├── mod.rs
        └── order.rs
```

执行入口在```bin/order-service.rs```中。其他源码通过每个文件夹下的mod.rs对外进行暴露，在```order-service.rs```中进行引入。如下：
``` Rust
#[path = "../db_access/mod.rs"]
mod db_access;
#[path = "../handlers/mod.rs"]
mod handlers;
#[path = "../models/mod.rs"]
mod models;
```
各个模块的实现我们后面慢慢介绍。

# 1.2 入口
在```order-service.rs```中，我们需要实现一个main函数。这个函数我们添加了```#[tokio::main]```的宏。这个宏可以将我们的函数自动展开转换为tokio的异步函数。

``` Rust
#[tokio::main]
async fn main() {
    dotenv().ok();
    
    //set database pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let db_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .unwrap();
    
    // build our application with a route
    let app = Router::new()
        .route("/", get(health_handler))
        .with_state(db_pool);

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

```

首先我们调用了```dotenv```的函数，这个函数可以在debug编译模式下将```.env```中的设置自动设置到环境变量中。然后我们使用环境变量```DATABASE_URL```来连接我们的数据库。获取到的```db_pool```对象会被设置到路由中，后续我们将看到如何使用这个数据库。

继续我们的代码。我们使用```Router::new()```新建了一个路由，然后使用```route```函数将我们的实现和路由路径关联起来。注意这里使用了函数式变成，传递的是```health_handler```等restful函数。

最后我们通过```axum::Server```将路由绑定到我们本地的监听上。

# 1.3 一个最简单的restful方法的实现

前面我们说了如何绑定我们的函数到路由上。在我们的代码中，为了维护方便，我们将我们的实现放到```handlers/general.rs```中。我们的```health_handler```如下实现：

``` Rust
pub async fn health_handler() -> Html<&'static str> {
    Html("<h1>Order server health ok.</h1>")
}
```

好了，到现在为止，我们就实现了一个最简单的axum服务。我们通过Cargo run来运行我们的服务，然后访问```127.0.0.1:3001```就可以得到一个简单的```Order server health ok```html响应。

# 1.4 带参数的restful实现

我们前面实现的```health_handler```不带任何参数。但是http请求中往往需要携带参数，而且我们也需要查询数据库。Axum框架实现了提取器，只要我们在方法上声明我们需要的参数，框架就会自动为我们匹配所需的参数。

在这里我们先定义两个方法，方法的定义如下：

 ``` Rust
pub async fn get_all_orders(
    State(pool): State<PgPool>,
) -> Result<axum::Json<Vec<Order>>, (StatusCode, String)>
 ```

 ``` Rust

 #[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddOrder {
    pub user_id: i64,

    pub items_id: Vec<String>,
    pub price: Vec<i32>,
    pub total_price: i32,
    pub currency: String,

    pub description: Option<String>,

    pub token: i64,
}

 pub async fn add_new_order(
    State(pool): State<PgPool>,
    Json(data): Json<AddOrder>,
) -> Result<axum::Json<AddOrderResult>, (StatusCode, String)>
 ```

这里涉及到两种提取器。一种是```State```，对应的是我们在创建路由的时候通过```with_state```方法设置的数据库连接池。```Json```提取器是用于提取http请求中的json对象，axum框架已经帮我们做好了json到对象的转换，我们只需要申明结构体就好。

另外还需要注意我们的返回值。Axum可以自动识别Result对象，一个成功的请求需要是```axum::Json```返回值，当请求失败的时候，我们需要返回```(axum::http::StatusCode, String)```，代表了http请求码和具体的错误描述。

当然，不要忘记在路由中添加我们新增的两个方法：
``` Rust
   let rest = Router::new()
        .route("/", get(health_handler))
        .route("/orders", get(get_all_orders))
        .route("/add_order", post(add_new_order))
        .with_state(db_pool);
```

# 1.5 数据库查询

接下来我们需要进行具体的数据库请求。这里我们使用```sqlx```框架。作为适当的解耦，我们将实现放到```db_access```中。
具体代码如下：

``` Rust
use axum::http::StatusCode;
use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;

use crate::models::{
    error::internal_error,
    order::{AddOrder, AddOrderResult, Order},
};

pub async fn get_all_orders_from_db(
    pool: &PgPool,
    user_id: i64,
    page: i64,
    page_size: i64,
) -> Result<Vec<Order>, (StatusCode, String)> {
    let offset = page_size * page;
    let orders = sqlx::query!(
        "SELECT * FROM orders WHERE user_id = $1 LIMIT $2 OFFSET $3",
        user_id,
        page_size,
        offset
    )
    .map({
        |row| Order {
            id: row.id,
            user_id: row.user_id,
            items_id: serde_json::from_str(row.items_id.as_str()).unwrap_or_default(),
            price: serde_json::from_str(row.price.as_str()).unwrap_or_default(),
            total_price: row.total_price,
            currency: row.currency.unwrap_or_default(),
            sub_time: NaiveDateTime::from(row.sub_time.unwrap()).timestamp_millis(),
            pay_time: NaiveDateTime::from(row.pay_time.unwrap()).timestamp_millis(),
            description: row.description,
        }
    })
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    println!("get_all_orders_from_db size: {}", orders.len());

    Ok(orders)
}

pub async fn add_new_order_from_db(
    pool: &PgPool,
    data: AddOrder,
) -> Result<AddOrderResult, (StatusCode, String)> {
    let des = data.description.unwrap_or_default();
    let price = data.price;

    println!("add_new_order des: {}", des);

    let item_ids_str = serde_json::to_string(&data.items_id).unwrap_or_default();
    let price_json = serde_json::to_string(&price).unwrap_or_default();

    let ts_1970 = NaiveDateTime::from_timestamp_opt(0, 0).unwrap_or_default();

    let _rows = sqlx::query!("INSERT INTO orders (user_id, items_id, price, total_price, currency, pay_time, description) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        data.user_id,
        item_ids_str, price_json,
        data.total_price, data.currency,
        ts_1970, des)
        .fetch_one(pool)
        .await
        .map_err(internal_error);

    let result = AddOrderResult {
        description: "add successed.".to_string(),
    };
    Ok(result)
}

```

我们使用```sqlx::query!```宏来实现数据库操作。这个宏可以在编译期检查数据库结构和代码的对应关系，避免错误的数据库操作被编译通过，同时可以防止数据库注入的漏洞。当然缺点也存在，就是编译代码的主机必须要有链接数据库的环境，这点在某些情况下比较麻烦。

你可能注意到了，我还使用了一个```internal_error```函数来处理异常情况。这个函数的实现如下：

``` Rust
use axum::http::StatusCode;

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
```
作为一个demo，为了简化，我将所有数据库查询错误都指定为http 500错误。实际生产中请根据实际情况进行修改。

另外你可能注意到了我使用```serde_json```来处理json序列号和反序列化，使用```NaiveDateTime```来处理时间和时间戳之间的转换。这里就不详细介绍这两个库的具体使用了。

然后我们在restful的函数中使用这两个数据库查询：

``` Rust
pub async fn get_all_orders(
    State(pool): State<PgPool>,
) -> Result<axum::Json<Vec<Order>>, (StatusCode, String)> {
    get_all_orders_from_db(&pool).await
}

pub async fn add_new_order(
    State(pool): State<PgPool>,
    Json(data): Json<AddOrder>,
) -> Result<axum::Json<AddOrderResult>, (StatusCode, String)> {
    add_new_order_from_db(&pool,data).await
}
```

这样一个简单的操作数据库的服务就实现了。

# 1.6 订单token

在实际生产中，客户端可能因为网络重试等原因，重复发起同一个http请求，导致订单被反复发起。这种情况可能给用户和商家造成损失，一种简单的解决方法是发起订单的之前先获取一个订单token，然后真正发起订单的时候附带上这个token，服务端在发起订单之后消费掉这个token，只要保证token不会重复消费，就可以避免订单的重复发起了。

生成的订单token我们可以直接存在内存里面，也可以使用redis缓存等保存。因为我们预计我们的服务是一系列微服务，所以建议使用一个统一的redis服务来保存我们的订单token。在本文中，我们暂时不使用redis，只做一下订单token的生成实践。

订单token的生成需要一定的随机性和离散型。常见的有雪花算法等。这里我们使用社区提供的```idgenerator```库。
``` Rust
[dependencies]
...

# 唯一id生成库，雪花算法
idgenerator = "2.0.0"
```

这个库在使用之前需要进行初始化：
``` Rust
// 雪花算法生成唯一id
let options = IdGeneratorOptions::new().worker_id(1).worker_id_bit_len(6);
// Initialize the id generator instance with the option.
// Other options not set will be given the default value.
let _ = IdInstance::init(options);
```

``` Rust
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NewOrderToken {
    pub token: i64,
}

pub async fn request_new_order_token() -> Result<axum::Json<NewOrderToken>, (StatusCode, String)> {
    let id = IdInstance::next_id();
    println!("request_new_order_token: {}", id);
    Ok(axum::Json(NewOrderToken { token: id }))
}
```