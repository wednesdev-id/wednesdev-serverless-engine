use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
    routing::any,
    Router,
};
use std::sync::Arc;
use wednes_core::{Header, Request};
use wednes_runtime::RuntimeBackend;

pub struct GatewayState {
    pub runtime: Arc<dyn RuntimeBackend>,
}

pub fn create_router(runtime: Arc<dyn RuntimeBackend>) -> Router {
    let state = Arc::new(GatewayState { runtime });

    Router::new()
        .route("/fn/:id", any(handle_function_root))
        .route("/fn/:id/*path", any(handle_function_subpath))
        .with_state(state)
}

async fn handle_function_root(
    State(state): State<Arc<GatewayState>>,
    Path(id): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> AxumResponse {
    dispatch(state, id, method, "/".to_string(), headers, body).await
}

async fn handle_function_subpath(
    State(state): State<Arc<GatewayState>>,
    Path((id, path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> AxumResponse {
    let path = format!("/{}", path);
    dispatch(state, id, method, path, headers, body).await
}

async fn dispatch(
    state: Arc<GatewayState>,
    id: String,
    method: Method,
    path: String,
    headers: HeaderMap,
    body: Bytes,
) -> AxumResponse {
    let mut req_headers = Vec::new();
    for (k, v) in headers.iter() {
        if let Ok(v_str) = v.to_str() {
            req_headers.push(Header {
                name: k.as_str().to_string(),
                value: v_str.to_string(),
            });
        }
    }

    let req = Request {
        method: method.as_str().to_string(),
        path,
        headers: req_headers,
        body: body.to_vec(),
    };

    match state.runtime.execute(&id, req).await {
        Ok(res) => {
            let mut builder = axum::http::Response::builder()
                .status(StatusCode::from_u16(res.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR));
            
            for h in res.headers {
                builder = builder.header(h.name, h.value);
            }

            builder.body(axum::body::Body::from(res.body)).unwrap_or_else(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response").into_response()
            })
        }
        Err(err) => {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Invocation error: {err}")).into_response()
        }
    }
}
