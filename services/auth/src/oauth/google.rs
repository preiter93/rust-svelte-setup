use std::marker::PhantomData;

use oauth::{OAuth, OAuthProvider, RandomSource, SecureRandom};
use tonic::async_trait;

use crate::{
    oauth::{error::Error, models::OAuth2Token},
    proto::OauthProvider,
    utils::OAuthAccount,
};

/// Google OAuth 2.0 endpoints.
const GOOGLE_JWKS_CERTS_ENDPOINT: &str = "https://www.googleapis.com/oauth2/v3/certs";
const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_AUTH_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";

/// OAuth 2.0 client for Google sign-in.
///
/// Handles authorization URL generation, token exchange, and ID token verification.
#[derive(Clone, Default)]
pub(crate) struct GoogleOAuth<R> {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    _phantom: PhantomData<R>,
}

impl GoogleOAuth<SecureRandom> {
    /// Creates a new [`GoogleOAuth`] client instance.
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
    R: RandomSource,
{
    type Account = OAuthAccount;
    type Error = Error;

    /// Generates the Google OAuth 2.0 authorization URL to begin the login flow.
    fn generate_authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, Self::Error> {
        let authorization_url = OAuth::<R>::generate_authorization_url(
            GOOGLE_AUTH_ENDPOINT,
            &self.client_id,
            &self.redirect_uri,
            vec!["openid", "profile", "email"],
            state,
            code_challenge,
        )?;
        Ok(authorization_url)
    }

    /// Exchanges the authorization code for tokens, verifies the ID token,
    /// and returns an [`OAuthAccount`] with Google user info.
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<Self::Account, Self::Error> {
        // Exchange authorization code for token
        let token = OAuth::<R>::validate_authorization_code::<OAuth2Token>(
            GOOGLE_TOKEN_ENDPOINT,
            &self.client_id,
            &self.client_secret,
            &self.redirect_uri,
            code,
            code_verifier,
        )
        .await?;

        let id_token = token.id_token.ok_or(Self::Error::MissingIDToken)?;

        // Verify ID token and extract OIDC claims
        let claims =
            OAuth::<R>::verify_oidc_token(GOOGLE_JWKS_CERTS_ENDPOINT, &id_token, &self.client_id)
                .await?;

        Ok(OAuthAccount {
            id: R::uuid().to_string(),
            provider: OauthProvider::Google.into(),
            external_user_id: claims.sub,
            external_user_name: claims.name,
            external_user_email: claims.email,
            ..Default::default()
        })
    }
}
