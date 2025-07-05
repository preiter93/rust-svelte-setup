use chrono::{DateTime, Utc};
use rand::distr::{Alphanumeric, SampleString as _};

use rand::SeedableRng as _;
use rand::rngs::StdRng;
use sha2::{Digest, Sha256};

#[derive(Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub secret_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

impl From<Session> for crate::proto::Session {
    fn from(val: Session) -> crate::proto::Session {
        crate::proto::Session {
            id: val.id,
            created_at: Some(val.created_at.into()),
        }
    }
}

/// Generates cryptographically secure random strings.
///
/// [`Documentation`]: https://lucia-auth.com/sessions/basic
#[must_use]
pub fn generate_secure_random_string() -> String {
    let mut rng = StdRng::from_os_rng();

    Alphanumeric.sample_string(&mut rng, 24)
}

/// Hashes a secret using SHA-256. While SHA-256 is unsuitable
/// for user passwords, because the secret has 120 bits of entropy
/// an offline brute-force attack is impossible.
///
/// [`Documentation`]: https://lucia-auth.com/sessions/basic
#[must_use]
pub fn hash_secret(secret: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(secret);
    hasher.finalize().to_vec()
}

// /// Converts `chrono::DateTime<Utc>` to `prost_types::Timestamp`
// #[must_use]
// pub fn datetime_to_prost(ts: DateTime<Utc>) -> Timestamp {
//     Timestamp {
//         seconds: ts.timestamp(),
//         nanos: ts.timestamp_subsec_nanos() as i32,
//     }
// }
//
// /// Converts `prost_types::Timestamp` to `chrono::DateTime<Utc>`
// ///
// /// # Errors
// /// - invalid timestamp
// pub fn prost_to_datetime(ts: &Timestamp) -> Result<DateTime<Utc>, TimestampError> {
//     Utc.timestamp_opt(ts.seconds, ts.nanos as u32)
//         .single()
//         .ok_or(TimestampError)
// }
//
// /// An error indicating that a `prost_types::Timestamp` value is invalid.
// #[derive(Debug, Error)]
// #[error("invalid timestamp")]
// pub struct TimestampError;

/// Compares two byte slices for equality in constant time to prevent timing attacks.
#[must_use]
pub fn constant_time_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut c = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        c |= x ^ y;
    }
    c == 0
}
