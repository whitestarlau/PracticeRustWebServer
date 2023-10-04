# JWT 单点登录

在之前的服务中，我们都使用一个 long 型来代表用户。这种设计是不合理的，如果我们是使用数据库主键递增来实现，很容易泄漏我们的用户数量、注册先后顺序等信息，对于商业公司来说这是一种很重要的商业机密。

一种结局办法是使用数据库的 uuid 生成功能，比如在 Postgresql 中这样声明字段：

```SQL
create table users (
       id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

       email varchar(200),
       password_hash varchar(200),

       create_time TIMESTAMP default now()
);
```

显然我们不能用户直接拿着 uuid 来向我们请求接口，这样攻击者可以随意伪造这种请求。因为我们使用的是微服务，所以我们需要使用一种被叫做“单点登录”的方式，JWT 就是来干这件事的。

## 1、JWT 简介

在正式开始我们的 Rust 代码前，我们有必要过一下 JWT 的概念。

JWT 是 JSON Web Tokens 的缩写，一般来说包含在 Http 请求的 Header 中。一个 JWT 包含 3 个部分：头部 Header，数据 Payload，签名 Signature，用点做间隔。比如我们这里有一个典型的 JWT 信息：

```
Authorization:Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.
eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWV9.TJVA95OrM7E2cBab30RMHrHDcEfxjoYZgeFONFh7HgQ
```

Bearer后面的这串字符串就是JWT的具体信息。JWT 的前两个部分可以直接使用 Base64 解码，我们解压可以得到如下信息：

```
Header：
{"alg":"HS256","typ":"JWT"}

Payload：
{"sub":"1234567890","name":"John Doe","admin":true}
```

Header 部分我们可以看到，申明了数据的类型为 JWT，并说明签名加密类型是 HS256。而 Payload 部分就是我们的业务数据了。我们可以在这里放置我们的用户名、uuid、过期时间等。

最后一部分就是签名信息了，这部分信息是使用前两部分一起加密得到的。我们拿到一个 JWT 之后，需要先验证签名，任何中间人篡改了 Header 或 Payload 中的信息，第三块加密的部分都无法验证成功。这样就保证了一个验证通过的 JWT 信息一定是我们的登录中心的密钥签出的，进而验证了用户信息是可靠的。

当然，如果我们的 JWT 整个泄漏了，攻击者依旧可以假装为自己是用户。针对这种情况，我们可以在第二部分加入额外的信息，比如过期时间，登录过期的用户需要重新登录。这样即使签出的 JWT 泄漏了，也只能在一段时间内生效，降低了用户的损失。另外还可以通过 IP 地址异常等方式来阻止异常的用户行为，当然这就不在本文的探讨范围之内了，暂时不做进一步讨论。

## 2、注册及登录微服务

### 2、1 jwt 库

我们将注册登录拆分为一个单独的微服务，名为 certify_server。因为我们验证 JWT 的实现所有微服务都需要使用，那么我们也另外新建一个`jwt_lib`模块。除了常见的依赖，我们需要如下这些依赖：

```Rust
[dependencies]
//...
# 密码hash
bcrypt = "0.10"

# jwt生成及验证
jsonwebtoken = "7.2"

# uuid生成
uuid = { version = "1.4.0", features = ["serde", "v4"] }

# 用于帮助解析请求头
headers = "0.3"

//...
```

让我们先了解一下 jsonwebtoken 这个库的几个主要方法签名：

```Rust
pub fn encode<T: Serialize>(header: &Header, claims: &T, key: &EncodingKey) -> Result<String>

impl Default for Header {
    /// Returns a JWT header using the default Algorithm, HS256
    fn default() -> Self {
        Header::new(Algorithm::default())
    }
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::HS256
    }
}

pub struct EncodingKey {
    pub(crate) family: AlgorithmFamily,
    content: Vec<u8>,
}

```

这个库实现了主要的算法的 JWT 签名，默认使用 HS256 签名，这是一种对称加密算法，在真正的项目中应当使用非对称加密算法较好，所有微服务都持有公钥，用于验证 JWT，而只有登录微服务持有私钥，用于签发 JWT。

claims 是由我们自己声明的。在这里我们申明如下字段，包含了 uuid、签名时间、失效时间。

```Rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(id: Uuid) -> Self {
        let iat = Utc::now();
        let exp = iat + Duration::hours(24);

        Self {
            sub: id,
            iat: iat.timestamp(),
            exp: exp.timestamp(),
        }
    }
}
```

我们用如下方法进行签名：

```Rust
pub fn sign(id: Uuid, encodingKey: &EncodingKey) -> Result<String, Error> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(id),
        encodingKey,
    )?)
}
```

### 2、2 用户表和密码验证

我们现在需要一个用户表来存储用户信息，最简单的表需要包含 uuid 和密码的关系。但是为了安全起见，我们的数据库不可以明文存储用户的密码信息，一般的做法是保存密码的 hash 信息进行对比验证。一种常见的做法被称为 bcrypt，除了 hash 之外，还会在密码中加盐，进一步降低密码被 hash 碰撞破解的风险。

我们需要引入如下库：

```Rust
[dependencies]
//...
# 密码hash
bcrypt = "0.10"
```

然后我们设计一下用户表，简单起见，我们只存储 uuid、email、用户 hash、创建时间这三项：

```SQL
create table users (
       id UUID DEFAULT gen_random_uuid() PRIMARY KEY,

       email varchar(200),
       password_hash varchar(200),

       create_time TIMESTAMP default now()
);
```

在验证用户密码的时候，我们从数据库表查询出 hash，然后使用如下方法验证密码是否正确：

```Rust
encryption::verify_password(password1, password2)
```

创建用户时使用如下方式写入用户表：

```Rust
#[derive(Deserialize, Validate, Serialize, Debug, Clone)]
pub struct SignUser {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

pub async fn find_user_by_email(
    pool: &PgPool,
    email: String,
) -> Result<User, (StatusCode, String)> {
    let users = sqlx::query!("SELECT * FROM users WHERE email = $1", email,)
        .map({
            |row| User {
                id: row.id,
                user_email: row.email.unwrap_or_default(),
                password_hash: row.password_hash.unwrap_or_default(),
                create_time: NaiveDateTime::from(row.create_time.unwrap()).timestamp_millis(),
            }
        })
        .fetch_one(pool)
        .await
        .map_err(internal_error)?;

    // info!("get_user size: {}", users);

    Ok(users)
}

pub async fn add_new_user_from_db(
    pool: &PgPool,
    user: SignUser,
) -> Result<Uuid, (StatusCode, String)> {
    println!("add_new_user_from_db user: {}", user.email);

    let email = user.email.clone();

    let find_user = find_user_by_email(pool, user.email).await;
    if let Ok(f_user) = find_user {
        println!("add_new_user_from_db but find registered user {:?}.", f_user);

        //这个email已经注册过了。
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "DuplicateUserEmail".to_string(),
        ));
    } else {
        let pwd = user.password.clone();

        let ts_1970 = NaiveDateTime::from_timestamp_opt(0, 0).unwrap_or_default();
        let password_hash = encryption::hash_password(user.password)
            .await
            .map_err(internal_error_dyn)?;

        println!("add_new_user_from_db password_hash: {}", password_hash);

        let insert_result: Result<Uuid, (StatusCode, String)> = sqlx::query!(
            "INSERT INTO users (email, password_hash, create_time) VALUES ($1, $2, $3) RETURNING id",
            email,
            password_hash,
            ts_1970,
        )
        .map(|row| row.id)
        .fetch_one(pool)
        .await
        .map_err(internal_error);

        match insert_result {
            Ok(user_id) => Ok(user_id),
            Err(e) => Err(e),
        }
    }
}
```

### 2、3 用户注册方法

准备好了用户表、密码验证和 JWT 后，我们就可以申明注册的方法了。

因为我们使用的是对称加密算法，所以这里简单使用从环境变量来获取密钥：

```Rust
lazy_static! {
    pub static ref JWT_SECRET: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
}
```

注册用户方法如下。登录方法和注册极为相似，只是将新增用户改为查询用户然后使用 bcrypt 验证密码 hash，这里就不浪费篇幅了：

```Rust


/**
 * 注册
 */
pub async fn sign_up(
    State(state): State<AppState>,
    Json(user): Json<SignUser>,
) -> Result<axum::Json<SignUserResp>, (StatusCode, String)> {
    validate_payload(&user).map_err(internal_error)?;
    let addResultId = add_new_user_from_db(&state.pool, user).await?;

    println!("sign_up add_new_user_from_db success.");

    let encodingKey: EncodingKey = EncodingKey::from_secret(env::JWT_SECRET.as_bytes());
    let token = jwt::sign(addResultId, &encodingKey).map_err(internal_error)?;
    let token_payload = TokenPayload {
        access_token: token,
        token_type: "Bearer".to_string(),
    };

    return Ok(Json(SignUserResp {
        uid: addResultId,
        token: token_payload,
    }));
}

```

## 3、Token 自动验证

还记得我们说过 JWT 在 header 中吗？如果我们每个方法都使用 Axum 的默认提取器取得 header 然后再验证 jwt，代码上会很麻烦。

在 Axum 中，我们只需要为 JWT 的 Claims 结构体实现提取器，即`FromRequestParts`特性：

```Rust

pub fn verify(token: &str, decodeKey: &DecodingKey) -> Result<Claims, Error> {
    Ok(
        jsonwebtoken::decode(token, decodeKey, &Validation::default())
            .map(|data: jsonwebtoken::TokenData<Claims>| data.claims)?,
    )
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let deKey = &DecodingKey::from_secret(JWT_SECRET.as_bytes());

        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
        // Decode the user data
        let claims = verify(bearer.token(), deKey)
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;

        Ok(claims)
    }
}

```

可以注意到我们这里使用了`RequestPartsExt`的`extract`函数匹配`TypedHeader`和`Authorization`来帮助我们快速提取 header 中的 JWT，这里的这个匹配实现很有意思，但是不是我们这篇文章的重点，所以这里不做展开介绍。

有了这个提取器，我们就可以在函数方法中直接申明我们需要`Claims`了，Axum 会自动匹配我们申明的自定义提取器并使用，比如我们的验证 token 方法：

```Rust
pub async fn verify_token(
    claims_op: Option<Claims>,
) -> Result<axum::Json<bool>, (StatusCode, String)> {
    if let Some(claims) = claims_op {
        return Ok(Json(true));
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "wrong claims".to_string(),
        ));
    }
}
```

有了这个方法，我们之前的所有微服务都可以做相应的改造，简单引入`jwt_lib`库，然后修改对应的方法即可。比如我们的新增订单方法：

```Rust
pub async fn add_new_order(
    claims_op: Option<Claims>,
    State(state): State<AppState>,
    Json(data): Json<AddOrder>,
) -> Result<axum::Json<AddOrderResult>, (StatusCode, String)> {
    if let Some(claims) = claims_op {
        let uuid = claims.sub;
        //...
    } else {
        //...
    }
}
```

非常得直观方便。

# 4、总结

在这一章中，我们完成了用户注册、登录的微服务，实现了hash密码验证、jwt的签发与验证，并将其引入我们所有的现存微服务中。至此，我们就完成了一套较为完善的微服务单点用户登录实践。