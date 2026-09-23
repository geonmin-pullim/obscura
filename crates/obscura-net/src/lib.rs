pub mod client;
pub mod cookies;
pub mod encoding;
pub mod interceptor;
pub mod robots;
pub mod blocklist;
#[cfg(feature = "stealth")]
pub mod wreq_client;

pub use client::{
    env_allows_private_network, is_forbidden_ip, CallbackRegistry, ObscuraHttpClient,
    ObscuraNetError, RequestCallback, RequestCredentials, RequestInfo, RequestMode,
    ResourceRequest, ResourceType, Response, ResponseCallback, SsrfGuardResolver,
};
pub use cookies::{
    canonical_domain, default_cookie_path, same_site, CookieInfo, CookieJar, SameSiteContext,
};
pub use encoding::{
    decode_non_html, decode_response, decode_response_with_name, decode_with_label, label_name,
    url_encode_query,
};
pub use robots::RobotsCache;
pub use blocklist::is_blocked as is_tracker_blocked;
#[cfg(feature = "stealth")]
pub use wreq_client::{
    StealthHttpClient, STEALTH_NAVIGATOR_PLATFORM, STEALTH_UA_PLATFORM,
    STEALTH_UA_PLATFORM_VERSION, STEALTH_USER_AGENT,
};

/// `OBSCURA_LANGUAGES`, e.g. `ko-KR,ko,en-US,en`: the language list the
/// browser identity reports (navigator.languages, Accept-Language, Intl).
/// None when unset, which keeps the built-in en-US identity.
pub fn env_languages() -> Option<Vec<String>> {
    let list: Vec<String> = std::env::var("OBSCURA_LANGUAGES")
        .ok()?
        .split(',')
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    (!list.is_empty()).then_some(list)
}

/// Chrome's Accept-Language for a language list:
/// `[ko-KR, ko, en-US, en]` -> `ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7`.
pub fn accept_language_for(languages: &[String]) -> String {
    languages
        .iter()
        .enumerate()
        .map(|(i, l)| match i {
            0 => l.clone(),
            _ => format!("{l};q=0.{}", 10usize.saturating_sub(i).max(1)),
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// Accept-Language derived from `OBSCURA_LANGUAGES`, computed once.
pub fn env_accept_language() -> Option<&'static str> {
    static VALUE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    VALUE
        .get_or_init(|| env_languages().map(|l| accept_language_for(&l)))
        .as_deref()
}

#[cfg(test)]
mod language_tests {
    #[test]
    fn accept_language_matches_chrome_format() {
        let langs: Vec<String> = ["ko-KR", "ko", "en-US", "en"].iter().map(|s| s.to_string()).collect();
        assert_eq!(super::accept_language_for(&langs), "ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7");
    }
}
