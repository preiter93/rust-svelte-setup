use oauth::{RandomSource, SecureRandom};
use std::marker::PhantomData;

use oauth::{OAuth, OAuthProvider};
use reqwest::{
    Client,
    header::{AUTHORIZATION, USER_AGENT},
};
use serde::Deserialize;
use tonic::async_trait;

use crate::{
    SERVICE_NAME,
    oauth::{error::Error, models::OAuth2Token},
    proto::OauthProvider,
    utils::OAuthAccount,
};

/// GitHub OAuth 2.0 endpoints.
const GITHUB_AUTH_ENDPOINT: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_ENDPOINT: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_ENDPOINT: &str = "https://api.github.com/user";
const GITHUB_EMAILS_ENDPOINT: &str = "https://api.github.com/user/emails";

/// GitHub OAuth 2.0 client.
///
/// Handles authorization URL generation, token exchange, and user data fetching.
#[derive(Clone, Default)]
pub(crate) struct GithubOAuth<R> {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    _phantom: PhantomData<R>,
}

impl GithubOAuth<SecureRandom> {
    /// Creates a new [`GithubOAuth`] client instance.
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
    R: RandomSource,
{
    type Account = OAuthAccount;
    type Error = Error;

    /// Generates the GitHub OAuth 2.0 authorization URL.
    fn generate_authorization_url(
        &self,
        state: &str,
        code_challenge: &str,
    ) -> Result<String, Self::Error> {
        let authorizaton_url = OAuth::<R>::generate_authorization_url(
            GITHUB_AUTH_ENDPOINT,
            &self.client_id,
            &self.redirect_uri,
            vec!["user", "user:email"],
            state,
            code_challenge,
        )?;

        Ok(authorizaton_url)
    }

    /// Exchanges the authorization code for an access token,
    /// then retrieves GitHub user info and primary email.
    async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<Self::Account, Self::Error> {
        #[derive(Debug, Deserialize)]
        struct GithubUser {
            id: u64,
            login: String,
            name: Option<String>,
            email: Option<String>,
        }

        #[derive(Debug, Deserialize)]
        struct GithubEmail {
            email: String,
            primary: bool,
        }

        // Exchange authorization code for token
        let token = OAuth::<R>::validate_authorization_code::<OAuth2Token>(
            GITHUB_TOKEN_ENDPOINT,
            &self.client_id,
            &self.client_secret,
            &self.redirect_uri,
            code,
            code_verifier,
        )
        .await?;

        let access_token = token.access_token.ok_or(Self::Error::MissingAccessToken)?;

        let client = Client::new();

        // Fetch GitHub user info
        let user_response = client
            .get(GITHUB_USER_ENDPOINT)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(USER_AGENT, SERVICE_NAME)
            .send()
            .await?;

        let user: GithubUser = user_response.json().await?;
        let user_id = user.id.to_string();
        let user_name = user.name.unwrap_or(user.login);

        // Use email if available directly
        if let Some(user_email) = user.email {
            return Ok(Self::Account {
                id: R::uuid().to_string(),
                provider: OauthProvider::Github.into(),
                external_user_id: user_id,
                external_user_name: Some(user_name),
                external_user_email: Some(user_email),
                ..Default::default()
            });
        }

        // Otherwise, fetch email list
        let email_response = client
            .get(GITHUB_EMAILS_ENDPOINT)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .header(USER_AGENT, SERVICE_NAME)
            .send()
            .await?;

        let emails: Vec<GithubEmail> = email_response.json().await?;

        let user_email = emails
            .iter()
            .find(|e| e.primary)
            .map(|e| e.email.clone())
            .ok_or(Self::Error::MissingEmail)?;

        Ok(Self::Account {
            id: R::uuid().to_string(),
            provider: OauthProvider::Github.into(),
            external_user_id: user_id,
            external_user_name: Some(user_name),
            external_user_email: Some(user_email),
            ..Default::default()
        })
    }
}
