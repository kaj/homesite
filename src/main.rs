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

    tree_builder.add_route(static_route(vec![Method::Get, Method::Head],
                                        || Ok(homepage),
                                        (global, ()),
                                        ps.clone()));

    let mut stat = NodeBuilder::new("s", SegmentType::Static);
    let mut name = NodeBuilder::new("name", SegmentType::Dynamic);
    name.add_route(challenge_route(vec![Method::Get, Method::Head],
                                   || Ok(static_file),
                                   (global, ()),
                                   ps.clone()));
    stat.add_child(name);
    tree_builder.add_child(stat);

    let mut gifta = NodeBuilder::new("gifta", SegmentType::Static);
    gifta.add_route(static_route(vec![Method::Get, Method::Head],
                                 || Ok(married),
                                 (global, ()),
                                 ps.clone()));
    tree_builder.add_child(gifta);

    let mut robots_txt = NodeBuilder::new("robots.txt", SegmentType::Static);
    robots_txt.add_route(static_route(vec![Method::Get, Method::Head],
                                 || Ok(robots),
                                 (global, ()),
                                 ps.clone()));
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
        Some((
            buf,
            "text/html; charset=utf-8".parse().unwrap(),
            )),
        );
    (state, res)
}


fn married(state: State, _: Request) -> (State, Response) {
    let mut buf = Vec::new();
    gifta(&mut buf).expect("render template");
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((
            buf,
            "text/html; charset=utf-8".parse().unwrap(),
            )),
        );
    (state, res)
}

fn robots(state: State, _: Request) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((
            b"".to_vec(),
            mime::TEXT_PLAIN,
            )),
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
    let matcher = MethodOnlyRouteMatcher::new(methods);
    let dispatcher = DispatcherImpl::new(new_handler, active_pipelines, pipeline_set);

    // Note the Route isn't simply not caring about the Request path and Query string. It will
    // extract data from both, in a type safe way and safely deposit it into a instance of the
    // structs shown below, ready for use by Middleware and Handlers (Usually just your handler,
    // which is a function in your controller).
    let extractors: Extractors<FilenameRequestPath, NoopQueryStringExtractor> = Extractors::new();
    let route = RouteImpl::new(
        matcher,
        Box::new(dispatcher),
        extractors,
        Delegation::Internal,
    );
    Box::new(route)
}

fn static_file(state: State, _req: Request) -> (State, Response) {
    let res = {
        let params = FilenameRequestPath::borrow_from(&state);
        if let Some(data) = statics::StaticFile::get(&params.name) {
            create_response(&state, StatusCode::Ok,
                            Some((data.content.to_vec(), data.mime.clone())))
        } else {
            println!("Static file {} not found", params.name);
            create_response(&state, StatusCode::NotFound,
                            Some(("not found".as_bytes().to_vec(), mime::TEXT_PLAIN)))
        }
    };
    (state, res)
}



fn static_route<NH, P, C>(
    methods: Vec<Method>,
    new_handler: NH,
    active_pipelines: C,
    ps: PipelineSet<P>,
) -> Box<Route + Send + Sync>
where
    NH: NewHandler + 'static,
    C: PipelineHandleChain<P> + Send + Sync + 'static,
    P: Send + Sync + 'static,
{
    // Requests must have used the specified method(s) in order for this Route to match.
    //
    // You could define your on RouteMatcher of course.. perhaps you'd like to only match on
    // requests that are made using the GET method and send a User-Agent header for a particular
    // version of browser you'd like to make fun of....
    let matcher = MethodOnlyRouteMatcher::new(methods);

    // For Requests that match this Route we'll dispatch them to new_handler via the pipelines
    // defined in active_pipelines.
    //
    // n.b. We also specify the set of all known pipelines in the application so the dispatcher can
    // resolve the pipeline references provided in active_pipelines. For this application that is
    // only the global pipeline.
    let dispatcher = DispatcherImpl::new(new_handler, active_pipelines, ps);
    let extractors: Extractors<NoopPathExtractor, NoopQueryStringExtractor> = Extractors::new();
    let route = RouteImpl::new(
        matcher,
        Box::new(dispatcher),
        extractors,
        Delegation::Internal,
    );
    Box::new(route)
}

include!(concat!(env!("OUT_DIR"), "/templates.rs"));
