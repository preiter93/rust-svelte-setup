use chrono::Duration;

/// The session token cookie key.
pub const SESSION_TOKEN_COOKIE_KEY: &'static str = "session_token";

/// The session token expiry duration.
pub const SESSION_TOKEN_EXPIRY_DURATION: Duration = Duration::days(7);

/// Represents session state.
#[derive(Clone, Debug, Default)]
pub struct SessionState {
    /// The id of the authenticated used.
    pub user_id: String,
}

impl SessionState {
    /// Creates a new `SessionState`.
    pub fn new(user_id: String) -> Self {
        Self { user_id }
    }
}
