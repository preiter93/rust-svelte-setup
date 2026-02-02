use uuid::Uuid;

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio_postgres::Row;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct DBSession {
    pub id: String,
    pub secret_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub user_id: Uuid,
}

impl TryFrom<&Row> for DBSession {
    type Error = tokio_postgres::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(DBSession {
            id: row.try_get("id")?,
            secret_hash: row.try_get("secret_hash")?,
            created_at: row.try_get("created_at")?,
            expires_at: row.try_get("expires_at")?,
            user_id: row.try_get("user_id")?,
        })
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct OAuthAccount {
    pub id: String,
    pub provider: i32,
    pub external_user_id: String,
    pub external_user_name: Option<String>,
    pub external_user_email: Option<String>,
    pub access_token: Option<String>,
    pub access_token_expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
    pub user_id: Option<Uuid>,
}

impl TryFrom<&Row> for OAuthAccount {
    type Error = tokio_postgres::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(OAuthAccount {
            id: row.try_get("id")?,
            provider: row.try_get("provider")?,
            external_user_id: row.try_get("external_user_id")?,
            external_user_name: row.try_get("external_user_name")?,
            external_user_email: row.try_get("external_user_email")?,
            access_token: row.try_get("access_token")?,
            access_token_expires_at: row.try_get("access_token_expires_at")?,
            refresh_token: row.try_get("refresh_token")?,
            user_id: row.try_get("user_id")?,
        })
    }
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

/// Represents the claims in an OIDC ID token.
#[derive(Debug, Deserialize)]
struct TokenClaims {
    /// The subject identifier for the user.
    sub: String,
    /// The user's email address.
    email: String,
    /// The user's name.
    name: String,
}

/// Represents a JSON Web Key Set (JWKS).
#[derive(Debug, Deserialize)]
struct Jwks {
    /// The list of JSON Web Keys.
    keys: Vec<Jwk>,
}

/// Represents a single JSON Web Key (JWK).
#[derive(Debug, Deserialize)]
struct Jwk {
    /// Key ID
    kid: String,
    /// RSA modulus
    n: String,
    /// RSA exponent
    e: String,
    /// Key type (e.g., "RSA")
    kty: String,
    /// Algorithm (e.g., "RS256")
    alg: String,
}

/// Fetches the JSON web key set (JWKS) from the given endpoint.
async fn get_jwks(endpoint: &str) -> Result<Jwks, Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client.get(endpoint).send().await?.json::<Jwks>().await?;
    Ok(res)
}
