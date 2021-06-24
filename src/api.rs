use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use governor::clock::DefaultClock;
use governor::RateLimiter;
use governor::state::keyed::DashMapStateStore;
use image::{EncodableLayout, RgbImage};
use image::codecs::png::PngEncoder;
use uuid::Uuid;
use warp::http::{header, HeaderValue};

use crate::{Config, minecraft};
use crate::cache::Cache;
use crate::render;
use crate::skin::{self, Skin};
use sha1::Sha1;

const CACHE_CLEAR_INTERVAL: Duration = Duration::from_secs(60 * 60 * 24);

struct Caches {
    raw_faces: Cache<Uuid, Arc<RgbImage>>,
    faces: Cache<(Uuid, u32), ImageBytes>,
}

impl Caches {
    fn new() -> Caches {
        Caches {
            raw_faces: Cache::new(512),
            faces: Cache::new(128),
        }
    }

    async fn clear(&self) {
        self.raw_faces.clear().await;
        self.faces.clear().await;
    }
}

#[derive(Clone)]
pub struct Api {
    caches: Arc<Caches>,
    rate_limiter: Arc<RateLimiter<SocketAddr, DashMapStateStore<SocketAddr>, DefaultClock>>,
}

impl Api {
    pub fn new(config: Config) -> Api {
        let caches = Arc::new(Caches::new());

        let caches_weak = Arc::downgrade(&caches);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(CACHE_CLEAR_INTERVAL);
            loop {
                interval.tick().await;
                if let Some(caches) = caches_weak.upgrade() {
                    caches.clear().await;
                } else {
                    break;
                }
            }
        });

        let quota = governor::Quota::per_minute(NonZeroU32::new(config.requests_per_minute).unwrap());
        let rate_limiter = RateLimiter::dashmap(quota);
        let rate_limiter = Arc::new(rate_limiter);

        Api { caches, rate_limiter }
    }

    pub fn try_access(&self, addr: Option<&SocketAddr>) -> Option<ApiAccess> {
        if let Some(addr) = addr {
            if let Err(_) = self.rate_limiter.check_key(addr) {
                return None;
            }
        }

        Some(ApiAccess { caches: self.caches.clone() })
    }
}

#[derive(Clone)]
pub struct ApiAccess {
    caches: Arc<Caches>,
}

impl ApiAccess {
    #[inline]
    pub async fn get_face(&self, uuid: Uuid, scale: u32) -> Result<ImageBytes> {
        get_face(self.clone(), uuid, scale).await
    }
}

async fn get_face(api: ApiAccess, uuid: Uuid, scale: u32) -> Result<ImageBytes> {
    let caches = api.caches.clone();
    caches.faces.try_get((uuid, scale), move |(uuid, scale)| load_face(api, uuid, scale)).await
}

async fn get_raw_face(api: ApiAccess, uuid: Uuid) -> Result<Arc<RgbImage>> {
    let caches = api.caches.clone();
    caches.raw_faces.try_get(uuid, load_raw_face).await
}

async fn load_face(api: ApiAccess, uuid: Uuid, scale: u32) -> Result<ImageBytes> {
    let raw_face = get_raw_face(api, uuid).await?;

    tokio::task::spawn_blocking(move || {
        let face = if scale > 0 {
            render::rescale(&*raw_face, scale)
        } else {
            (*raw_face).clone()
        };

        Ok(encode_image(face)?)
    }).await.unwrap()
}

async fn load_raw_face(uuid: Uuid) -> Result<Arc<RgbImage>> {
    let skin = get_skin(uuid).await?.unwrap_or_else(|| {
        let default = skin::DefaultSkin::from(uuid);
        default.as_skin().clone()
    });

    Ok(tokio::task::spawn_blocking(move || {
        let image = render::render_face(&skin);
        Arc::new(image)
    }).await.unwrap())
}

async fn get_skin(uuid: Uuid) -> Result<Option<Skin>> {
    let skin = minecraft::get_profile(uuid).await?
        .and_then(|profile| profile.textures())
        .and_then(|textures| textures.refs.skin);

    if let Some(skin) = skin {
        let skin = minecraft::get_texture(skin).await?;
        Ok(Skin::from(skin))
    } else {
        Ok(None)
    }
}

fn encode_image(face: RgbImage) -> Result<ImageBytes> {
    let mut bytes = Vec::new();

    let encoder = PngEncoder::new(&mut bytes);
    encoder.encode(face.as_bytes(), face.width(), face.height(), image::ColorType::Rgb8)?;

    Ok(ImageBytes::from(Bytes::from(bytes)))
}

#[derive(Clone)]
pub struct ImageBytes {
    bytes: Bytes,
    etag: String,
}

impl ImageBytes {
    #[inline]
    pub fn matches(&self, etag: Option<String>) -> bool {
        match etag {
            Some(etag) => self.etag == etag,
            None => false,
        }
    }
}

impl From<Bytes> for ImageBytes {
    fn from(bytes: Bytes) -> Self {
        let mut sha1 = Sha1::new();
        sha1.update(bytes.as_ref());
        let sha1 = sha1.digest();

        let etag = base64::encode_config(sha1.bytes(), base64::URL_SAFE_NO_PAD);
        ImageBytes { bytes, etag }
    }
}

const CACHE_MAX_AGE: usize = 60 * 60 * 24;

impl warp::Reply for ImageBytes {
    fn into_response(self) -> warp::reply::Response {
        let mut response = warp::reply::Response::new(self.bytes.into());

        let headers = response.headers_mut();
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
        headers.insert(header::ETAG, HeaderValue::from_str(&self.etag).unwrap());
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_str(&format!("public, max-age={}, stale-while-revalidate", CACHE_MAX_AGE)).unwrap());

        response
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, thiserror::Error)]
pub enum Error {
    #[error("failed to encode image")]
    EncodeImage,
    #[error("minecraft api gave error")]
    MinecraftApi,
}

impl From<image::ImageError> for Error {
    fn from(_: image::ImageError) -> Self {
        Error::EncodeImage
    }
}

impl From<minecraft::Error> for Error {
    fn from(_: minecraft::Error) -> Self {
        Error::MinecraftApi
    }
}

