use crate::session::{SESSION_TOKEN_COOKIE_KEY, SESSION_TOKEN_EXPIRY_DURATION};
use chrono::Duration;
use http::HeaderValue;
use std::fmt;

/// Representation of an HTTP cookie.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cookie {
    /// The cookie's name.
    name: String,

    /// The cookie's value.
    value: String,

    /// The cookie's maximum age, if any.
    max_age: Duration,

    /// The cookie's path domain, if any.
    path: String,

    /// Whether this cookie was marked Secure.
    secure: bool,

    /// Whether this cookie was marked HttpOnly.
    http_only: bool,

    /// The draft `SameSite` attribute.
    same_site: SameSite,
}

impl Into<HeaderValue> for &Cookie {
    fn into(self) -> HeaderValue {
        HeaderValue::from_str(&self.to_string()).unwrap()
    }
}

impl fmt::Display for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;

        if self.max_age.num_seconds() >= 0 {
            write!(f, "; Max-Age={}", self.max_age.num_seconds())?;
        }

        if !self.path.is_empty() {
            write!(f, "; Path={}", self.path)?;
        }

        if self.secure {
            write!(f, "; Secure")?;
        }

        if self.http_only {
            write!(f, "; HttpOnly")?;
        }

        write!(f, "; SameSite={}", self.same_site)?;

        Ok(())
    }
}

/// Creates a new session token cookie.
pub fn create_session_token_cookie<T: Into<String>>(token: T) -> Cookie {
    build_cookie(
        SESSION_TOKEN_COOKIE_KEY,
        token,
        SESSION_TOKEN_EXPIRY_DURATION,
    )
}

/// Creates a new oauth cookie.
pub fn create_oauth_cookie<S, T>(name: S, value: T) -> Cookie
where
    S: Into<String>,
    T: Into<String>,
{
    build_cookie(name, value, Duration::minutes(10))
}

/// Creates a cookie that instructs the browser to delete it.
pub fn create_expired_cookie<S>(name: S) -> Cookie
where
    S: Into<String>,
{
    build_cookie(name, "", Duration::zero())
}

fn build_cookie<N: Into<String>, V: Into<String>>(name: N, value: V, max_age: Duration) -> Cookie {
    Cookie {
        name: name.into(),
        value: value.into(),
        max_age,
        path: String::from("/"),
        secure: false, // TODO: Enable on production
        http_only: true,
        same_site: SameSite::Lax,
    }
}

// Sets the session token cookie in the response headers.
pub(crate) fn set_session_token_cookie<B, T: Into<String>>(
    response: &mut http::Response<B>,
    token: T,
) {
    use http::header::{HeaderValue, SET_COOKIE};
    let cookie = create_session_token_cookie(token);
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&cookie.to_string()).unwrap(),
    );
}

/// Extracts the session token cookie from the response headers.
pub fn extract_session_token_cookie(value: &HeaderValue) -> Option<String> {
    extract_cookie_by_name(SESSION_TOKEN_COOKIE_KEY, value)
}

/// Extracts a cookie by name from a cookie header value.
pub fn extract_cookie_by_name(name: &str, value: &HeaderValue) -> Option<String> {
    value
        .to_str()
        .ok()?
        .split(';')
        .map(str::trim)
        .filter_map(|cookie| cookie.split_once('='))
        .find_map(|(k, v)| (k == name).then(|| v.to_string()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SameSite {
    Lax,
}

impl fmt::Display for SameSite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SameSite::Lax => write!(f, "Lax"),
        }
    }
}

/// A helper extension for attaching cookies to HTTP responses.
pub trait ResponseCookies {
    /// Adds a single [`Cookie`] to the response.
    fn with_cookie(self, cookie: Cookie) -> Self;

    /// Adds multiple [`Cookie`]s to the response.
    fn with_cookies(self, cookies: impl IntoIterator<Item = Cookie>) -> Self;
}

impl ResponseCookies for http::response::Builder {
    fn with_cookies(mut self, cookies: impl IntoIterator<Item = Cookie>) -> Self {
        for cookie in cookies {
            self = self.with_cookie(cookie);
        }
        self
    }

    fn with_cookie(mut self, cookie: Cookie) -> Self {
        self = self.header(
            http::header::SET_COOKIE,
            http::HeaderValue::from_str(&cookie.to_string()).expect("valid cookie"),
        );
        self
    }
}

#[cfg(test)]
mod tests {
    use axum::response::Response;
    use http::header::SET_COOKIE;

    use super::*;

    #[test]
    fn test_session_token_cookie() {
        // when
        let cookie = create_session_token_cookie("session-token");

        // then
        assert_eq!(
            cookie.to_string(),
            "session_token=session-token; Max-Age=604800; Path=/; HttpOnly; SameSite=Lax"
        );
    }

    #[test]
    fn test_oauth_cookie() {
        // when
        let cookie = create_oauth_cookie("name", "value");

        // then
        assert_eq!(
            cookie.to_string(),
            "name=value; Max-Age=600; Path=/; HttpOnly; SameSite=Lax"
        );
    }

    #[test]
    fn test_expired_cookie() {
        // when
        let cookie = create_expired_cookie("name");

        // then
        assert_eq!(
            cookie.to_string(),
            "name=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax"
        );
    }

    #[test]
    fn test_extract_cookie() {
        // given
        let cookie = build_cookie("name", "value", Duration::zero());
        let header = HeaderValue::from_str(&cookie.to_string()).unwrap();

        // when
        let cookie = extract_cookie_by_name("name", &header);

        // then
        assert_eq!(cookie, Some("value".to_string()));
    }

    #[test]
    fn test_response_with_cookie() {
        // given
        let cookie = build_cookie("name", "value", Duration::zero());

        // when
        let response = Response::builder().with_cookie(cookie).body(()).unwrap();

        // then
        assert_eq!(
            response.headers().get(SET_COOKIE).unwrap(),
            "name=value; Max-Age=0; Path=/; HttpOnly; SameSite=Lax"
        );
    }

    #[test]
    fn test_response_with_cookies() {
        // given
        let cookie1 = build_cookie("name1", "value1", Duration::zero());
        let cookie2 = build_cookie("name2", "value2", Duration::zero());

        // when
        let response = Response::builder()
            .with_cookies([cookie1, cookie2])
            .body(())
            .unwrap();

        // then
        let headers: Vec<_> = response.headers().get_all(SET_COOKIE).iter().collect();
        assert_eq!(
            extract_cookie_by_name("name1", headers[0]).unwrap(),
            "value1"
        );
        assert_eq!(
            extract_cookie_by_name("name2", headers[1]).unwrap(),
            "value2"
        );
    }

    #[test]
    fn test_set_session_token() {
        // given
        let token = "token";
        let mut response = Response::builder().body(()).unwrap();

        // when
        set_session_token_cookie(&mut response, token);

        // then
        assert_eq!(
            response.headers().get(SET_COOKIE).unwrap(),
            "session_token=token; Max-Age=604800; Path=/; HttpOnly; SameSite=Lax"
        );
    }
}
