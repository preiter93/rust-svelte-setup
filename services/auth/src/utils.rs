use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use std::collections::HashMap;
use std::error::Error;

use base64::Engine as _;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use rand::distr::{Alphanumeric, SampleString as _};

use rand::rngs::StdRng;
use rand::{Rng as _, SeedableRng as _};
use reqwest::{Client, ClientBuilder, RequestBuilder};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub secret_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub user_id: String,
}

/// Generates cryptographically secure random strings.
///
/// [`Documentation`]: https://lucia-auth.com/sessions/basic
#[must_use]
pub fn generate_secure_random_string() -> String {
    let mut rng = StdRng::from_os_rng();

    Alphanumeric.sample_string(&mut rng, 24)
}

/// Generates the oauth state/csrf token.
#[must_use]
pub fn generate_random_base64_encoded_string(num_bytes: usize) -> String {
    let random_bytes: Vec<u8> = (0..num_bytes).map(|_| rand::rng().random()).collect();
    BASE64_URL_SAFE_NO_PAD.encode(&random_bytes[..])
}

pub(crate) struct OAuth;
impl OAuth {
    /// Generates the oauth state/csrf token.
    #[must_use]
    pub fn generate_state() -> String {
        generate_random_base64_encoded_string(32)
    }

    /// Generates the oauth code verifier.
    #[must_use]
    pub fn generate_code_verifier() -> String {
        generate_random_base64_encoded_string(32)
    }

    /// Creates a S256 code challenge.
    fn create_s256_code_challenge(code_verifier: &str) -> String {
        let digest = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(digest);
        code_challenge
    }

    /// Creates an oauth2 request.
    pub(crate) fn create_oauth2_request(
        endpoint: &str,
        body: HashMap<String, String>,
    ) -> Result<RequestBuilder, Box<dyn Error>> {
        let body_str = serde_urlencoded::to_string(body)?;
        let body_bytes = body_str.as_bytes();

        let client = Client::new();

        let req = client
            .post(endpoint)
            .body(body_bytes.to_vec())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "application/json")
            // .header("User-Agent", "arctic")
            .header("Content-Length", body_bytes.len().to_string());

        Ok(req)
    }
}
#[derive(Debug, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub scope: String,
    pub token_type: String,
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
pub struct Oauth2TokenClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: String,
    pub exp: usize,
}

#[derive(Clone)]
pub(crate) struct GoogleOAuth {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}
impl GoogleOAuth {}

impl GoogleOAuth {
    const AUTHORIZATION_ENDPOINT: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
    const TOKEN_ENDPOINT: &'static str = "https://oauth2.googleapis.com/token";
    const TOKEN_REVOCATION_ENDPOINT: &'static str = "https://oauth2.googleapis.com/revoke";
    const JWKS_ENDPOINT: &'static str = "https://www.googleapis.com/oauth2/v3/certs";

    /// Creates a new [Google] oauth client.
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
        }
    }

    /// Generates the google authorization url.
    #[must_use]
    pub fn generate_authorization_url(
        &self,
        state: &str,
        code_verifier: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let scopes = ["openid", "profile", "email"];
        let code_challenge = OAuth::create_s256_code_challenge(code_verifier);
        let params = [
            ("response_type", "code"),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
            ("state", state),
            ("code_challenge_method", "S256"),
            ("code_challenge", &code_challenge),
            ("scope", &scopes.join(" ")),
        ];
        Ok(Url::parse_with_params(Self::AUTHORIZATION_ENDPOINT, &params)?.into())
    }

    // Validates the authorization code.
    pub async fn validate_authorization_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<OAuth2TokenResponse, Box<dyn Error>> {
        let token_endpoint = Self::TOKEN_ENDPOINT;

        let mut params = HashMap::new();
        params.insert("grant_type".to_string(), "authorization_code".to_string());
        params.insert("redirect_uri".to_string(), self.redirect_uri.to_owned());
        params.insert("code".to_string(), code.to_owned());
        params.insert("code_verifier".to_string(), code_verifier.to_owned());

        let request = OAuth::create_oauth2_request(token_endpoint, params)?;
        let request = request
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .build()?;

        let client = ClientBuilder::new().build()?;
        let response = client.execute(request).await?;

        Ok(response.json().await?)
    }

    // Decodes the id token and returns the token claims.
    pub async fn decode_id_token(&self, token: &str) -> Result<Oauth2TokenClaims, Box<dyn Error>> {
        let header = decode_header(token)?;
        let kid = header.kid.ok_or("missing 'kid' in token header")?;

        let jwks = get_jwks(Self::JWKS_ENDPOINT).await?;

        let jwk = jwks
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or("no matching JWK found for token kid")?;

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.client_id.clone()]);

        let token_data = decode::<Oauth2TokenClaims>(token, &decoding_key, &validation)?;

        Ok(token_data.claims)
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
