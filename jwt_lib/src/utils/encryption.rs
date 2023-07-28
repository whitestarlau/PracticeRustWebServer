use std::error::Error;

use bcrypt::{BcryptError, DEFAULT_COST};


// consume password value to make it unusable
pub async fn hash_password(password: String) -> Result<String, Box<dyn Error>> {
    let (send, recv) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let result = bcrypt::hash(password, DEFAULT_COST);
        let _ = send.send(result);
    });
    Ok(recv.await??)
}

pub async fn verify_password(password: String, hash: String) -> Result<bool, Box<dyn Error>> {
    let (send, recv) = tokio::sync::oneshot::channel();
    rayon::spawn(move || {
        let result = bcrypt::verify(password, &hash);
        let _ = send.send(result);
    });
    Ok(recv.await??)
}
