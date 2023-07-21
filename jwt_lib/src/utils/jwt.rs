use std::env;

use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, TypedHeader},
    http::request::Parts,
    RequestPartsExt,
};
use chrono::{Duration, Utc};
use headers::{authorization::Bearer, Authorization, HeaderMap};
use http::StatusCode;
use jsonwebtoken::{errors::Error, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

lazy_static! {
    pub static ref JWT_SECRET: String = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
}

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

pub fn sign(id: Uuid, encodingKey: &EncodingKey) -> Result<String, Error> {
    Ok(jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(id),
        encodingKey,
    )?)
}

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
