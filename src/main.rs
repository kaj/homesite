extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate mime;

use gotham::handler::NewHandler;
use gotham::handler::NewHandlerService;
use gotham::http::response::create_response;
use gotham::middleware::pipeline::new_pipeline;
use gotham::router::Router;
use gotham::router::request::path::NoopPathExtractor;
use gotham::router::request::query_string::NoopQueryStringExtractor;
use gotham::router::response::finalizer::ResponseFinalizerBuilder;
use gotham::router::route::{Delegation, Extractors, Route, RouteImpl};
use gotham::router::route::dispatch::{finalize_pipeline_set, new_pipeline_set,
                                      DispatcherImpl, PipelineHandleChain,
                                      PipelineSet};
use gotham::router::route::matcher::MethodOnlyRouteMatcher;
use gotham::router::tree::TreeBuilder;
use gotham::router::tree::node::{NodeBuilder, SegmentType};
use gotham::state::{FromState, State};
use hyper::{Method, Request, Response, StatusCode};
use hyper::server::Http;
use templates::*;

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new()
        .bind(&addr, NewHandlerService::new(router()))
        .unwrap();

    println!("Listening on http://{}/", server.local_addr().unwrap());
    server.run().unwrap();
}

pub fn router() -> Router {
    let mut tree_builder = TreeBuilder::new();

    let ps_builder = new_pipeline_set();
    let (ps_builder, global) = ps_builder.add(new_pipeline().build());
    let ps = finalize_pipeline_set(ps_builder);

    tree_builder.add_route(static_route(
        vec![Method::Get, Method::Head],
        || Ok(homepage),
        (global, ()),
        ps.clone(),
    ));

    let mut stat = NodeBuilder::new("s", SegmentType::Static);
    let mut name = NodeBuilder::new("name", SegmentType::Dynamic);
    name.add_route(challenge_route(
        vec![Method::Get, Method::Head],
        || Ok(static_file),
        (global, ()),
        ps.clone(),
    ));
    stat.add_child(name);
    tree_builder.add_child(stat);

    let mut gifta = NodeBuilder::new("gifta", SegmentType::Static);
    gifta.add_route(static_route(
        vec![Method::Get, Method::Head],
        || Ok(married),
        (global, ()),
        ps.clone(),
    ));
    tree_builder.add_child(gifta);

    let mut robots_txt = NodeBuilder::new("robots.txt", SegmentType::Static);
    robots_txt.add_route(static_route(
        vec![Method::Get, Method::Head],
        || Ok(robots),
        (global, ()),
        ps.clone(),
    ));
    tree_builder.add_child(robots_txt);

    let tree = tree_builder.finalize();

    let response_finalizer_builder = ResponseFinalizerBuilder::new();
    let response_finalizer = response_finalizer_builder.finalize();

    Router::new(tree, response_finalizer)
}

fn homepage(state: State, _: Request) -> (State, Response) {
    let mut buf = Vec::new();
    index(&mut buf).expect("render template");
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((buf, "text/html; charset=utf-8".parse().unwrap())),
    );
    (state, res)
}

fn married(state: State, _: Request) -> (State, Response) {
    let mut buf = Vec::new();
    gifta(&mut buf).expect("render template");
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((buf, "text/html; charset=utf-8".parse().unwrap())),
    );
    (state, res)
}

fn robots(state: State, _: Request) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((b"".to_vec(), mime::TEXT_PLAIN)),
    );
    (state, res)
}

#[derive(StateData, FromState, PathExtractor, StaticResponseExtender)]
pub struct FilenameRequestPath {
    pub name: String,
}

fn challenge_route<NH, P, C>(
    methods: Vec<Method>,
    new_handler: NH,
    active_pipelines: C,
    pipeline_set: PipelineSet<P>,
) -> Box<Route + Send + Sync>
where
    NH: NewHandler + 'static,
    C: PipelineHandleChain<P> + Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    Box::new(RouteImpl::new(
        MethodOnlyRouteMatcher::new(methods),
        Box::new(DispatcherImpl::new(
            new_handler,
            active_pipelines,
            pipeline_set,
        )),
        Extractors::<FilenameRequestPath, NoopQueryStringExtractor>::new(),
        Delegation::Internal,
    ))
}

fn static_route<NH, P, C>(
    methods: Vec<Method>,
    new_handler: NH,
    active_pipelines: C,
    pipeline_set: PipelineSet<P>,
) -> Box<Route + Send + Sync>
where
    NH: NewHandler + 'static,
    C: PipelineHandleChain<P> + Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    Box::new(RouteImpl::new(
        MethodOnlyRouteMatcher::new(methods),
        Box::new(DispatcherImpl::new(
            new_handler,
            active_pipelines,
            pipeline_set,
        )),
        Extractors::<NoopPathExtractor, NoopQueryStringExtractor>::new(),
        Delegation::Internal,
    ))
}

fn static_file(state: State, _req: Request) -> (State, Response) {
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
