use axum::{Json, Router, body::Body, extract::Query, response::{Html, IntoResponse, Redirect, Response}, routing::{any, get, post}};
use li_logger::get_logger;
use reqwest::{StatusCode, header};
use serde_json::{Value, json};
use thiserror::Error;

use crate::netease_music::{protocal::*, web::CLIENT};

pub mod protocal;
pub mod crypto;
pub mod web;

#[derive(Error, Debug)]
pub enum NetEaseMusicError {
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Operation failed: {0}")]
    Operation(#[from] anyhow::Error),
    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Server error: {0}")]
    Server(#[from] axum::http::Error)
}

impl IntoResponse for NetEaseMusicError {
    fn into_response(self) -> Response {
        get_logger().error(&self);
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
pub fn router() -> Router {
    Router::new()
        .route("/", any(info))
        .route("/song", get(song_get))
        .route("/song", post(song_post))
        .route("/audio", get(audio_get))
        .route("/audio", post(audio_post))
        .route("/audio/redirect", get(audio_redirect_get))
        .route("/audio/redirect", post(audio_redirect_post))
        .route("/audio/proxy", get(audio_proxy_get))
        .route("/audio/proxy", post(audio_proxy_post))
        .route("/search", get(search_get))  
        .route("/search", post(search_post))
        .route("/playlist/download", get(plsylist_download_get))
        .route("/playlist", get(playlist_get))
        .route("/playlist", post(playlist_post))
        .route("/album", get(album_get))
        .route("/album", post(album_post))
}

async fn song_get(Query(req): Query<SongsRequest>) -> Result<Json<Vec<Song>>, NetEaseMusicError> {
    song(req.id_list()).await
}

async fn song_post(Json(req): Json<SongsRequest>) -> Result<Json<Vec<Song>>, NetEaseMusicError> {
    song(req.id_list()).await
}

async fn song(id_list: Vec<u64>) -> Result<Json<Vec<Song>>, NetEaseMusicError> {
    if id_list.is_empty() {
        return Ok(Json(vec![]));
    }
    let data = serde_json::to_string(&json!({
        "c": serde_json::to_string(&Value::Array(
            id_list.iter().map(|id| json!({"id": id}))
            .collect::<Vec<Value>>()
        ))?,
        "ids": serde_json::to_string(&Value::Array(
            id_list.iter().map(|id| json!(id))
            .collect::<Vec<Value>>()
        ))?,
    }))?;
    let form = crypto::make_weapi_form(data)?;
    
    let songs = CLIENT.post("https://music.163.com/weapi/v3/song/detail")
        .form(&form).send().await?.json::<Songs>().await?;
    
    Ok(Json(songs.songs))
}

async fn audio_get(Query(req): Query<AudiosRequest>) -> Result<Json<Vec<Audio>>, NetEaseMusicError> {
    Ok(Json(audio(req.id_list(), req.quality.unwrap_or(Quality::Standard)).await?))
}

async fn audio_post(Json(req): Json<AudiosRequest>) -> Result<Json<Vec<Audio>>, NetEaseMusicError> {
    Ok(Json(audio(req.id_list(), req.quality.unwrap_or(Quality::Standard)).await?))
}

async fn audio_redirect_get(Query(req): Query<AudioPlayRequest>) -> Result<Redirect, NetEaseMusicError> {
    audio_redirect(req.id, req.quality.unwrap_or(Quality::Standard)).await
}

async fn audio_redirect_post(Json(req): Json<AudioPlayRequest>) -> Result<Redirect, NetEaseMusicError> {
    audio_redirect(req.id, req.quality.unwrap_or(Quality::Standard)).await
}

async fn audio_redirect(id: u64, quality: Quality) -> Result<Redirect, NetEaseMusicError> {
    let url = audio(vec![id], quality).await?
        .pop().ok_or(anyhow::anyhow!("Id not found"))?
        .url.ok_or(anyhow::anyhow!("Url not found"))?;
    Ok(Redirect::to(&url))
}

async fn audio_proxy_get(Query(req): Query<AudioPlayRequest>) -> Result<Response<Body>, NetEaseMusicError> {
    audio_proxy(req.id, req.quality.unwrap_or(Quality::Standard)).await
}

async fn audio_proxy_post(Json(req): Json<AudioPlayRequest>) -> Result<Response<Body>, NetEaseMusicError> {
    audio_proxy(req.id, req.quality.unwrap_or(Quality::Standard)).await
}

async fn audio_proxy(id: u64, quality: Quality) -> Result<Response<Body>, NetEaseMusicError> {
    let url = audio(vec![id], quality).await?
        .pop().ok_or(anyhow::anyhow!("Id not found"))?
        .url.ok_or(anyhow::anyhow!("Url not found"))?;
    
    let response = CLIENT.get(url).send().await?;

    let mut builder = Response::builder();
    if let Some(content_type) = response.headers().get(header::CONTENT_TYPE) {
        builder = builder.header(header::CONTENT_TYPE, content_type.clone());
    }
    if let Some(content_length) = response.headers().get(header::CONTENT_LENGTH) {
        builder = builder.header(header::CONTENT_LENGTH, content_length.clone());
    }
    if let Some(content_disposition) = response.headers().get(header::CONTENT_DISPOSITION) {
        builder = builder.header(header::CONTENT_DISPOSITION, content_disposition.clone());
    }
    builder =builder.header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    Ok(builder.body(Body::from(response.bytes().await?))?)
}

async fn audio(id_list: Vec<u64>, quality: Quality) -> Result<Vec<Audio>, NetEaseMusicError> {
    let payload = serde_json::to_string(&json!({
        "ids": id_list,
        "level": quality.to_string(),
        "encodeType": "flac", // fallback to mp3 at netease side
        "header": crypto::make_eapi_header()?
    }))?;
    let form = crypto::make_eapi_form(
        "/api/song/enhance/player/url/v1".to_string(),
        payload
    )?;

    let mut audios = CLIENT.post("https://interface3.music.163.com/eapi/song/enhance/player/url/v1")
        .form(&form).send().await?.json::<Audios>().await?;
    audios.fix_urls();

    Ok(audios.data)
}

async fn search_get(Query(req): Query<SearchRequest>) -> Result<Json<Vec<Song>>, NetEaseMusicError> {
    search(req.keyword, req.limit.unwrap_or(10)).await
}

async fn search_post(Json(req): Json<SearchRequest>) -> Result<Json<Vec<Song>>, NetEaseMusicError> {
    search(req.keyword, req.limit.unwrap_or(10)).await
}

async fn search(keyword: String, limit: u64) -> Result<Json<Vec<Song>>, NetEaseMusicError> {
    let form = json!({
        "s": keyword,
        "type": 1,
        "limit": limit
    });
    let response = CLIENT.post("https://music.163.com/api/cloudsearch/pc")
        .form(&form).send().await?.json::<SearchResponse>().await?;

    Ok(Json(response.result.songs))
}

async fn plsylist_download_get() -> Result<Response, NetEaseMusicError> {
    let page = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Down</title>
        </head>
        <body>
            <div id="text"></div>
        </body>
        <script src="/static/netease/playlist_download.js"></script>
        </html>
    "#;
    Ok(Html(page).into_response())
}

async fn playlist_get(Query(req): Query<PlaylistRequest>) -> Result<Json<Playlist>, NetEaseMusicError> {
    playlist(req.id).await
}

async fn playlist_post(Json(req): Json<PlaylistRequest>) -> Result<Json<Playlist>, NetEaseMusicError> {
    playlist(req.id).await
}

async fn playlist(id: u64) -> Result<Json<Playlist>, NetEaseMusicError> {
    let form = json!({
        "id": id,
        "n": 100000,
        "s": 8
    });
    let response = CLIENT.post("https://music.163.com/api/v6/playlist/detail")
        .form(&form).send().await?.json::<PlaylistResponse>().await?;
    Ok(Json(response.playlist))
}

async fn album_get(Query(req): Query<AlbumRequest>) -> Result<Json<Album>, NetEaseMusicError> {
    album(req.id).await
}

async fn album_post(Json(req): Json<AlbumRequest>) -> Result<Json<Album>, NetEaseMusicError> {
    album(req.id).await
}

async fn album(id: u64) -> Result<Json<Album>, NetEaseMusicError> {
    let album = CLIENT.get(format!("https://music.163.com/api/v1/album/{id}"))
        .send().await?.json::<AlbumResponse>().await?;
    Ok(Json(album.album))
}

async fn info() -> Result<Json<Value>, NetEaseMusicError> {
    Ok(Json(json!({
        "is_vip": vip().await?
    })))
}

async fn vip() -> Result<bool, NetEaseMusicError> {
    let form = crypto::make_weapi_form("{}".to_string())?;
    let response = CLIENT.post("https://music.163.com/weapi/nuser/account/get")
        .form(&form).send().await?.json::<Value>().await?;
    let vip_type = response
        .get("account")
        .and_then(|value| value.get("vipType"))
        .and_then(|value| value.as_i64())
        .ok_or(anyhow::anyhow!("Failed to get VIP type from response"))?;
    Ok(vip_type > 0)
}