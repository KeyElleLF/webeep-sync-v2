//! Authentication: login window, token parsing/verification and secure storage.

pub mod store;
pub mod token;
pub mod window;

use url::Url;

/// WeBeep site root as configured in Moodle (`wwwroot`).
pub const SITE_ROOT: &str = "https://webeep.polimi.it";
/// Accepted `wwwroot` values for the token signature (verified on 2026-11-09: https only).
pub const SIGNATURE_ROOTS: &[&str] = &[SITE_ROOT];
/// Custom scheme requested from `launch.php`.
pub const URL_SCHEME: &str = "moodlemobile";
/// Web service used by the official Moodle app, enabled on WeBeep.
const MOBILE_SERVICE: &str = "moodle_mobile_app";

/// Base URL of the site, with trailing slash so that `join` keeps the root.
pub fn site_url() -> Result<Url, url::ParseError> {
    Url::parse(&format!("{SITE_ROOT}/"))
}

/// Builds the `launch.php` URL that makes Moodle issue a mobile web service token.
pub fn launch_url(passport: &str) -> Result<Url, url::ParseError> {
    let mut url = site_url()?.join("admin/tool/mobile/launch.php")?;
    url.query_pairs_mut()
        .append_pair("service", MOBILE_SERVICE)
        .append_pair("passport", passport)
        .append_pair("urlscheme", URL_SCHEME);
    Ok(url)
}

/// Random, unguessable passport (128 bits, hex) used to sign the token exchange.
pub fn new_passport() -> String {
    format!("{:032x}", rand::random::<u128>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_url_has_trailing_slash() {
        assert_eq!(site_url().unwrap().as_str(), "https://webeep.polimi.it/");
    }

    #[test]
    fn launch_url_contains_service_passport_and_scheme() {
        let url = launch_url("abc123").unwrap();
        assert_eq!(
            url.as_str(),
            "https://webeep.polimi.it/admin/tool/mobile/launch.php\
             ?service=moodle_mobile_app&passport=abc123&urlscheme=moodlemobile"
        );
    }

    #[test]
    fn passports_are_long_and_unique() {
        let a = new_passport();
        let b = new_passport();
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
    }
}
