use std::convert::Infallible;

use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::server::AppState;

pub async fn stream_logs(State(state): State<AppState>) -> impl IntoResponse {
    let rx = state.log_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|res| {
        res.ok().and_then(|log| {
            serde_json::to_string(&log)
                .ok()
                .map(|data| Ok::<Event, Infallible>(Event::default().data(data)))
        })
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
