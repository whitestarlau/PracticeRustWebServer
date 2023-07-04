use std::f32::consts::E;

use axum::http::StatusCode;
use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::{
    models::{
        error::internal_error,
        user::{SignUser, User},
    },
    utils::encryption,
};

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
            .map_err(internal_error)?;
    
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
