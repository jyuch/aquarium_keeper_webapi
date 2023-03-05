use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Extension, Router};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};

use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(Mutex::new(State {
        data: HashMap::new(),
    }));

    let app = Router::new()
        .route("/", get(root))
        .route("/-/:path", get(get_data))
        .route("/-/:path", post(post_data))
        .layer(Extension(state));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"))
}

async fn get_data(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
) -> impl IntoResponse {
    let s = state.lock().await;
    let e = &"".to_string();
    let value = s.data.get(&key).unwrap_or(e);
    tracing::debug!("return {} {}", &key, &value);
    value.clone()
}

async fn post_data(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
    body: String,
) -> impl IntoResponse {
    let mut s = state.lock().await;
    s.data.insert(key.clone(), body.clone());
    tracing::debug!("insert {} {}", &key, &body);
    StatusCode::OK
}

struct State {
    data: HashMap<String, String>,
}

type SharedState = Arc<Mutex<State>>;
