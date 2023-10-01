# Consul 微服务发现中心

在之前的几篇文章中，我们构建了订单、库存两个微服务，然后使用 GRPC 的方式实现了两者的相互调用。但是目前来说，我们相互调用的地址是通过硬编码实现的，这样做虽然能实现调用，但是很明显不利于后续的微服务横向扩展。

业界通用的处理办法是通过一个微服务发现、注册中心来实现这一过程。每一个微服务都在启动时向中心注册自己的存在，其他微服务需要 RPC 调用时通过这个中心来获取需要的微服务的地址。

服务的注册与发现是比较大的话题，我们不打算去实现一个自己的发现中心，而是使用业界现成的开源软件。在这里我们选用Consul，它是一个流行的开源的微服务注册与发现中心，由GO语言开发，使用```MPL2.0```协议。

## 1、注册

### 1.1 安装与启动

因为 Consul 是跨平台软件，并且在实际部署中 Consul 会部署在多个服务器上，我们不打算详细介绍这一软件的安装。你可以前往 [Consul Install](https://developer.hashicorp.com/consul/docs/install)。 来查看如何安装 Consul。

如果你和我一样安装一个“单机版”来测试，你可以使用如下命令来启动一个测试版的 Consul：

```
consul agent -dev -client=0.0.0.0
```

### 1.2 注册

通过查阅 Consul 的官方文档我们得知，Consul 的 API 都是可以通过 HTTP 进行调用的。这样通过查阅文档我们可以简单地实现 Consul 在 Rust 微服务中的使用。

为了使用 http 请求，我们需要引入 reqwest 这个 http-client 库。

```
# 用来请求consul中心的接口
reqwest = { version = "0.11", features = ["json"] }
urlencoding = "2"
```

然后我们实现一个简单的注册方法：

```Rust
#[derive(Serialize, Deserialize)]
pub struct ConsulOption {
    pub addr: String,
    pub timeout_sec: u64,
    pub protocol: String,
}

//TODO 默认直接使用本地8500的端口，真实项目中需要修改为环境变量或者配置文件配置
impl Default for ConsulOption {
    fn default() -> Self {
        Self {
            addr: String::from("127.0.0.1:8500"),
            timeout_sec: 1u64,
            protocol: "http".to_string(),
        }
    }
}

pub struct Consul {
    option: ConsulOption,
    client: reqwest::Client,
}

impl Consul {
    pub fn newDefault() -> Result<Self, reqwest::Error> {
        return Consul::new(ConsulOption::default());
    }

    pub fn new(option: ConsulOption) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(option.timeout_sec))
            .build()?;
        Ok(Self { option, client })
    }

    pub async fn register(&self, registration: &Registration) -> Result<(), reqwest::Error> {
        self.client
            .put(self.api_url("service/register"))
            .json(registration)
            .send()
            .await?;
        Ok(())
    }
}
```

可以看到我们需要序列化的`Registration`对象是关键，这个对象我们需要查阅 Consul 的官方文档，得到需要定义如下字段：

```Rust
pub struct Registration {
    pub name: String,
    pub id: String,
    pub tags: Vec<String>,
    pub address: String,
    pub port: i32,
}
```

大部分字段都不言自明，在这里就不过多进行介绍了。实际上我们还需要实现一个 new 方法来方便创建对象，这个方法比较简单我们就不介绍了。

然后在订单微服务中，我们进行如下调用：

```Rust

async fn web_server() {
    tokio::spawn(register_consul(&addr));
}

async fn register_consul(addr: &str) {
    println!("register consul doing...");
    let addrs: Vec<&str> = addr.split(":").collect();
    let addr = addrs[0];
    let port: i32 = addrs[1].parse().unwrap();
    let opt = consul_reg_lib::model::ConsulOption::default();
    let cs = consul_reg_lib::consul::Consul::new(opt).unwrap();

    println!("register consul health_check params:{:?}", health_check);

    //register consul name as order-srv.
    let reg = consul_reg_lib::model::Registration::new(
        "order-srv",
        "order-srv",
        vec![],
        addr,
        port,
    );

    cs.register(&reg).await.unwrap();
    println!("register consul done.");
}

```

## 2、健康检查

一个微服务在运行过程中可能出现宕机、网络故障等失联的情况。为了应对这种情况，我们离不开健康检查。

查阅 Consul 的文档，发现它支持 http、GRPC 等多种健康检查。GRPC 的健康检查相对较为复杂，这里我们只添加 http 的检查。我们在注册的消息中添加相应的字段：

```Rust
/**
 * deregister_critical_service_after 服务critical多久之后将会被注销
 * http 检查url
 * interval 检查间隔
 */
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct HealthCheck {
    pub deregisterCriticalServiceAfter: String,
    pub http: String,
    pub interval: String,
}

impl HealthCheck {
    /**
     *  新建一个健康检查的参数，默认30m分钟废弃，20s一检查
     */
    pub fn new(http: String) -> Self {
        return Self {
            deregisterCriticalServiceAfter: "30m".to_string(),
            http: http,
            interval: "20s".to_string(),
        };
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Registration {
    pub name: String,
    pub id: String,
    pub tags: Vec<String>,
    pub address: String,
    pub port: i32,
    pub check: HealthCheck,
}
```

通过“http”字段来标记我们的 http 健康检查地址。然后我们修改我们的代码：

```Rust
async fn web_server() {
    let health_check_path = "/health_check";

    let rest = Router::new()
        .route(health_check_path, get(health_handler))
        //...

    let addr = "127.0.0.1:3002";

    tokio::spawn(register_consul(&addr, health_check_path));
}

async fn register_consul(addr: &str, health_check_path: &str) {
    //...
    let health_check_url = format!("http://{}:{}{}", addr, port, health_check_path);

    let health_check = consul_reg_lib::model::HealthCheck::new(health_check_url.to_string());

    println!("register consul health_check params:{:?}", health_check);

    //register consul name as order-srv.
    let reg = consul_reg_lib::model::Registration::simple_with_health_check(
        "order-srv",
        addr,
        port,
        health_check,
    );

    //...
}

```

可以想见在这里我们的健康检查地址是```http://127.0.0.1:3002/health_check```。我们在```health_handler```中添加一些打印:

``` Rust

pub async fn health_handler() -> Html<&'static str> {
    println!("some one call health check api.{}",Utc::now());
    Html("<h1>Order server health ok.</h1>")
}
```

然后运行我们的任务，就可以看到Consul会周期性地检查我们的任务了，时间间隔为我们设定的20s：

``` 
start corn sched in main.
start web_server in main.
start corn sched.
listening on 127.0.0.1:3002
register consul doing...
register consul health_check params:HealthCheck { deregisterCriticalServiceAfter: "30m", http: "http://127.0.0.1:3002/health_check", interval: "20s" }
register consul done.
some one call health check api.2023-10-01 09:50:22.057937 UTC
some one call health check api.2023-10-01 09:50:42.060176 UTC
```
## 3、服务发现

在上一个小节中我们完成了订单微服务向Consul注册的流程，我们可以如法炮制库存微服务的相应时间。在都完成之后，我们还需要实现查询对应微服务的代码。

我们为之前的Consul结构体添加如下方法实现：

``` Rust
#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Service {
    #[serde(rename = "ID")]
    pub id: String,
    pub service: String,
    pub tags: Vec<String>,
    pub address: String,
    pub port: i32,
    pub datacenter: String,
}

pub type Services = HashMap<String, Service>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Filter {
    Service(String),
    ID(String),
}


//

impl Consul {
    //...
    
    pub async fn services(&self) -> Result<Services, reqwest::Error> {
        let list: Services = self
            .client
            .get(self.api_url("services"))
            .send()
            .await?
            .json()
            .await?;
        Ok(list)
    }
    
    
    pub async fn get_service(&self, filter: &Filter) -> Result<Option<Service>, reqwest::Error> {
        let list = self.services().await?;
        for (_, s) in list {
            let has = match &filter {
                &Filter::ID(id) => id == &s.id,
                &Filter::Service(srv) => srv == &s.service,
            };
            if has {
                return Ok(Some(s));
            }
        }
        Ok(None)
    }
}
```
services可以用于查询所有的微服务，get_service可以用于查询指定的微服务。可以注意到我们的Services实际上是一个Hashmap。

在调用时我们这样使用：

``` Rust

pub async fn add_new_order(
    claims_op: Option<Claims>,
    State(state): State<AppState>,
    Json(data): Json<AddOrder>,
) -> Result<axum::Json<AddOrderResult>, (StatusCode, String)> {
    //..
    let cs = consul_reg_lib::consul::Consul::newDefault().map_err(map_consult_error)?;
    let filter = consul_reg_lib::model::Filter::ID(state.inventory_srv_id);
    let srv_option : Option<Service> = cs.get_service(&filter).await.map_err(map_consult_error)?;

    //..
}

```

很明显，我们每次使用都需要查询所有服务，然后过滤出我们需要的服务。这种实现并非最佳。Consul还有```/agent/service/{service_id}```这样的直接查询指定服务的http api，但是时间所限，在我们的demo中暂时不打算介绍，读者可以自行查阅Consul的API文档进行实现，当作一种练习。

我们再总结一下上面这些实现做到的事情。我们让一个微服务向Consul微服务中心注册报道，同时Consul微服务中心会定期检查我们的微服务是否正常而没有出现服务中断。然后我们其他的微服务通过请求Consul，来获取需要的微服务的地址在进行RPC调用，避免需要在代码中硬编码微服务地址的问题，进而利于一个微服务的横向扩展。

## 4、通过 Workspace 共享代码

在本章前面的几个小节中我们实现了向 Consul 注册微服务的代码。这一部分代码很明显是可以在不同的微服务之间通用的，按照我们之前的方式，需要将代码拷贝到每个仓库中才可以实现。很明显这是一种很繁琐的方式。

有多种方式可以共享我们的Rust代码，比如Workspace（工作空间）、上传Cargo Lib等。这里因为我们项目的定制化目的比较强，所以采用Workspace的方式来共享lib会比较好。本小节将对项目改为Workspace所需的调整做简单的介绍。

我们在我们两个微服务的共同上一级目录添加一个```Cargo.toml```文件，然后整个项目的文件夹应当如下：
```
.
├── Cargo.toml       # 项目基础依赖
├── consul_reg_lib   # 封装了用于注册consul微服务发现中心的公共方法
├── goods_server     # 商品微服务
├── inventory_server # 库存微服务
├── proto            # 用于生成grpc的公共proto声明
└── target
```

我们为根目录的```Cargo.toml```加入如下代码：

``` Rust
[workspace]

members = [
    "order_server",
    "inventory_server",
    "consul_reg_lib",
]
```

这样就简单建立了一个共享的workspace，我们通过```members```声明当前有哪些子文件夹会被包含在这个workspace中。在这个workspace的根目录，我们可以通过```Cargo run -p xxx```来运行我们指定的成员，效果等同于在子文件夹中运行```Cargo run```。

我们还可以注意到，在一个子工程编译以后，会在Workspace的根目录生成一个```Cargo.lock```文件。这样每个工程的依赖版本锁定会由Workspace来控制，不同工程之间的lock共享，避免出现依赖冲突的问题。

可以注意到我们有新建一个```consul_reg_lib```的文件夹。这里将我们的Consul相关的实现代码全部移动到此文件夹中，这一个lib的文件夹结构如下：

``` 
.
├── Cargo.toml
└── src
    ├── consul.rs
    ├── lib.rs
    └── model.rs
```
通过```lib.rs```我们表明我们的这个工程是一个库。然后我们可以在同一Workspace的其他工程下引入这个lib，比如在库存微服务中。

``` Rust
[package]
name = "inventory_server"
version = "0.1.0"
edition = "2021"
default-run = "inventory-service"

[dependencies]
consul_reg_lib = { path = "../consul_reg_lib" }
//..
```

这样通过Workspace，我们就简单地实现了在不同的工程下面共享同一块代码的目的。

## 5、总结与后续

在本章中，我们通过Consul微服务中心和它的http api实现了微服务的注册、发现，并通过Workspace实现了lib代码的共享。

读者可以注意到我们的项目中还存在大量的问题，比如说现在用户id还是一个简单long型数字，而且缺少相应的注册、登陆机制。在下一章中我们将要实现一个JWT的单点登录，并在每个微服务中集成JWT验证服务。
