use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::{
    Client,
    header::{ACCEPT, CONTENT_LENGTH, CONTENT_TYPE},
    redirect::Policy,
};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, marker::PhantomData};
use tonic::async_trait;
use tracing::info;
use url::Url;

use crate::{
    error::Error,
    models::{Jwks, OidcTokenClaims},
    random::RandomSource,
};

/// Generic OAuth 2.0 helper that abstracts PKCE, authorization URL creation, and token validation.
#[derive(Default, Clone)]
pub struct OAuth<R> {
    _phantom: PhantomData<R>,
}

impl<R: RandomSource> OAuth<R> {
    /// Creates a new `OAuth` helper for a given random source.
    #[inline]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Generates the OAuth `state` (CSRF protection token).
    #[must_use]
    pub fn generate_state() -> String {
        R::base64_url(32)
    }

    /// Generates a PKCE `code_verifier` string.
    #[must_use]
    pub fn generate_code_verifier() -> String {
        R::base64_url(32)
    }

    /// Creates an S256 code challenge from a given PKCE code verifier.
    #[must_use]
    pub fn create_s256_code_challenge(code_verifier: &str) -> String {
        let digest = Sha256::digest(code_verifier.as_bytes());
        BASE64_URL_SAFE_NO_PAD.encode(digest)
    }

    /// Constructs the OAuth 2.0 authorization URL.
    pub fn generate_authorization_url(
        auth_endpoint: &str,
        client_id: &str,
        redirect_uri: &str,
        scopes: Vec<&str>,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, Error> {
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

        let url = Url::parse_with_params(auth_endpoint, &params)?;
        Ok(url.into())
    }

    /// Exchanges an authorization code for a token response.
    pub async fn validate_authorization_code<T: DeserializeOwned>(
        token_endpoint: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        code: &str,
        code_verifier: &str,
    ) -> Result<T, Error> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("grant_type".into(), "authorization_code".into());
        params.insert("redirect_uri".into(), redirect_uri.into());
        params.insert("code".into(), code.into());
        if !code_verifier.is_empty() {
            params.insert("code_verifier".into(), code_verifier.into());
        }

        let body = serde_urlencoded::to_string(&params)?;
        let client = Client::builder()
            .redirect(Policy::none())
            .build()
            .map_err(|_| Error::BuildHttpClient)?;

        let response = client
            .post(token_endpoint)
            .basic_auth(client_id, Some(client_secret))
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(ACCEPT, "application/json")
            .header(CONTENT_LENGTH, body.len().to_string())
            .body(body)
            .send()
            .await?
            .json::<T>()
            .await?;

        Ok(response)
    }

    /// Verifies an OpenID Connect ID token using the provider's JWKS.
    pub async fn verify_oidc_token(
        endpoint: &str,
        id_token: &str,
        client_id: &str,
    ) -> Result<OidcTokenClaims, Error> {
        let header = decode_header(id_token)?;
        let kid = header.kid.ok_or(Error::MissingKID)?;

        let jwks = Client::new()
            .get(endpoint)
            .send()
            .await?
            .json::<Jwks>()
            .await?;

        let jwk = jwks
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or(Error::NoMatchingJWKS)?;

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[client_id.to_string()]);

        let token_data = decode::<OidcTokenClaims>(id_token, &decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}

/// Generic trait implemented by all OAuth 2.0 providers (e.g., Polar, Strava, etc.).
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// The type returned when the OAuth flow completes successfully (e.g., an account or token set).
    type Account: Send + Sync + 'static;

    /// The provider’s error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Generates a provider-specific authorization URL to start the OAuth login flow.
    fn generate_authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, Self::Error>;

    /// Exchanges an authorization code for tokens and account information.
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<Self::Account, Self::Error>;
}
