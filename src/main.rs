extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate mime;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use gotham::http::response::create_response;
use gotham::router::builder::{
    build_simple_router, DefineSingleRoute, DrawRoutes,
};
use gotham::router::Router;
use gotham::state::{FromState, State};
use hyper::{Response, StatusCode};
use templates::*;

fn main() {
    let addr = "127.0.0.1:3000";
    println!("Listening on http://{}/", addr);
    gotham::start(addr, router())
}

pub fn router() -> Router {
    build_simple_router(|route| {
        route.get("/").to(homepage);
        route.get("/gifta").to(married);
        route.get("/robots.txt").to(robots);
        route
            .get("/s/:name")
            .with_path_extractor::<FilenameRequestPath>()
            .to(static_file);
    })
}

fn homepage(state: State) -> (State, Response) {
    let mut buf = Vec::new();
    index(&mut buf).expect("render template");
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((buf, "text/html; charset=utf-8".parse().unwrap())),
    );
    (state, res)
}

fn married(state: State) -> (State, Response) {
    let mut buf = Vec::new();
    gifta(&mut buf).expect("render template");
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((buf, "text/html; charset=utf-8".parse().unwrap())),
    );
    (state, res)
}

fn robots(state: State) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((b"".to_vec(), mime::TEXT_PLAIN)),
    );
    (state, res)
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct FilenameRequestPath {
    pub name: String,
}

fn static_file(state: State) -> (State, Response) {
    let res = {
        let params = FilenameRequestPath::borrow_from(&state);
        if let Some(data) = statics::StaticFile::get(&params.name) {
            create_response(
                &state,
                StatusCode::Ok,
                Some((data.content.to_vec(), data.mime.clone())),
            )
        } else {
            println!("Static file {} not found", params.name);
            create_response(
                &state,
                StatusCode::NotFound,
                Some(("not found".as_bytes().to_vec(), mime::TEXT_PLAIN)),
            )
        }
    };
    (state, res)
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
