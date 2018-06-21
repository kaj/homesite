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
extern crate env_logger;

mod ructe_response;

use gotham::http::response::create_response;
use gotham::router::builder::{
    build_simple_router, DefineSingleRoute, DrawRoutes,
};
use gotham::router::Router;
use gotham::state::{FromState, State};
use hyper::{Response, StatusCode};
use ructe_response::RucteResponse;
use templates::*;

fn main() {
    env_logger::init();
    let addr = "127.0.0.1:3000";
    gotham::start(addr, router())
}

pub fn router() -> Router {
    build_simple_router(|route| {
        route.get("/").to(homepage);
        route.get("/gifta").to(married);
        route.get("/robots.txt").to(robots);
        route
            .get("/s/:name")
            .with_path_extractor::<FilePath>()
            .to(static_file);
    })
}

fn homepage(state: State) -> (State, Response) {
    state.html(index)
}

fn married(state: State) -> (State, Response) {
    state.html(gifta)
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
pub struct FilePath {
    pub name: String,
}

fn static_file(state: State) -> (State, Response) {
    let res = {
        let FilePath { ref name } = FilePath::borrow_from(&state);
        if let Some(data) = statics::StaticFile::get(&name) {
            create_response(
                &state,
                StatusCode::Ok,
                Some((data.content.to_vec(), data.mime.clone())),
            )
        } else {
            info!("Static file {} not found", name);
            create_response(&state, StatusCode::NotFound, None)
        }
    };
    (state, res)
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
