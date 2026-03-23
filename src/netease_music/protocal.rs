use li_logger::get_logger;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SongsRequest {
    pub id: Option<u64>,
    pub ids: Option<Vec<u64>>
}

impl SongsRequest {
    pub fn id_list(&self) -> Vec<u64> {
        let mut id_list = Vec::new();
        self.id.inspect(|id| id_list.push(*id));
        self.ids.clone().inspect(|ids| ids.iter().for_each(|id: &u64| id_list.push(*id)));
        id_list
    }
}

#[derive(Serialize, Deserialize)]
pub struct Songs {
    pub songs: Vec<Song>
}

#[derive(Serialize, Deserialize)]
pub struct Song {
    pub id: u64,
    pub name: String,
    #[serde(rename(deserialize = "ar"))]
    pub artists: Option<Vec<Artist>>,
    #[serde(rename(deserialize = "al"))]
    pub album: Album,
    #[serde(rename(deserialize = "mv"))]
    pub mv_id: Option<u64>
}

#[derive(Serialize, Deserialize)]
pub struct Artist {
    pub id: u64,
    pub name: String,
    #[serde(rename(deserialize = "picUrl"))]
    pub avatar: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct AudiosRequest {
    pub id: Option<u64>,
    pub ids: Option<Vec<u64>>,
    pub quality: Option<Quality>
}

impl AudiosRequest {
    pub fn id_list(&self) -> Vec<u64> {
        let mut id_list = Vec::new();
        self.id.inspect(|id| id_list.push(*id));
        self.ids.clone().inspect(|ids| ids.iter().for_each(|id: &u64| id_list.push(*id)));
        id_list
    }
}

#[derive(Serialize, Deserialize)]
pub struct Audios { 
    pub data: Vec<Audio>
}

impl Audios {
    pub fn fix_urls(&mut self) {
        for audio in self.data.iter_mut() {
            audio.fix_url();
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Audio { 
    pub id: u64,
    pub url: Option<String>,
    #[serde(rename(deserialize = "type"))]
    pub encoding: Option<String>,
    #[serde(rename(deserialize = "br"))]
    pub bitrate: Option<u64>,
    pub size: Option<u64>,
    pub md5: Option<String>,
    #[serde(rename(deserialize = "time"))]
    pub duration: Option<u64>,
    #[serde(rename(deserialize = "sr"))]
    pub sample_rate: Option<u64>
}

impl Audio {
    pub fn fix_url(&mut self) {
        if let Some(ref url) = self.url {
            match fix_url(url) {
                Ok(url) => {
                    self.url = Some(url);
                },
                Err(err) => {
                    get_logger().error(format!("url invalid: {}", err));
                    self.url = None;
                }
            }
        }
    }
}

/// ### fix url
/// m04.music.126.net -> m01.music.126.net
fn fix_url(url: &str) -> anyhow::Result<String> { 
    let mut parsed = url::Url::parse(url)?;
    let host = parsed.host_str().ok_or(anyhow::anyhow!("without host"))?;
    let new_host = 
    if let Some(captures) = 
    regex::Regex::new(r"^m([78])04\.music\.126\.net$")?
        .captures(host) {
            format!("m{}01.music.126.net", &captures[1])
        } else {
            host.to_string()
        };
    parsed.set_scheme("https").map_err(|_| anyhow::anyhow!("failed to set scheme"))?;
    parsed.set_host(Some(&new_host))?;
    Ok(parsed.to_string())
}

#[derive(Serialize, Deserialize)]
pub enum Quality {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "exhigh")]
    Exhigh,
    #[serde(rename = "lossless")]
    Lossless
}

impl ToString for Quality {
    fn to_string(&self) -> String {
        match self {
            Quality::Standard => "standard",
            Quality::Exhigh => "exhigh",
            Quality::Lossless => "lossless"
        }.to_string()
    }
}

#[derive(Serialize, Deserialize)]
pub struct AudioPlayRequest {
    pub id: u64,
    pub quality: Option<Quality>
}

#[derive(Serialize, Deserialize)]
pub struct SearchRequest {
    pub keyword: String,
    pub limit: Option<u64>
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse { 
    pub result: Songs
}

#[derive(Serialize, Deserialize)]
pub struct PlaylistRequest {
    pub id: u64
}

#[derive(Serialize, Deserialize)]
pub struct Playlist {
    pub id: u64,
    pub name: String,
    pub decription: Option<String>,
    #[serde(rename(deserialize = "coverImgUrl"))]
    pub cover: Option<String>,
    #[serde(rename(deserialize = "trackCount"))]
    pub track_count: u64,
    #[serde(rename(deserialize = "playCount"))]
    pub play_count: u64,
    pub creator: Option<PlaylistCreator>,
    pub tracks: Vec<Song>
}

#[derive(Serialize, Deserialize)]
pub struct PlaylistCreator {
    #[serde(rename(deserialize = "userId"))]
    pub id: u64,
    #[serde(rename(deserialize = "nickname"))]
    pub name: String,
    pub signature: Option<String>,
    #[serde(rename(deserialize = "avatarUrl"))]
    pub avatar: Option<String>,
    #[serde(rename(deserialize = "backgroundUrl"))]
    pub background: Option<String>,
    #[serde(rename(deserialize = "city"))]
    pub city_code: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct AlbumRequest { 
    pub id: u64
}

#[derive(Serialize, Deserialize)]
pub struct AlbumWrapper {
    pub album: Album
}

#[derive(Serialize, Deserialize)]
pub struct Album { 
    pub id: u64,
    pub name: String,
    #[serde(rename(deserialize = "picUrl"))]
    pub cover: Option<String>,
    pub description: Option<String>,
    pub artists: Option<Vec<Artist>>,
    pub songs: Option<Vec<Song>>,
}