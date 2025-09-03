use auth::SERVICE_NAME;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT};
use reqwest::redirect::Policy;
use std::collections::HashMap;
use std::marker::PhantomData;
use tonic::async_trait;
use uuid::Uuid;

use base64::Engine as _;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use rand::distr::{Alphanumeric, SampleString as _};
use rand::rngs::StdRng;
use rand::{Rng as _, SeedableRng as _};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio_postgres::Row;
use url::Url;

use crate::error::ExchangeCodeErr;
use crate::proto::OauthProvider;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Session {
    pub id: String,
    pub secret_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub user_id: String,
}

impl TryFrom<&Row> for Session {
    type Error = tokio_postgres::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Session {
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
    pub provider_user_id: String,
    pub provider_user_name: Option<String>,
    pub provider_user_email: Option<String>,
    pub access_token: Option<String>,
    pub access_token_expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
    pub user_id: Option<String>,
}

impl TryFrom<&Row> for OAuthAccount {
    type Error = tokio_postgres::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(OAuthAccount {
            id: row.try_get("id")?,
            provider: row.try_get("provider")?,
            provider_user_id: row.try_get("provider_user_id")?,
            provider_user_name: row.try_get("provider_user_name")?,
            provider_user_email: row.try_get("provider_user_email")?,
            access_token: row.try_get("access_token")?,
            access_token_expires_at: row.try_get("access_token_expires_at")?,
            refresh_token: row.try_get("refresh_token")?,
            user_id: row.try_get("user_id")?,
        })
    }
}

/// Trait for generating cryptographically secure random strings.
pub trait RandomValueGeneratorTrait: Send + Sync + 'static {
    /// Generates a cryptographically secure random alphanumeric string.
    fn generate_secure_random_string() -> String {
        let mut rng = StdRng::from_os_rng();
        Alphanumeric.sample_string(&mut rng, 24)
    }

    /// Generates the oauth state/csrf token.
    fn generate_random_base64_encoded_string(num_bytes: usize) -> String {
        let random_bytes: Vec<u8> = (0..num_bytes).map(|_| rand::rng().random()).collect();
        BASE64_URL_SAFE_NO_PAD.encode(&random_bytes)
    }

    /// Generates a random uuid
    fn generate_uuid() -> Uuid {
        Uuid::new_v4()
    }
}

/// The default random value generator.
#[derive(Clone)]
pub struct RandomValueGenerator;

impl RandomValueGeneratorTrait for RandomValueGenerator {}

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

#[derive(Default, Clone)]
pub(crate) struct OAuthHelper<R> {
    _phantom: PhantomData<R>,
}

impl<R: RandomValueGeneratorTrait> OAuthHelper<R> {
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
    #[must_use]
    pub fn create_s256_code_challenge(code_verifier: &str) -> String {
        let digest = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(digest);
        code_challenge
    }

    fn generate_authorization_url(
        auth_endpoint: &str,
        client_id: &str,
        redirect_uri: &str,
        scopes: Vec<&str>,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, url::ParseError> {
        let mut params = vec![
            ("response_type", "code"),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
            ("state", state),
        ];

        if !code_challenge.is_empty() {
            params.push(("code_challenge_method", "S256"));
            params.push(("code_challenge", code_challenge));
        }

        let scopes = scopes.join(" ");
        if !scopes.is_empty() {
            params.push(("scope", scopes.as_str()));
        }

        let authorization_url = Url::parse_with_params(auth_endpoint, &params)?;

        Ok(authorization_url.into())
    }

    async fn validate_authorization_code(
        token_endpoint: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        code: &str,
        code_verifier: &str,
    ) -> Result<OAuth2Token, ExchangeCodeErr> {
        let mut params = HashMap::new();
        params.insert("grant_type".to_string(), "authorization_code");
        params.insert("redirect_uri".to_string(), redirect_uri);
        params.insert("code".to_string(), code);
        if !code_verifier.is_empty() {
            params.insert("code_verifier".to_string(), code_verifier);
        }

        let body_str = serde_urlencoded::to_string(params)?;
        let body_bytes = body_str.as_bytes();

        let client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .map_err(|_| ExchangeCodeErr::BuildHttpClient)?;

        let request = client
            .post(token_endpoint)
            .basic_auth(client_id, Some(client_secret))
            .body(body_bytes.to_vec())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(ACCEPT, "application/json")
            .header(CONTENT_LENGTH, body_bytes.len().to_string());

        let response: OAuth2Token = request.send().await?.json().await?;

        Ok(response)
    }

    async fn verify_oidc_token(
        endpoint: &str,
        id_token: &str,
        client_id: &str,
    ) -> Result<TokenClaims, ExchangeCodeErr> {
        let header = decode_header(id_token)?;
        let kid = header.kid.ok_or(ExchangeCodeErr::MissingKID)?;

        let client = Client::new();
        let jwks = client.get(endpoint).send().await?.json::<Jwks>().await?;

        let jwk = jwks
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or(ExchangeCodeErr::NoMatchingJWKS)?;

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[client_id.to_string()]);

        let token_data = decode::<TokenClaims>(id_token, &decoding_key, &validation)?;

        Ok(token_data.claims)
    }
}
#[derive(Debug, Deserialize)]
pub struct OAuth2Token {
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

#[async_trait]
pub trait OAuthProvider: Send + Sync {
    fn generate_authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, url::ParseError>;

    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<OAuthAccount, ExchangeCodeErr>;
}

#[derive(Clone, Default)]
pub(crate) struct GoogleOAuth<R> {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    _phantom: PhantomData<R>,
}

impl<R: RandomValueGeneratorTrait> GoogleOAuth<R> {
    /// Creates a new [Google] oauth client.
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<R> OAuthProvider for GoogleOAuth<R>
where
    R: RandomValueGeneratorTrait,
{
    fn generate_authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, url::ParseError> {
        const AUTH_ENDPOINT: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";

        let authorization_url = OAuthHelper::<R>::generate_authorization_url(
            AUTH_ENDPOINT,
            &self.client_id,
            &self.redirect_uri,
            vec!["openid", "profile", "email"],
            state,
            code_challenge,
        )?;
        Ok(authorization_url)
    }

    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<OAuthAccount, ExchangeCodeErr> {
        const JWKS_CERTS_ENDPOINT: &'static str = "https://www.googleapis.com/oauth2/v3/certs";
        const TOKEN_ENDPOINT: &'static str = "https://oauth2.googleapis.com/token";

        let token = OAuthHelper::<R>::validate_authorization_code(
            TOKEN_ENDPOINT,
            &self.client_id,
            &self.client_secret,
            &self.redirect_uri,
            code,
            code_verifier,
        )
        .await?;

        let Some(id_token) = token.id_token else {
            return Err(ExchangeCodeErr::MissingIDToken.into());
        };

        let claims =
            OAuthHelper::<R>::verify_oidc_token(JWKS_CERTS_ENDPOINT, &id_token, &self.client_id)
                .await?;

        Ok(OAuthAccount {
            id: R::generate_uuid().to_string(),
            provider: OauthProvider::Google.into(),
            provider_user_id: claims.sub,
            provider_user_name: Some(claims.name),
            provider_user_email: Some(claims.email),
            ..Default::default()
        })
    }
}

#[derive(Clone, Default)]
pub(crate) struct GithubOAuth<R> {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    _phantom: PhantomData<R>,
}

impl<R: RandomValueGeneratorTrait> GithubOAuth<R> {
    pub(crate) fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<R> OAuthProvider for GithubOAuth<R>
where
    R: RandomValueGeneratorTrait,
{
    fn generate_authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, url::ParseError> {
        const AUTH_ENDPOINT: &'static str = "https://github.com/login/oauth/authorize";

        let authorization_url = OAuthHelper::<R>::generate_authorization_url(
            AUTH_ENDPOINT,
            &self.client_id,
            &self.redirect_uri,
            vec!["user", "user:email"],
            state,
            code_challenge,
        )?;
        Ok(authorization_url)
    }

    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<OAuthAccount, ExchangeCodeErr> {
        const GET_USER_ENDPOINT: &'static str = "https://api.github.com/user";
        const LIST_EMAILS_ENDPOINT: &'static str = "https://api.github.com/user/emails";
        const TOKEN_ENDPOINT: &'static str = "https://github.com/login/oauth/access_token";

        #[derive(Debug, Deserialize)]
        pub struct GithubUser {
            pub id: u64,
            pub login: String,
            pub name: Option<String>,
            pub email: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct GitHubEmail {
            email: String,
            primary: bool,
        }

        let token = OAuthHelper::<R>::validate_authorization_code(
            TOKEN_ENDPOINT,
            &self.client_id,
            &self.client_secret,
            &self.redirect_uri,
            code,
            code_verifier,
        )
        .await?;

        let Some(access_token) = token.access_token else {
            return Err(ExchangeCodeErr::MissingAccessToken.into());
        };

        let client = Client::new();

        let response = client
            .get(GET_USER_ENDPOINT)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(USER_AGENT, SERVICE_NAME)
            .send()
            .await?;

        let user: GithubUser = response.json().await?;
        let user_id = user.id.to_string();
        let user_name = user.name.unwrap_or(user.login);

        if let Some(user_email) = user.email {
            return Ok(OAuthAccount {
                id: R::generate_uuid().to_string(),
                provider: OauthProvider::Github.into(),
                provider_user_id: user_id,
                provider_user_name: Some(user_name),
                provider_user_email: Some(user_email),
                ..Default::default()
            });
        }

        let response = client
            .get(LIST_EMAILS_ENDPOINT)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(USER_AGENT, SERVICE_NAME)
            .send()
            .await?;

        let emails: Vec<GitHubEmail> = response.json().await?;

        let Some(user_email) = emails.iter().find(|e| e.primary).map(|e| e.email.clone()) else {
            return Err(ExchangeCodeErr::NoEmailFound);
        };

        Ok(OAuthAccount {
            id: R::generate_uuid().to_string(),
            provider: OauthProvider::Github.into(),
            provider_user_id: user_id,
            provider_user_name: Some(user_name),
            provider_user_email: Some(user_email),
            ..Default::default()
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

#[cfg(test)]
pub(crate) mod tests {
    use chrono::TimeZone;
    use tonic::{Code, Response, Status};

    use crate::utils::OAuthAccount;

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

    #[derive(Default, Clone)]
    pub(crate) struct MockRandomValueGenerator;

    impl RandomValueGeneratorTrait for MockRandomValueGenerator {
        fn generate_secure_random_string() -> String {
            "secret".to_string()
        }

        fn generate_random_base64_encoded_string(_: usize) -> String {
            "secret-encoded".to_string()
        }

        fn generate_uuid() -> Uuid {
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
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

    pub(crate) fn fixture_oauth_account<F>(mut func: F) -> OAuthAccount
    where
        F: FnMut(&mut OAuthAccount),
    {
        let mut token = OAuthAccount {
            id: "oauth-id".to_string(),
            provider_user_id: "provider-user-id".to_string(),
            provider_user_name: Some("provider-user-name".to_string()),
            provider_user_email: Some("provider-user-email".to_string()),
            provider: 0,
            access_token: Some("access-token".to_string()),
            access_token_expires_at: None,
            refresh_token: None,
            user_id: None,
        };
        func(&mut token);
        token
    }
}
