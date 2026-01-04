pub(crate) struct OauthConfig {
    pub(super) google_client_id: String,
    pub(super) google_client_secret: String,
    pub(super) google_redirect_uri: String,
    pub(super) github_client_id: String,
    pub(super) github_client_secret: String,
    pub(super) github_redirect_uri: String,
}

impl OauthConfig {
    fn must_get_env(key: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
    }

    pub(crate) fn from_env() -> Self {
        Self {
            google_client_id: Self::must_get_env("GOOGLE_CLIENT_ID"),
            google_client_secret: Self::must_get_env("GOOGLE_CLIENT_SECRET"),
            google_redirect_uri: Self::must_get_env("GOOGLE_REDIRECT_URI"),
            github_client_id: Self::must_get_env("GITHUB_CLIENT_ID"),
            github_client_secret: Self::must_get_env("GITHUB_CLIENT_SECRET"),
            github_redirect_uri: Self::must_get_env("GITHUB_REDIRECT_URI"),
        }
    }
}
