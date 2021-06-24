use std::net::SocketAddr;

use uuid::Uuid;
use warp::Filter;
use warp::http::StatusCode;
use warp::reply;

use crate::api::Api;
use crate::Config;

pub async fn run(api: Api, config: Config) {
    let cors = warp::cors()
        .allow_any_origin();

    let face = warp::path("face")
        .and(warp::addr::remote())
        .and(warp::path::param::<u32>())
        .and(warp::path::param::<Uuid>())
        .and_then({
            let api = api.clone();
            move |addr, size, uuid| get_face(api.clone(), addr, size, uuid)
        });

    warp::serve(face.with(cors))
        .run(([127, 0, 0, 1], config.port))
        .await;
}

async fn get_face(api: Api, addr: Option<SocketAddr>, size: u32, uuid: Uuid) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    log::debug!("receiving face request for {0} ({1}x{1}) from {2:?}", uuid, size, addr);

    let api = match api.try_access(addr.as_ref()) {
        Some(api) => api,
        None => return Ok(Box::new(StatusCode::TOO_MANY_REQUESTS)),
    };

    let scale = match parse_scale(size) {
        Some(scale) => scale,
        None => return Ok(Box::new(StatusCode::BAD_REQUEST)),
    };

    match api.get_face(uuid, scale).await {
        Ok(face) => Ok(Box::new(face)),
        Err(err) => {
            log::error!("internal server error: {:?}", err);
            Ok(Box::new(StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}

#[inline]
fn parse_scale(size: u32) -> Option<u32> {
    if size % 8 == 0 && size >= 8 && size <= 256 {
        log2(size / 8)
    } else {
        return None;
    }
}

#[inline]
fn log2(value: u32) -> Option<u32> {
    if value > 0 && value.is_power_of_two() {
        Some((u32::BITS - 1) - value.leading_zeros())
    } else {
        None
    }
}
