use crate::static_files::EmbedFile;
use actix_web::{web, Error};
use rust_embed::RustEmbed;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "static/core/"]
struct CoreStatics;

pub(crate) async fn favicon() -> Result<EmbedFile, Error> {
    Ok(EmbedFile::open::<CoreStatics, _>(PathBuf::from(
        "favicon.ico",
    ))?)
}

pub fn register_endpoints(_cfg: &mut web::ServiceConfig) {}
