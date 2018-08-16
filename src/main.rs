extern crate warp;
extern crate log;
extern crate mime;
extern crate env_logger;
extern crate chrono;

mod render_ructe;

use render_ructe::RenderRucte;
use chrono::{Utc, Duration};
use templates::statics::StaticFile;
use warp::http::{Response, StatusCode};
use warp::{path, reject, Filter, Rejection, Reply};

fn main() {
    env_logger::init();

    let router = warp::get2().and(
        warp::index().and_then(homepage)
            .or(path("gifta").and_then(married))
            .or(path("robots.txt").map(robots))
            .or(path("s").and(path::param()).and_then(static_file))
    );
    warp::serve(router.recover(customize_error)).run(([127, 0, 0, 1], 3030));
}

fn homepage() -> Result<impl Reply, Rejection> {
    Response::builder().html(templates::index)
}
fn married() -> Result<impl Reply, Rejection> {
    Response::builder().html(templates::gifta)
}
fn robots() -> impl Reply {
    ""
}

/// Handler for static files.
/// Create a response from the file data with a correct content type
/// and a far expires header (or a 404 if the file does not exist).
fn static_file(name: String) -> Result<impl Reply, Rejection> {
    if let Some(data) = StaticFile::get(&name) {
        let far_expires = Utc::now() + Duration::days(180);
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", data.mime.as_ref())
            .header("expires", far_expires.to_rfc2822())
            .body(data.content))
    } else {
        println!("Static file {} not found", name);
        Err(reject::not_found())
    }
}

/// Create custom error pages.
fn customize_error(err: Rejection) -> Result<impl Reply, Rejection> {
    match err.status() {
        StatusCode::NOT_FOUND => {
            eprintln!("Got a 404: {:?}", err);
            // We have a custom 404 page!
            Response::builder().status(StatusCode::NOT_FOUND).html(|o| {
                templates::error(
                    o,
                    StatusCode::NOT_FOUND,
                    "The resource you requested could not be located.",
                )
            })
        }
        code => {
            eprintln!("Got a {}: {:?}", code.as_u16(), err);
            Response::builder()
                .status(code)
                .html(|o| templates::error(o, code, "Something went wrong."))
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
