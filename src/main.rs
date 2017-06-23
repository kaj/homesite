#[macro_use]
extern crate mime;
extern crate iron;
extern crate router;

use iron::prelude::*;
use iron::status;
use router::Router;
use templates::*;

fn main() {
    let mut router = Router::new();
    router.get("/", homepage, "index");
    router.get("/gifta/", married, "gifta");
    router.get("/s/:name", static_file, "static_file");
    let server = Iron::new(router).http("localhost:3000").unwrap();
    println!("Listening on {}.", server.socket);
}

fn homepage(_: &mut Request) -> IronResult<Response> {
    let mut buf = Vec::new();
    index(&mut buf).expect("render template");
    Ok(Response::with((status::Ok, mime!(Text / Html; Charset=Utf8), buf)))
}

fn married(_: &mut Request) -> IronResult<Response> {
    let mut buf = Vec::new();
    gifta(&mut buf).expect("render template");
    Ok(Response::with((status::Ok, mime!(Text / Html; Charset=Utf8), buf)))
}

fn static_file(req: &mut Request) -> IronResult<Response> {
    let router = req.extensions.get::<Router>().expect("router");
    let name = router.find("name").expect("name");
    if let Some(data) = statics::StaticFile::get(name) {
        Ok(Response::with((status::Ok, data.mime(), data.content)))
    } else {
        println!("Static file {} not found", name);
        Ok(Response::with((status::NotFound, mime!(Text / Plain), "not found")))
    }
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
