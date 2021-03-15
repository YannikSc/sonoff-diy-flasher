#[macro_use]
extern crate gotham_derive;

use file_range::{read_file_ranges, RangedFileResult};
use gotham::{
    handler::{HandlerFuture, IntoResponse},
    hyper::HeaderMap,
    middleware::Middleware,
    pipeline::{new_pipeline, single::single_pipeline},
    router::{
        builder::{build_router, DefineSingleRoute, DrawRoutes},
        Router,
    },
    state::{FromState, State},
};
use range::parse_range_header;
use std::pin::Pin;
use std::{env::current_dir, io::stdin, path::PathBuf};

mod file_range;
mod range;

#[derive(Clone, NewMiddleware)]
struct ServerResources {
    path: PathBuf,
}

impl Middleware for ServerResources {
    fn call<Chain>(self, mut state: State, chain: Chain) -> Pin<Box<HandlerFuture>>
    where
        Chain: FnOnce(State) -> Pin<Box<HandlerFuture>>,
    {
        state.put(ServerResourcesData {
            path: self.path.clone(),
        });

        chain(state)
    }
}

#[derive(Clone, StateData)]
struct ServerResourcesData {
    path: PathBuf,
}

fn handle_firmware_get(state: State) -> (State, impl IntoResponse) {
    let data = ServerResourcesData::borrow_from(&state).clone();
    let headers = HeaderMap::borrow_from(&state);

    if let Some(range) = headers.get("Range") {
        let range = range.to_str().unwrap_or("");
        let range = parse_range_header(String::from(range));

        return (state, read_file_ranges(range, data.path.clone()));
    }

    (state, RangedFileResult::Multiple(vec![]))
}

fn router(path: PathBuf) -> Router {
    let (chain, pipelines) = single_pipeline(new_pipeline().add(ServerResources { path }).build());

    build_router(chain, pipelines, |route| {
        route.get("firmware.bin").to(handle_firmware_get);
    })
}

fn main() {
    let bind_address = "127.0.0.1:8001";

    gotham::start(
        bind_address,
        router(current_dir().unwrap().join("./firmware.bin")),
    );

    let mut buf = String::new();
    stdin().read_line(&mut buf).unwrap();
}
