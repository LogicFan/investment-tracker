pub mod delete;
pub mod exist;
pub mod login;
pub mod register;
pub mod rotate;
pub mod update;

use hmac::{Hmac, Mac};
use jwt::{Header, Token, VerifyWithKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::sync::LazyLock;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use uuid::Uuid;

static PRIVATE_KEY: LazyLock<Hmac<Sha256>> = LazyLock::new(|| {
    let mut rng = rand::thread_rng();
    let mut bytes = [0_u8; 32];
    rng.fill_bytes(&mut bytes);

    // test code, use 0 as hmac key
    let bytes = [0_u8; 32];

    Hmac::new_from_slice(&bytes).expect("fail to generate HMAC key.")
});

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: Uuid,
    iat: u64,
    exp: u64,
}

pub trait AsUser {
    fn user_id(&self) -> Result<Option<Uuid>, SystemTimeError>;
}

impl AsUser for str {
    fn user_id(&self) -> Result<Option<Uuid>, SystemTimeError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        match self.verify_with_key(&*PRIVATE_KEY).ok() {
            Some(token) => {
                let token: Token<Header, Claims, _> = token;
                let claims: &Claims = token.claims();
                if claims.exp > now {
                    return Ok(Some(claims.iss));
                }
            }
            _ => (),
        }

        Ok(None)
    }
}

pub fn authenticate(token: &str) -> Result<Option<Uuid>, SystemTimeError> {
    token.user_id()
}
