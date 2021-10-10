use actix_web::{
    http::{header, StatusCode},
    Error, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use futures_util::future::{ready, Ready};
use rust_embed::RustEmbed;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::io;
use std::path::Path;

type EtagMap = HashMap<&'static str, BTreeMap<String, u64>>;

// ETags of resource in RustEmbed classes should never be changed since resources be embeded into the binary.
// To avoid repeatable calculate ETag, make a Pool to store these constant etag value.
// Use thread_local to avoid lock acquires between threads.
thread_local! {
    static ETAG: RefCell<EtagMap> = init();
}

fn init() -> RefCell<EtagMap> {
    RefCell::new(EtagMap::new())
}

fn get_etag<E>(filename: &str) -> Option<u64>
where
    E: RustEmbed,
{
    let filename = filename.to_string();
    let typename = std::any::type_name::<E>();
    ETAG.with(|m| {
        if let Some(map) = m.borrow().get(typename) {
            return map.get(&filename).map(|u| *u);
        }
        let map = init_etag::<E>();
        let r = map.get(&filename).map(|u| *u);
        m.borrow_mut().insert(typename, map);
        r
    })
}

fn init_etag<E>() -> BTreeMap<String, u64>
where
    E: RustEmbed,
{
    let mut map = BTreeMap::new();
    for file in E::iter() {
        let file = file.as_ref();
        let etag = match E::get(file).and_then(|c| Some(fxhash::hash64(&c))) {
            Some(etag) => etag,
            None => continue,
        };
        map.insert(file.into(), etag);
    }
    map
}

/// Returns true if `req` doesn't have an `If-None-Match` header matching `req`.
fn none_match(etag: Option<&header::EntityTag>, req: &HttpRequest) -> bool {
    match req.get_header::<header::IfNoneMatch>() {
        Some(header::IfNoneMatch::Any) => false,

        Some(header::IfNoneMatch::Items(ref items)) => {
            if let Some(some_etag) = etag {
                for item in items {
                    if item.weak_eq(some_etag) {
                        return false;
                    }
                }
            }

            true
        }

        None => true,
    }
}

fn io_not_found<S>(info: S) -> io::Error
where
    S: AsRef<str>,
{
    io::Error::new(io::ErrorKind::NotFound, info.as_ref())
}

pub struct EmbedFile {
    content: Vec<u8>,
    content_type: mime::Mime,
    etag: Option<header::EntityTag>,
}

impl EmbedFile {
    pub fn open<E, P>(_: &E, path: P) -> io::Result<EmbedFile>
    where
        E: RustEmbed,
        P: AsRef<Path>,
    {
        let mut path = path.as_ref();
        while let Ok(new_path) = path.strip_prefix(".") {
            path = new_path;
        }
        Self::open_impl::<E>(&path).ok_or(io_not_found("File not found"))
    }

    fn open_impl<E>(path: &Path) -> Option<EmbedFile>
    where
        E: RustEmbed,
    {
        let content_type = mime_guess::from_path(&path).first_or_octet_stream();
        let filename = path.to_str()?;
        let etag = get_etag::<E>(filename);
        let r = EmbedFile {
            content: E::get(filename)?.into_owned(),
            content_type,
            etag: etag.map(|etag| header::EntityTag::strong(format!("{:x}", etag))),
        };
        Some(r)
    }

    fn into_response(self, req: &HttpRequest) -> HttpResponse {
        let status_code = if !none_match(self.etag.as_ref(), req) {
            StatusCode::NOT_MODIFIED
        } else {
            StatusCode::OK
        };

        HttpResponse::build(status_code)
            .set(header::ContentType(self.content_type))
            .if_some(self.etag, |etag, resp| {
                resp.set(header::ETag(etag));
            })
            .body(self.content)
    }
}

impl Responder for EmbedFile {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        ready(Ok(self.into_response(req)))
    }
}
