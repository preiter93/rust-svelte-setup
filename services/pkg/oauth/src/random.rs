use base64::Engine as _;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use rand::{
    Rng,
    distr::{Alphanumeric, SampleString as _},
};
use uuid::Uuid;

/// A source of cryptographically secure random values.
///
/// Most users will not implement this directly — use [`SecureRandom`]
/// (the default) or a mock implementation for testing.
///
/// # Example
/// ```
/// use oauth::RandomSource;
///
/// struct MockRandom;
///
/// impl RandomSource for MockRandom {
///     fn alphanumeric(_len: usize) -> String {
///         "MOCK123".to_string()
///     }
///     fn base64_url(_len: usize) -> String {
///         "dGVzdA".to_string()
///     }
///     fn uuid() -> uuid::Uuid {
///         uuid::Uuid::nil()
///     }
/// }
/// ```
pub trait RandomSource: Send + Sync + 'static {
    /// Returns a secure alphanumeric string (for PKCE verifier, etc.).
    fn alphanumeric(len: usize) -> String;

    /// Returns a random base64-url string (no padding).
    fn base64_url(num_bytes: usize) -> String;

    /// Returns a random UUIDv4.
    fn uuid() -> Uuid;
}

/// Default cryptographically secure random generator using [`OsRng`].
#[derive(Debug, Clone, Default)]
pub struct SecureRandom;

impl RandomSource for SecureRandom {
    fn alphanumeric(len: usize) -> String {
        Alphanumeric.sample_string(&mut rand::rng(), len)
    }

    fn base64_url(num_bytes: usize) -> String {
        let random_bytes: Vec<u8> = (0..num_bytes).map(|_| rand::rng().random()).collect();
        BASE64_URL_SAFE_NO_PAD.encode(&random_bytes)
    }

    fn uuid() -> Uuid {
        Uuid::new_v4()
    }
}

#[cfg(feature = "mock")]
pub mod mock {
    use super::*;

    /// Mock random generator for testing.
    #[derive(Default, Clone)]
    pub struct MockRandom;

    impl RandomSource for MockRandom {
        fn alphanumeric(_: usize) -> String {
            "secret".to_string()
        }

        fn base64_url(_: usize) -> String {
            "secret-encoded".to_string()
        }

        fn uuid() -> Uuid {
            Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap()
        }
    }
}
