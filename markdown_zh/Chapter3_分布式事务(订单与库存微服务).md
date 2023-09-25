# 分布式事务

**本章涉及到微服务中没有事实标准的“分布式事务”概念，作者在这方面也是属于初学者，如果存在理解错误，请各位读者批评指正。**

在前两章我们构建了两个微服务，分别负责管理订单和库存。按照正常的业务逻辑来说，我们在订单产生的时候需要调用库存微服务，来尝试扣减库存。为了实现不同微服务之间的调用，我们需要通过 GRPC 进行调用，这一步我们已经在上一章进行了实现。

但是 RPC 调用毕竟是通过网络进行远程调用，因为各种原因，存在各种复杂情况（网络故障、微服务宕机等），可能导致远程服务调用失败。这种场景和数据库中的“事务”很像，数据库的事务可以保证多个原子操作按照我们的期望得到执行，而不是某几个成功某几个失败，进而产生数据不一致的问题。在微服务涉及中，我们也需要通过某种机制来保证我们订单整个是一个“事务”，进而保证各个微服务的数据是一致的，这种机制通常被称为“分布式事务”。

分布式事务并不存在一种学术标准或者说事实标准，有多种实现方式。常见的有 2PC、3PC、TCC、本地消息表、消息事务等。不同的实现方式都存在各自的优点和缺点，在这里我们不进一步分析不同实现方式的优缺点，因为我们的“电商业务”对时延没有那么高的要求，且从初学者实现方便的角度，先采用相对容易实现的本地消息表。

## 1、定义本地消息表

本地消息表顾名思义就是会有一张存放本地消息的表，一般通过数据库存放。在执行业务的时候 将本地数据库业务执行，还有对消息表中操作放在同一个事务中，这样就能保证本地业务和本地消息表的一致性。然后再去调用下一个 RPC 操作，如果 RPC 操作调用成功了，消息表的消息状态可以直接改成已成功。如果调用失败了，那么我们由后台任务的业务轮询对失败消息的业务进行重试。实际操作中，如果失败了次数过多，我们还可以引入报警和人工处理。当然，我们的 demo 只是为了示范，暂时不加入报警人工处理这一个实现。

既然决定采用本地消息表的方式，那我们就需要先定义我们的消息列表需要存放什么信息。我们的消息是从订单微服务发出，去扣减库存微服务的库存的，所以需要关联订单 id，同时我们后期查询方便，也关联一下用户 id。

```Rust

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OrderDeInventoryMsg {
    pub id: i32,
    pub user_id: i64,

    pub order_id: i32,
}

```

在这里我需要说明的是，随着我们项目的完善，我们需要逐渐修改我们之前设定的数据库结构。比如在最开始，我们设定订单数据结构如下：

```Rust
pub struct AddOrder {
    pub user_id: i64,

    pub items_id: Vec<String>,
    pub price: Vec<i32>,
    pub total_price: i32,
    pub currency: String,

    pub description: Option<String>,

    pub token: i64,
}
```

但是实际上我在后续发现同一个订单中维护多个商品，在业务逻辑上并不好。所以我们修改为如下结构：

```Rust

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddOrder {
    pub user_id: i64,

    pub items_id: i32,
    pub price: i32,
    pub count : i32,
    pub currency: String,

    pub description: Option<String>,

    pub token: i64,
}

```

此外，为了标志一个订单的库存是否扣减成功，我们再为订单表新增一个库存扣减的标志，所以最终的表结构如下：

```sql
create table orders (
       id serial primary key,

       user_id UUID not null,

       item_id INT not null,
       price INT not null,
       count INT not null,

       currency varchar(2000),

       sub_time TIMESTAMP default now(),
       pay_time TIMESTAMP default '1970-01-01 00:00:00',

       inventory_state INT not null DEFAULT 0,

       description varchar(140)
);
```

我们在数据库中使用 INT 类型来表达库存扣减的状态，在 Rust 中我们使用枚举来进行表达：

```Rust
#[derive(FromPrimitive)]
pub enum InventoryState {
    DOING = 0,
    SUCCESS = 1,
    FAIL = 2,
}
```

## 2、本地消息表的使用

在此前的文章中，我们已经实现了向订单数据库表中插入数据。现在新增了一个库存扣减消息表，但是需要注意的是，我们不希望因为程序的异常，出现订单数据增加了，但是消息数据库未插入的情况。所以我们必须要使用本地的数据库事务来保证两个表的数据一致性。

在我们使用的 sqlx 中，使用如下方式来开启一个数据库事务：

```Rust
let mut conn = pool.acquire().await.unwrap();
let mut tx = conn.begin().await.map_err(internal_error)?;
```

然后我们执行我们的业务逻辑，插入订单数据，然后插入库存扣减消息。注意我们默认设置新订单为`InventoryState::DOING`，来表达我们的订单库存还在扣减中，在最后RPC调用成功后我们将会修改这一字段：

```Rust
    let insert_order =  sqlx::query!("INSERT INTO orders (user_id, item_id, price, count, currency, pay_time, description,inventory_state) VALUES ($1, $2, $3, $4, $5, $6, $7,$8) RETURNING id",uuid,data.items_id, data.price,data.count, data.currency,ts_1970, des,InventoryState::DOING as i32)
                .map(|row| row.id)
                .fetch_one(&mut tx)
                .await;

    let mut order_id_cp = -1;
    let result = match insert_order {
        Ok(order_id) => {
            order_id_cp = order_id;

            println!("insert_order suceess");

            let insert_msg = sqlx::query!(
                "INSERT INTO orders_de_inventory_msg (user_id, order_id) VALUES ($1, $2) RETURNING id",
                uuid,
                order_id,
            )
            .map(|row| row.id)
            .fetch_one(&mut tx)
            .await
            .map_err(internal_error);

            let innerResult = if let Err(e) = insert_msg {
                println!("insert_msg fail should rollback.");

                Err(e)
            } else {
                println!("insert_msg success,try rpc call.");


                let addResult = AddOrderResult {
                    description: "add successed.".to_string(),
                };
                Ok(addResult)
            };

            innerResult
        }
        Err(e) => {
            println!("insert_order failed should rollback.");

            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    };

    if let Ok(_) = result {
        tx.commit().await.unwrap();
        //TODO 远程RPC调用
    } else {
        tx.rollback().await.unwrap();
    }
```

请注意最后的判断，如果数据库全部操作成功，则调用 commit 进行提交，如果存在失败，则全部数据库操作都需要进行回退。

我们还留了一个 TODO，即进行远程 RPC 调用，来实现让库存微服务进行库存扣减操作。在库存扣减成功后，我们需要将消息表中的消息删除，然后将订单表的对应标志设为库存扣减成功：

```Rust
/**
 * 扣减库存，并更新本地数据库。
 * 没有返回，调用者不关心这个函数的执行情况，因为结果是会放到数据库中，并由定时器定期轮询检查。
 */
pub async fn deduction_inventory(
    pool: &PgPool,
    inventory_addr: String,
    items_id: i32,
    count: i32,
    order_id: i32,
) {
    //分布式事务，扣减库存
    let deducation_resp = deduction_inventory_call(inventory_addr, items_id, count, order_id).await;
    if let Ok(resp) = deducation_resp {
        //响应为success的时候我们记录扣减库存成功
        let inventory_state = if InventoryResult::SUCCESS as i32 == resp.result {
            InventoryState::SUCCESS
        } else {
            InventoryState::FAIL
        };

        //删除消息数据库
        let mut conn = pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.map_err(internal_error)?;

        //注意，这一步可能写成功也可能写失败，所以可能导致deduction_inventory_call反复被调用，库存那边需要保证同一个订单id不会重复扣减。
        let _update_msg_result = sqlx::query!(
            "DELETE FROM orders_de_inventory_msg where  order_id = ($1)",
            order_id
        )
        .fetch_one(&mut tx)
        .await
        .map_err(internal_error);

        //标记扣减库存成功或者失败
        let _update_result = sqlx::query!(
            "UPDATE orders SET inventory_state = ($1) where id = ($2)",
            inventory_state as i32,
            order_id
        )
        .fetch_one(&mut tx)
        .await
        .map_err(internal_error);

        if let Ok(_) = _update_msg_result {
            if let Ok(a) = _update_result {
                tx.commit().await.unwrap();
            } else {
                tx.rollback().await.unwrap();
            }
        } else {
            tx.rollback().await.unwrap();
        }
    } else {
        // 远程调用失败，不代表扣减库存失败，等待定时器轮训的时候继续尝试
    }
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
``` 

可以注意到，这里RPC调用后我们操作业务表和消息表的时候依旧是通过数据库事务来调用的，时刻要记得保证两个本地表的数据一致性。

最后，我们将这个RPC调用放到最开始的订单操作中。有两点我们需要在意：
1. 在这里我先不关注RPC调用成功与否，但是如果真实的业务中需要在首次用户库存扣减失败的情况下给予用户提示，那么我们需要将RPC调用结果在返回结果中进行体现。
2. ```deduction_inventory```需要在数据库事务commit之后调用而不是之前，是因为我们后续还要写对应的数据库表字段。

``` Rust
    if let Ok(_) = result {
        tx.commit().await.unwrap();
        deduction_inventory(pool, inventory_addr, data.items_id, data.count, order_id_cp)
                    .await;
    } else {
        tx.rollback().await.unwrap();
    }
```

## 3、定时进行失败重试

到这里我们就已经实现了本地消息表的主要逻辑。但是还有一环没有完成，就是在RPC调用失败导致库存扣减失败之后怎么办。一般来说有如下几种操作：
1. 前端给用户一个提示，让用户手动触发重试；
2. 实现一个定时任务，定时进行重试；
3. 在多次失败后进行报警，进行人工修正处理；

其中1、2亮点在实际的业务中是需要的，但是比较依赖人工。我们在这里先实现2，定时任务。

常见的定时任务通常用corn的方式，在tokio生态中存在一个库可以实现定时任务的需求，我们引入依赖：
```
# cron定时器
tokio-cron-scheduler = "0.9.4"
```

然后我们实现一个定时任务：

``` Rust
async fn corn_aysnc() {
    let mut sched = JobScheduler::new().await.unwrap();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set.");
    let local_database_url =
        env::var("DATABASE_URL_LOCAL").expect("DATABASE_URL_LOCAL should be set.");

    let db_pool_arc = Arc::new(PgPoolOptions::new().connect(&database_url).await.unwrap());

    let job = Job::new("1/10 * * * * *", move |uuid, l| {
        let span = span!(Level::TRACE, "corn_async");
        let enter = span.enter();

        let now = Utc::now().timestamp_millis();

        info!("I run every 10 seconds ts:{}", now);

        let db_pool = db_pool_arc.clone();
        poll_inventory_state_order_from_db(
            &db_pool,
            "https://127.0.0.1:3001".to_string(),
        );
    })
    .unwrap();

    //定时任务必须要启动成功，不能失败，所以直接用unwarp
    sched.add(job).await.unwrap();
    sched.start().await.unwrap();

    println!("start corn sched.");

    loop {
        //一直等待，这里的定时任务要永久执行下去。
        if let Ok(Some(it)) = sched.time_till_next_job().await {
            info!("time_till_next_job {:?}", it);
            tokio::time::sleep(it).await;
        };
    }
}
```

我们使用```1/10 * * * * *```来定义任务，代表每10s一次。而corn库实际上只为我们实现了一个简单的任务计时操作，最后我们还是需要通过tokio运行时来运行我们的job。

轮询的任务实现逻辑比较简单，就是查询本地消息表，然后进行RPC调用：

``` Rust
pub async fn poll_inventory_state_order_from_db(
    pool: &PgPool,
    inventory_addr: String,
) {
    let orders_msg: Result<Vec<OrderDeInventoryMsg>, _> =
        sqlx::query!("SELECT * FROM orders_de_inventory_msg",)
            .map({
                |row| OrderDeInventoryMsg {
                    id: row.id,
                    user_id: row.user_id,
                    order_id: row.order_id,
                }
            })
            .fetch_all(pool)
            .await
            .map_err(internal_error);


    match orders_msg {
        Ok(msg_list) => for msg in msg_list {
            try_de_inventory(pool, inventory_addr.clone(), msg);
        },
        Err(e) => {
            //print error msg;
        }
    }
}

async fn try_de_inventory(
    pool: &PgPool,
    inventory_addr: String,
    msg: OrderDeInventoryMsg,
) {
    let orders = sqlx::query!("SELECT * FROM orders WHERE id = $1", msg.order_id,)
        .map({
            |row| Order {
                id: row.id,
                user_id: row.user_id,
                item_id: row.item_id,
                price: row.price,
                count: row.count,
                currency: row.currency.unwrap_or_default(),
                sub_time: NaiveDateTime::from(row.sub_time.unwrap()).timestamp_millis(),
                pay_time: NaiveDateTime::from(row.pay_time.unwrap()).timestamp_millis(),
                description: row.description,
                inventory_state: row.inventory_state,
            }
        })
        .fetch_one(pool)
        .await
        .map_err(internal_error);

    if let Ok(order) = orders {
        deduction_inventory(
            pool,
            inventory_addr,
            order.item_id,
            order.count,
            msg.order_id,
        );
    }
}

```

最后，我们需要执行我们的corn任务。在最开始的时候，我们使用```#[tokio::main]```这个宏来让我们的主函数转换成tokio运行的函数。但是我们现在新增的corn循环会阻塞掉整个tokio运行时，这个时候我们就需要使用新的方式来启动我们的主函数：

``` Rust
fn main() {
    thread::spawn(|| {
        //定时任务，用于定时轮询本地消息列表中有没有失败的任务没有处理
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            println!("start corn sched in main.");
            corn_aysnc().await;
        });
    });

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        println!("start web_server in main.");
        web_server().await;
    });
}
```

## 4、总结

在这一节中，我们通过本地消息表的方式实现了一个分布式微服务。关键点如下：

1. 通过本地数据库事务来保证业务表和消息表的数据一致性；
2. 通过RPC调用来调用其他微服务，然后更新本地数据库；
3. 通过定时任务来在RPC调用失败的时候进行重试；

读者可能发现了，我们调用库存微服务是通过硬编码的地址来调用的。但是我们既然实现了微服务，那么肯定是希望微服务可以进行横向扩展的，但是写死地址的方式肯定不满足我们的要求。在下一章中，我们将介绍如何用Consul实现微服务的注册和发现。