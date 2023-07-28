use validator::{Validate, ValidationErrors};

use http::StatusCode;
use std::error::Error;

pub fn validate_payload<T: Validate>(payload: &T) -> Result<(), ValidationErrors> {
    Ok(payload.validate()?)
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

pub fn internal_error_dyn(err: Box<dyn Error>) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }
}
