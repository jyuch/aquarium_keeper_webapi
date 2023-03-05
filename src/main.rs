use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Extension, Router};
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
        .route("/-/:path", delete(delete_data))
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

    match s.data.get(&key) {
        Some(value) => {
            tracing::debug!("get key: {} value: {}", &key, &value);
            (StatusCode::OK, value.clone())
        }
        None => {
            tracing::debug!("get key: {}, but not found", &key);
            (StatusCode::NOT_FOUND, "".to_string())
        }
    }
}

async fn post_data(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
    body: String,
) -> impl IntoResponse {
    let mut s = state.lock().await;
    s.data.insert(key.clone(), body.clone());
    tracing::debug!("post key: {} value: {}", &key, &body);
    (StatusCode::OK, body.clone())
}

async fn delete_data(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
) -> impl IntoResponse {
    let mut s = state.lock().await;

    match s.data.remove(&key) {
        Some(value) => {
            tracing::debug!("delete key: {} value: {}", &key, &value);
            (StatusCode::OK, value)
        }
        None => {
            tracing::debug!("delete key: {}, but not found", &key);
            (StatusCode::NOT_FOUND, "".to_string())
        }
    }
}

struct State {
    data: HashMap<String, String>,
}

type SharedState = Arc<Mutex<State>>;
