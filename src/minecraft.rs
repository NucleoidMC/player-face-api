use std::collections::HashMap;
use std::io;
use std::io::Read;

use bytes::Bytes;
use image::{DynamicImage, ImageFormat};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use tokio::time::Duration;
use uuid::Uuid;
use warp::hyper::http::StatusCode;

const PROFILE_ENDPOINT: &'static str = "https://sessionserver.mojang.com/session/minecraft/profile";
const TIMEOUT: Duration = Duration::from_secs(10);

pub async fn get_profile(uuid: Uuid) -> Result<Option<PlayerProfile>> {
    log::debug!("getting player profile for {}", uuid);

    let client = client()?;
    let url = format!("{}/{}", PROFILE_ENDPOINT, uuid);

    let response = client.get(url).send().await?;
    match response.status() {
        StatusCode::OK => Ok(Some(response.json().await?)),
        _ => Ok(None),
    }
}

pub async fn get_texture(texture: PlayerTextureRef) -> Result<PlayerTexture> {
    log::debug!("requesting player skin at {}", texture.url);

    let client = client()?;
    let response = client.get(&texture.url).send().await?;
    let response = response.bytes().await?;

    tokio::task::spawn_blocking(move || resolve_texture(texture, response)).await.unwrap()
}

fn resolve_texture(texture: PlayerTextureRef, bytes: Bytes) -> Result<PlayerTexture> {
    let cursor = io::Cursor::new(bytes.as_ref());
    let reader = image::io::Reader::with_format(cursor, ImageFormat::Png);
    let image = reader.decode()?;
    match image {
        DynamicImage::ImageRgba8(image) => Ok(PlayerTexture {
            image,
            metadata: texture.metadata,
        }),
        _ => Err(Error::InvalidImageFormat),
    }
}

fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .gzip(true)
        .timeout(TIMEOUT)
        .use_rustls_tls()
        .build()
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<ProfileProperty>,
}

impl PlayerProfile {
    #[inline]
    pub fn textures(&self) -> Option<PlayerTextures> {
        self.property("textures")
    }

    pub fn property<T: DeserializeOwned>(&self, name: &str) -> Option<T> {
        let property = self.properties.iter()
            .find(|p| p.name == name)?;

        parse_base64(&property.value).ok()
    }
}

#[inline]
fn parse_base64<T: DeserializeOwned>(input: &str) -> Result<T> {
    let mut input = io::Cursor::new(input.as_bytes());
    let mut input = base64::read::DecoderReader::new(&mut input, base64::STANDARD);

    let mut bytes = Vec::new();
    input.read_to_end(&mut bytes)?;

    Ok(serde_json::from_slice(&bytes)?)
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerTextures {
    pub timestamp: u64,
    pub profile_id: Uuid,
    pub profile_name: String,
    #[serde(rename = "textures")]
    pub refs: PlayerTextureUrls,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerTextureUrls {
    #[serde(rename = "SKIN")]
    pub skin: Option<PlayerTextureRef>,
    #[serde(rename = "CAPE")]
    pub cape: Option<PlayerTextureRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerTextureRef {
    pub url: String,
    pub metadata: HashMap<String, String>,
}

pub struct PlayerTexture {
    pub image: image::RgbaImage,
    pub metadata: HashMap<String, String>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("http error")]
    Http(#[from] reqwest::Error),
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("deserialize json")]
    Json(#[from] serde_json::Error),
    #[error("parse image")]
    Image(#[from] image::ImageError),
    #[error("invalid image format")]
    InvalidImageFormat,
}
