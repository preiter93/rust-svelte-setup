use base64::Engine as _;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use rand::distr::{Alphanumeric, SampleString as _};

use rand::rngs::StdRng;
use rand::{Rng as _, SeedableRng as _};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Clone, PartialEq)]
pub struct Session {
    pub id: String,
    pub secret_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub user_id: String,
}

/// Generates cryptographically secure random strings.
///
/// [`Documentation`]: https://lucia-auth.com/sessions/basic
#[must_use]
pub fn generate_secure_random_string() -> String {
    let mut rng = StdRng::from_os_rng();

    Alphanumeric.sample_string(&mut rng, 24)
}

/// Generates the oauth state/csrf token.
#[must_use]
pub fn generate_random_base64_encoded_string(num_bytes: usize) -> String {
    let random_bytes: Vec<u8> = (0..num_bytes).map(|_| rand::rng().random()).collect();
    BASE64_URL_SAFE_NO_PAD.encode(&random_bytes[..])
}

pub(crate) struct OAuth;
impl OAuth {
    /// Generates the oauth state/csrf token.
    #[must_use]
    pub fn generate_state() -> String {
        generate_random_base64_encoded_string(32)
    }

    /// Generates the oauth code verifier.
    #[must_use]
    pub fn generate_code_verifier() -> String {
        generate_random_base64_encoded_string(32)
    }

    /// Creates a S256 code challenge.
    fn create_s256_code_challenge(code_verifier: &str) -> String {
        let digest = Sha256::digest(code_verifier.as_bytes());
        let code_challenge = BASE64_URL_SAFE_NO_PAD.encode(digest);
        code_challenge
    }
}

#[derive(Clone)]
pub(crate) struct GoogleOAuth {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl GoogleOAuth {
    const AUTHORIZATION_ENDPOINT: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
    const TOKEN_ENDPOINT: &'static str = "https://oauth2.googleapis.com/token";
    const TOKEN_REVOCATION_ENDPOINT: &'static str = "https://oauth2.googleapis.com/revoke";

    /// Creates a new [Google] oauth client.
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
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
        let code_challenge = OAuth::create_s256_code_challenge(code_verifier);
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

    // async validateAuthorizationCode(tokenEndpoint, code, codeVerifier) {
    //     const body = new URLSearchParams();
    //     body.set("grant_type", "authorization_code");
    //     body.set("code", code);
    //     if (this.redirectURI !== null) {
    //         body.set("redirect_uri", this.redirectURI);
    //     }
    //     if (codeVerifier !== null) {
    //         body.set("code_verifier", codeVerifier);
    //     }
    //     if (this.clientPassword === null) {
    //         body.set("client_id", this.clientId);
    //     }
    //     const request = createOAuth2Request(tokenEndpoint, body);
    //     if (this.clientPassword !== null) {
    //         const encodedCredentials = encodeBasicCredentials(this.clientId, this.clientPassword);
    //         request.headers.set("Authorization", `Basic ${encodedCredentials}`);
    //     }
    //     const tokens = await sendTokenRequest(request);
    //     return tokens;
    // }
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
