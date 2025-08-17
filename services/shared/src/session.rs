use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Duration;
use http::HeaderValue;

/// The session token cookie key.
pub const SESSION_TOKEN_COOKIE_KEY: &'static str = "session_token";

/// The session token expiry duration.
pub const SESSION_TOKEN_EXPIRY_DURATION: Duration = Duration::days(7);

/// Represents session state.
#[derive(Clone, Debug)]
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

/// Creates a new session token cookie.
pub fn create_session_token_cookie<T>(token: T) -> Cookie<'static>
where
    T: Into<String>,
{
    Cookie::build((SESSION_TOKEN_COOKIE_KEY, token.into()))
        .http_only(true)
        .secure(false) // TODO: Enable on production
        .max_age(time::Duration::seconds(
            SESSION_TOKEN_EXPIRY_DURATION.num_seconds(),
        ))
        .path("/")
        .same_site(SameSite::Lax)
        .build()
}

// Sets the session token cookie in the response headers.
pub(crate) fn set_session_token_cookie<B, T>(response: &mut http::Response<B>, token: T)
where
    T: Into<String>,
{
    use http::header::{HeaderValue, SET_COOKIE};
    let cookie = create_session_token_cookie(token);
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );
}

/// Extracts the session token cookie from the response headers.
pub(crate) fn extract_session_token_cookie(header_value: &HeaderValue) -> Option<String> {
    let Ok(cookie_str) = header_value.to_str() else {
        return None;
    };

    for cookie in cookie_str.split(';') {
        let cookie = cookie.trim();
        let mut parts = cookie.splitn(2, '=');
        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
            if key == SESSION_TOKEN_COOKIE_KEY {
                return Some(value.to_string());
            }
        }
    }

    None
}
