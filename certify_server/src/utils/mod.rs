use axum::Error;
use validator::{Validate, ValidationErrors};


pub mod encryption;
pub mod jwt;

pub fn validate_payload<T: Validate>(payload: &T) -> Result<(),ValidationErrors> {
    Ok(payload.validate()?)
}
