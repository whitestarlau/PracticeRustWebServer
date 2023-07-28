use axum::{async_trait, extract::FromRequest};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

use axum::{extract::FromRequestParts, http::HeaderMap};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct User {
    #[serde(skip_serializing)]
    pub id: Uuid,
    pub user_email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub create_time: i64,
}

#[derive(Deserialize, Validate, Serialize, Debug, Clone)]
pub struct SignUser {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SignUserResp {
    pub uid: Uuid,
    pub token: TokenPayload,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenPayload {
    pub access_token: String,
    pub token_type: String,
}