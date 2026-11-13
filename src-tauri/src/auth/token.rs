//! Parsing and verification of the token returned by Moodle's `launch.php`.
//!
//! `launch.php` redirects to `<scheme>://token=<BASE64>`, where the decoded payload is
//! `<signature>:::<token>[:::<privatetoken>]` and `signature = md5(wwwroot + passport)`.
//! The private token is deliberately discarded: we never need it and never store it.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use md5::{Digest, Md5};
use thiserror::Error;

/// Separator used by Moodle inside the decoded token payload.
const PAYLOAD_SEPARATOR: &str = ":::";
/// Marker that precedes the base64 payload in the redirect URL.
const TOKEN_MARKER: &str = "token=";

/// A Moodle web service token. `Debug` never reveals the value.
#[derive(Clone, PartialEq, Eq)]
pub struct WsToken(String);

impl WsToken {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Exposes the secret. Call sites must never log the returned value.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for WsToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WsToken(<redacted>)")
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TokenError {
    #[error("l'URL di ritorno non contiene un token")]
    MissingToken,
    #[error("il token ricevuto non è codificato correttamente")]
    BadEncoding,
    #[error("il token ricevuto ha un formato inatteso")]
    BadFormat,
    #[error("la firma del token non è valida: login rifiutato")]
    SignatureMismatch,
}

/// Extracts and verifies the web service token from a `launch.php` redirect URL.
///
/// `site_roots` lists the accepted values of Moodle's `wwwroot` (e.g. with and without https);
/// the signature must match one of them combined with `passport`.
pub fn parse_launch_redirect(
    redirect_url: &str,
    passport: &str,
    site_roots: &[&str],
) -> Result<WsToken, TokenError> {
    let payload = decode_payload(extract_base64(redirect_url)?)?;
    let mut parts = payload.split(PAYLOAD_SEPARATOR);
    let signature = parts.next().ok_or(TokenError::BadFormat)?;
    let token = parts.next().ok_or(TokenError::BadFormat)?;
    if token.is_empty() || !token.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(TokenError::BadFormat);
    }
    let is_signed = site_roots
        .iter()
        .any(|root| signature.eq_ignore_ascii_case(&expected_signature(root, passport)));
    if !is_signed {
        return Err(TokenError::SignatureMismatch);
    }
    Ok(WsToken::new(token))
}

/// Computes `md5(site_root + passport)` as lowercase hex, as `launch.php` does.
pub fn expected_signature(site_root: &str, passport: &str) -> String {
    let digest = Md5::digest(format!("{site_root}{passport}").as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn extract_base64(redirect_url: &str) -> Result<&str, TokenError> {
    let start = redirect_url
        .find(TOKEN_MARKER)
        .ok_or(TokenError::MissingToken)?
        + TOKEN_MARKER.len();
    // Some webviews append a trailing slash, a query or a fragment to custom-scheme URLs.
    // '/' is a valid base64 character, so only a trailing one is stripped.
    let raw = redirect_url[start..]
        .split(['?', '#', '&'])
        .next()
        .unwrap_or_default()
        .trim_end_matches('/');
    if raw.is_empty() {
        return Err(TokenError::MissingToken);
    }
    Ok(raw)
}

fn decode_payload(raw: &str) -> Result<String, TokenError> {
    let unescaped = raw.replace("%3D", "=").replace("%3d", "=");
    let bytes = STANDARD
        .decode(unescaped.as_bytes())
        .map_err(|_| TokenError::BadEncoding)?;
    String::from_utf8(bytes).map_err(|_| TokenError::BadEncoding)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "https://webeep.polimi.it";
    const PASSPORT: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90";
    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    fn redirect_for(payload: &str) -> String {
        format!("moodlemobile://token={}", STANDARD.encode(payload))
    }

    fn signed_payload(root: &str) -> String {
        format!("{}:::{}", expected_signature(root, PASSPORT), TOKEN)
    }

    #[test]
    fn signature_matches_known_md5() {
        // md5("abc") is a well-known test vector.
        assert_eq!(
            expected_signature("a", "bc"),
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }

    #[test]
    fn parses_valid_signed_token() {
        let url = redirect_for(&signed_payload(ROOT));
        let token = parse_launch_redirect(&url, PASSPORT, &[ROOT]).unwrap();
        assert_eq!(token.expose(), TOKEN);
    }

    #[test]
    fn discards_private_token() {
        let payload = format!("{}:::privatesecret", signed_payload(ROOT));
        let token = parse_launch_redirect(&redirect_for(&payload), PASSPORT, &[ROOT]).unwrap();
        assert_eq!(token.expose(), TOKEN);
    }

    #[test]
    fn accepts_any_listed_site_root() {
        let http_root = "http://webeep.polimi.it";
        let url = redirect_for(&signed_payload(http_root));
        assert!(parse_launch_redirect(&url, PASSPORT, &[ROOT, http_root]).is_ok());
    }

    #[test]
    fn tolerates_trailing_slash_and_escaped_padding() {
        let url = redirect_for(&signed_payload(ROOT)).replace('=', "%3D");
        let url = url.replacen("token%3D", "token=", 1) + "/";
        assert!(parse_launch_redirect(&url, PASSPORT, &[ROOT]).is_ok());
    }

    #[test]
    fn rejects_wrong_passport() {
        let url = redirect_for(&signed_payload(ROOT));
        let result = parse_launch_redirect(&url, "another-passport", &[ROOT]);
        assert_eq!(result, Err(TokenError::SignatureMismatch));
    }

    #[test]
    fn rejects_url_without_token() {
        let result = parse_launch_redirect("moodlemobile://nothing", PASSPORT, &[ROOT]);
        assert_eq!(result, Err(TokenError::MissingToken));
        let result = parse_launch_redirect("moodlemobile://token=", PASSPORT, &[ROOT]);
        assert_eq!(result, Err(TokenError::MissingToken));
    }

    #[test]
    fn rejects_invalid_base64() {
        let result = parse_launch_redirect("moodlemobile://token=!!!", PASSPORT, &[ROOT]);
        assert_eq!(result, Err(TokenError::BadEncoding));
    }

    #[test]
    fn rejects_payload_without_separator() {
        let result = parse_launch_redirect(&redirect_for("justonefield"), PASSPORT, &[ROOT]);
        assert_eq!(result, Err(TokenError::BadFormat));
    }

    #[test]
    fn rejects_token_with_unexpected_characters() {
        let payload = format!("{}:::bad token<>", expected_signature(ROOT, PASSPORT));
        let result = parse_launch_redirect(&redirect_for(&payload), PASSPORT, &[ROOT]);
        assert_eq!(result, Err(TokenError::BadFormat));
    }

    #[test]
    fn debug_output_never_contains_token() {
        let rendered = format!("{:?}", WsToken::new(TOKEN));
        assert!(!rendered.contains(TOKEN));
    }
}
