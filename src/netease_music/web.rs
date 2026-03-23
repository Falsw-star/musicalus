use std::sync::Arc;

use reqwest::header;

use crate::CONFIG;

lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::Client = make_client().unwrap();
}

pub fn make_client() -> anyhow::Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:143.0) Gecko/20100101 Firefox/143.0"
            .parse().unwrap()
    );
    headers.insert(
        header::REFERER,
        "https://music.163.com".parse().unwrap()
    );

    let cookie_store = Arc::new(reqwest::cookie::Jar::default());
    let cookie_url = reqwest::Url::parse("https://music.163.com").unwrap();
    cookie_store.add_cookie_str(
        &format!("MUSIC_U={}", &CONFIG.cookie),
        &cookie_url
    );
    cookie_store.add_cookie_str("__remember_me=true", &cookie_url);
    cookie_store.add_cookie_str("os=pc", &cookie_url);

    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .cookie_provider(cookie_store)
        .build()?)
}