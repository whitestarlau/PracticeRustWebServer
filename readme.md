# 这是什么项目
这是一个个人的Rust微服务练习项目。使用axum web框架，实现了一个简单的商城服务，并提供了一个vue构建的简单前端页面。

项目地址位于 [PracticeRustWebServer on Github](https://github.com/whitestarlau/PracticeRustWebServer)。

## 项目结构说明
项目的大致结构如下：
```
.
├── Cargo.lock       
├── Cargo.toml       # 项目基础依赖
├── certify_server   # 登陆认证服务
├── common_lib       # 公共库，封装了一些基础共用方法
├── consul           # consul微服务发现中心的启动命令，仅供测试用
├── consul_reg_lib   # 封装了用于注册consul微服务发现中心的公共方法
├── front_page_vue   # 一个简单的前端页面，使用vue3实现
├── goods_server     # 商品微服务
├── inventory_server # 库存微服务
├── jwt_lib          # 封装了jwt生成和验证的公共方法
├── markdown         # 一些关于本项目的个人记录博客
├── order_server     # 订单微服务
├── proto            # 用于生成grpc的公共proto声明
└── target
```

## 项目说明
数据库使用```potgresql```，然后将运行环境中的数据库密码像.env文件一样进行配置。

比如在我的测试环境中：
```
DATABASE_URL=postgres://testUser:testPassword@localhost:5432/testDb2
DATABASE_URL_LOCAL=postgres://testUser:testPassword@localhost:5432/testDb2
```

配置数据库环境之后，使用不同微服务下面的```db_new.sql```命令生成对应的表。因为我的数据库框架使用的是sqlx，这个框架会进行编译期检查，如果数据库表在测试环境中不正确将无法编译通过。

另外JWT_SECRET是生成jwt所用的密钥。因为是个人测试项目，所以这里使用了对称加密算法。实际项目中请使用非对称加密算法，并不要泄漏私钥。

不同的微服务之间的发现我这里使用consul。测试环境中，我使用如下命令启动：

```
## consul_dev.shell
consul agent -dev -client=0.0.0.0
```


配置完环境后可以使用如下方式进行运行微服务：
```
cargo run -p certify_server
cargo run -p goods_server
cargo run -p order_server
cargo run -p inventory_server
```

更多说明请见```markdown```文件夹。