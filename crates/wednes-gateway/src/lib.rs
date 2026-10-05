use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
    routing::any,
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use wednes_core::{Header, Request};
use wednes_runtime::RuntimeBackend;

pub struct Scheduler {
    global_sem: Arc<Semaphore>,
    fn_limits: HashMap<String, usize>,
    default_fn_concurrency: usize,
    fn_sems: Mutex<HashMap<String, Arc<Semaphore>>>,
}

pub struct ExecutionPermits {
    _global: OwnedSemaphorePermit,
    _function: OwnedSemaphorePermit,
}

impl Scheduler {
    pub fn new(
        global_limit: usize,
        fn_limits: HashMap<String, usize>,
        default_fn_concurrency: usize,
    ) -> Self {
        Self {
            global_sem: Arc::new(Semaphore::new(global_limit)),
            fn_limits,
            default_fn_concurrency,
            fn_sems: Mutex::new(HashMap::new()),
        }
    }

    pub fn try_acquire(&self, fn_id: &str) -> Result<ExecutionPermits, ()> {
        let global = self
            .global_sem
            .clone()
            .try_acquire_owned()
            .map_err(|_| ())?;

        let fn_sem = {
            let mut sems = self.fn_sems.lock().unwrap();
            sems.entry(fn_id.to_string())
                .or_insert_with(|| {
                    let limit = self
                        .fn_limits
                        .get(fn_id)
                        .copied()
                        .unwrap_or(self.default_fn_concurrency);
                    Arc::new(Semaphore::new(limit))
                })
                .clone()
        };

        let fn_permit = fn_sem.try_acquire_owned().map_err(|_| ())?;

        Ok(ExecutionPermits {
            _global: global,
            _function: fn_permit,
        })
    }
}

pub struct GatewayState {
    pub runtime: Arc<dyn RuntimeBackend>,
    pub scheduler: Arc<Scheduler>,
}

pub fn create_router(runtime: Arc<dyn RuntimeBackend>, scheduler: Arc<Scheduler>) -> Router {
    let state = Arc::new(GatewayState { runtime, scheduler });

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
    let _permits = match state.scheduler.try_acquire(&id) {
        Ok(p) => p,
        Err(_) => {
            return (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests").into_response();
        }
    };

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
            let mut builder = axum::http::Response::builder().status(
                StatusCode::from_u16(res.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            );

            for h in res.headers {
                builder = builder.header(h.name, h.value);
            }

            builder
                .body(axum::body::Body::from(res.body))
                .unwrap_or_else(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to build response",
                    )
                        .into_response()
                })
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Invocation error: {err}"),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_limits_per_function_and_releases() {
        let mut fn_limits = HashMap::new();
        fn_limits.insert("fn1".to_string(), 2);
        let scheduler = Scheduler::new(10, fn_limits, 5);

        // First two should succeed
        let p1 = scheduler.try_acquire("fn1");
        assert!(p1.is_ok());
        let p2 = scheduler.try_acquire("fn1");
        assert!(p2.is_ok());

        // Third should fail (per-function exhausted)
        let p3 = scheduler.try_acquire("fn1");
        assert!(p3.is_err());

        // Other function with default limit should succeed
        let other = scheduler.try_acquire("fn2");
        assert!(other.is_ok());

        // Dropping p1 allows another acquire
        drop(p1);
        let p4 = scheduler.try_acquire("fn1");
        assert!(p4.is_ok());
    }

    #[test]
    fn scheduler_global_limit() {
        let scheduler = Scheduler::new(2, HashMap::new(), 10);

        let p1 = scheduler.try_acquire("a");
        assert!(p1.is_ok());
        let p2 = scheduler.try_acquire("b");
        assert!(p2.is_ok());

        // Global limit of 2 reached
        let p3 = scheduler.try_acquire("c");
        assert!(p3.is_err());

        drop(p2);
        let p4 = scheduler.try_acquire("c");
        assert!(p4.is_ok());
    }
}
