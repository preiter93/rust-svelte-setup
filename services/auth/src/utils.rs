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
