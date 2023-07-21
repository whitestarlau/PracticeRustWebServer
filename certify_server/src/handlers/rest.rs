use std::f32::consts::E;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    Json,
};

use jsonwebtoken::EncodingKey;
use jwt_lib::{
    encryption,
    jwt::{self, Claims},
};

use common_lib::{
    validate_payload,internal_error,internal_error_dyn,
};

use tracing::{info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::{constants::BEARER, env},
    db_access::db::{add_new_user_from_db, find_user_by_email},
    models::{
        state::AppState,
        user::{SignUser, SignUserResp, TokenPayload},
    },
};

#[instrument]
pub async fn health_handler() -> Html<&'static str> {
    println!("some one call health check api.");
    Html("<h1>Certify server health ok.</h1>")
}

/**
 * 注册
 */
#[instrument]
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

/**
 * 登陆
 */
#[instrument]
pub async fn sign_in(
    State(state): State<AppState>,
    Json(user): Json<SignUser>,
) -> Result<axum::Json<SignUserResp>, (StatusCode, String)> {
    validate_payload(&user).map_err(internal_error)?;
    let find_user = find_user_by_email(&state.pool, user.email).await?;

    let verify_password = encryption::verify_password(user.password, find_user.password_hash)
        .await
        .map_err(internal_error_dyn)?;

    if verify_password {
        let encodingKey: EncodingKey = EncodingKey::from_secret(env::JWT_SECRET.as_bytes());

        let token = jwt::sign(find_user.id, &encodingKey).map_err(internal_error)?;
        let token_payload = TokenPayload {
            access_token: token,
            token_type: "Bearer".to_string(),
        };
        return Ok(Json(SignUserResp {
            uid: find_user.id,
            token: token_payload,
        }));
    } else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "wrong password.".to_string(),
        ));
    }
}

/**
 * 验证一下是否是我们签发的token
 */
#[instrument]
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

pub fn map_ok_result<T>(r: T) -> axum::Json<T> {
    axum::Json(r)
}

pub fn map_consult_error(err: reqwest::Error) -> (StatusCode, String) {
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        "consul error.".to_string(),
    );
}
