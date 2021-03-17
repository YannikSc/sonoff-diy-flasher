use crate::file_range::{read_file_ranges, RangedFileResult};
use crate::range::parse_range_header;
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
use std::path::PathBuf;
use std::pin::Pin;

#[derive(Clone, NewMiddleware)]
pub struct ServerResources {
    path: PathBuf,
}

#[derive(Clone, StateData)]
pub struct ServerResourcesData {
    path: PathBuf,
}

pub fn handle_firmware_get(state: State) -> (State, impl IntoResponse) {
    let data = ServerResourcesData::borrow_from(&state).clone();
    let headers = HeaderMap::borrow_from(&state);

    if let Some(range) = headers.get("Range") {
        let range = range.to_str().unwrap_or("");
        let range = parse_range_header(String::from(range));

        return (state, read_file_ranges(range, data.path.clone()));
    }

    (state, RangedFileResult::Multiple(vec![]))
}

pub fn router(path: PathBuf) -> Router {
    let (chain, pipelines) = single_pipeline(new_pipeline().add(ServerResources { path }).build());

    build_router(chain, pipelines, |route| {
        route.get("firmware.bin").to(handle_firmware_get);
    })
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
