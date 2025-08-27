use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use std::collections::HashMap;
use std::error::Error;
use std::marker::PhantomData;

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

#[derive(Clone, PartialEq, Debug)]
pub struct Session {
    pub id: String,
    pub secret_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub user_id: String,
}

/// Trait for generating cryptographically secure random strings.
pub trait RandomStringGenerator: Send + Sync + 'static {
    /// Generates a cryptographically secure random alphanumeric string.
    fn generate_secure_random_string() -> String;

    /// Generates the oauth state/csrf token.
    fn generate_random_base64_encoded_string(num_bytes: usize) -> String;
}

/// Trait for providing the current UTC time.
pub trait Now: Send + Sync + 'static {
    /// Returns the current UTC time.
    fn now() -> chrono::DateTime<chrono::Utc>;
}

/// Implementation of `UTC` that returns the actual current time.
pub struct SystemNow;

impl Now for SystemNow {
    fn now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }
}

/// The default random string generator using [`StdRng`].
pub struct StdRandomStringGenerator;

impl RandomStringGenerator for StdRandomStringGenerator {
    /// Generates cryptographically secure random strings.
    ///
    /// [`Documentation`]: https://lucia-auth.com/sessions/basic
    fn generate_secure_random_string() -> String {
        let mut rng = StdRng::from_os_rng();
        Alphanumeric.sample_string(&mut rng, 24)
    }

    /// Generates the oauth state/csrf token.
    fn generate_random_base64_encoded_string(num_bytes: usize) -> String {
        let random_bytes: Vec<u8> = (0..num_bytes).map(|_| rand::rng().random()).collect();
        BASE64_URL_SAFE_NO_PAD.encode(&random_bytes)
    }
}

#[derive(Default, Clone)]
pub(crate) struct OAuth<R> {
    _phantom: PhantomData<R>,
}

impl<R: RandomStringGenerator> OAuth<R> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Generates the oauth state/csrf token.
    #[must_use]
    pub fn generate_state() -> String {
        R::generate_random_base64_encoded_string(32)
    }

    /// Generates the oauth code verifier.
    #[must_use]
    pub fn generate_code_verifier() -> String {
        R::generate_random_base64_encoded_string(32)
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
            .header("User-Agent", "rust-svelte-setup")
            .header("Content-Length", body_bytes.len().to_string());

        Ok(req)
    }
}
#[derive(Debug, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: Option<String>,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
    pub token_type: Option<String>,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Oauth2TokenClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub exp: usize,
}

#[derive(Clone, Default)]
pub(crate) struct GoogleOAuth<R> {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    _phantom: PhantomData<R>,
}

impl<R: RandomStringGenerator> GoogleOAuth<R> {
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
            _phantom: PhantomData,
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
        let code_challenge = OAuth::<R>::create_s256_code_challenge(code_verifier);
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

        let request = OAuth::<R>::create_oauth2_request(token_endpoint, params)?;
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

#[derive(Clone, Default)]
pub(crate) struct GithubOAuth<R> {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    _phantom: PhantomData<R>,
}

impl<R: RandomStringGenerator> GithubOAuth<R> {
    const AUTHORIZATION_ENDPOINT: &'static str = "https://github.com/login/oauth/authorize";
    const TOKEN_ENDPOINT: &'static str = "https://github.com/login/oauth/access_token";
    const GET_USER_ENDPOINT: &'static str = "https://api.github.com/user";
    const LIST_USER_EMAILS_ENDPOINT: &'static str = "https://api.github.com/user/emails";

    pub(crate) fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            _phantom: PhantomData,
        }
    }

    /// Generates the authorization url.
    #[must_use]
    pub fn generate_authorization_url(
        &self,
        state: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let scopes = ["user", "user:email"];
        let params = [
            ("response_type", "code"),
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
            ("state", state),
            ("scope", &scopes.join(" ")),
        ];

        Ok(Url::parse_with_params(Self::AUTHORIZATION_ENDPOINT, &params)?.into())
    }

    // Validates the authorization code.
    pub async fn validate_authorization_code(
        &self,
        code: &str,
    ) -> Result<OAuth2TokenResponse, Box<dyn Error>> {
        let token_endpoint = Self::TOKEN_ENDPOINT;

        let mut params = HashMap::new();
        params.insert("client_id".to_string(), self.client_id.to_owned());
        params.insert("client_secret".to_string(), self.client_secret.to_owned());
        params.insert("grant_type".to_string(), "authorization_code".to_string());
        params.insert("redirect_uri".to_string(), self.redirect_uri.to_owned());
        params.insert("code".to_string(), code.to_owned());

        let request = OAuth::<R>::create_oauth2_request(token_endpoint, params)?;
        let request = request
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .build()?;

        let client = ClientBuilder::new().build()?;
        let response = client.execute(request).await?;

        Ok(response.json().await?)
    }

    pub async fn get_user(&self, access_token: &str) -> Result<GithubUser, Box<dyn Error>> {
        let client = Client::new();
        let response = client
            .get(Self::GET_USER_ENDPOINT)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "rust-svelte-setup")
            .send()
            .await?;

        Ok(response.json().await?)
    }

    pub async fn get_primary_email(&self, access_token: &str) -> Result<String, Box<dyn Error>> {
        #[derive(Debug, Deserialize)]
        struct GitHubEmail {
            email: String,
            primary: bool,
        }

        let client = Client::new();
        let response = client
            .get(Self::LIST_USER_EMAILS_ENDPOINT)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "rust-svelte-setup")
            .send()
            .await?;

        let emails: Vec<GitHubEmail> = response.json().await?;

        if let Some(primary) = emails.iter().find(|e| e.primary) {
            Ok(primary.email.to_string())
        } else {
            Err(Box::from("no primary email found for this user"))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
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

#[cfg(test)]
pub(crate) mod tests {
    use chrono::TimeZone;
    use tonic::{Code, Response, Status};

    use super::*;

    pub(crate) fn fixture_token() -> String {
        "secret.secret".to_string()
    }

    pub(crate) fn fixture_session<F>(mut func: F) -> Session
    where
        F: FnMut(&mut Session),
    {
        let mut session = Session {
            id: "session-id".to_string(),
            secret_hash: hash_secret("secret"),
            created_at: chrono::Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap(),
            expires_at: chrono::Utc.with_ymd_and_hms(2020, 1, 8, 0, 0, 0).unwrap(),
            user_id: "user-id".to_string(),
        };
        func(&mut session);
        session
    }

    #[derive(Default)]
    pub(crate) struct MockRandomStringGenerator;

    impl RandomStringGenerator for MockRandomStringGenerator {
        fn generate_secure_random_string() -> String {
            "secret".to_string()
        }

        fn generate_random_base64_encoded_string(_: usize) -> String {
            "secret-encoded".to_string()
        }
    }

    pub struct MockNow;

    impl Now for MockNow {
        fn now() -> chrono::DateTime<chrono::Utc> {
            chrono::Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap()
        }
    }

    pub(crate) fn assert_response<T: PartialEq + std::fmt::Debug>(
        got: Result<Response<T>, Status>,
        want: Result<T, Code>,
    ) {
        match (got, want) {
            (Ok(got), Ok(want)) => assert_eq!(got.into_inner(), want),
            (Err(got), Err(want)) => assert_eq!(got.code(), want),
            (Ok(got), Err(want)) => panic!("left: {got:?}\nright: {want}"),
            (Err(got), Ok(want)) => panic!("left: {got}\nright: {want:?}"),
        }
    }
}
