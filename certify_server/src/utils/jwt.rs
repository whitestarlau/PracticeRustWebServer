use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, TypedHeader},
    http::request::Parts, RequestPartsExt,
};
use chrono::{Duration, Utc};
use headers::{HeaderMap, Authorization, authorization::Bearer};
use jsonwebtoken::{errors::Error, DecodingKey, EncodingKey, Header, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{config::env, models::error::internal_error};

// use crate::{config::env::{JWT_SECRET, self}};

/**
 * jwt 中claims中的部分
 */
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

pub fn sign(id: Uuid) -> Result<String, Error> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(id),
        &EncodingKey::from_secret(env::JWT_SECRET.as_bytes()),
    )?)
}

pub fn verify(token: &str) -> Result<Claims, Error> {
    Ok(jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(env::JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data: jsonwebtoken::TokenData<Claims>| data.claims)?)
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(internal_error)?;
        // Decode the user data
        let claims = verify(bearer.token()).map_err(internal_error)?;

        Ok(claims)
    }
}