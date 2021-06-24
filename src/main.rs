pub use config::*;

mod api;
mod cache;
mod config;
mod minecraft;
mod render;
mod skin;
mod web;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = config::load();

    let api = api::Api::new(config.clone());

    web::run(api, config).await;
}
