use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
    routing::any,
    Router,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize},
    Arc,
};
use std::time::Instant;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use wednes_core::{Header, Request};
use wednes_runtime::RuntimeBackend;
use wednes_telemetry::Telemetry;

#[derive(Clone, Debug)]
pub struct FnConfig {
    pub concurrency: usize,
    pub memory_mb: usize,
}

pub struct Scheduler {
    global_sem: Arc<Semaphore>,
    memory_sem: Arc<Semaphore>,
    fn_configs: HashMap<String, FnConfig>,
    fn_sems: HashMap<String, Arc<Semaphore>>,
    unknown_sem: Arc<Semaphore>,

    pub telemetry: Telemetry,
    pub active_invocations: Arc<AtomicUsize>,
    pub total_invocations: Arc<AtomicUsize>,
    pub traps: Arc<AtomicUsize>,
    pub rejected: Arc<AtomicUsize>,
    pub total_duration_ms: Arc<AtomicU64>,
}

pub struct ExecutionPermits {
    _global: OwnedSemaphorePermit,
    _function: OwnedSemaphorePermit,
    _memory: OwnedSemaphorePermit,
}

impl Scheduler {
    pub fn new(
        global_limit: usize,
        memory_budget_mb: usize,
        fn_configs: HashMap<String, FnConfig>,
        default_fn_concurrency: usize,
    ) -> Self {
        let mut fn_sems = HashMap::new();
        for (id, cfg) in &fn_configs {
            fn_sems.insert(id.clone(), Arc::new(Semaphore::new(cfg.concurrency)));
        }

        let telemetry = Telemetry::new();

        Self {
            global_sem: Arc::new(Semaphore::new(global_limit)),
            memory_sem: Arc::new(Semaphore::new(memory_budget_mb)),
            fn_configs,
            fn_sems,
            unknown_sem: Arc::new(Semaphore::new(default_fn_concurrency)),
            active_invocations: telemetry.active_invocations.clone(),
            total_invocations: telemetry.total_invocations.clone(),
            traps: telemetry.traps.clone(),
            rejected: telemetry.rejected.clone(),
            total_duration_ms: telemetry.total_duration_ms.clone(),
            telemetry,
        }
    }

    pub fn try_acquire(&self, fn_id: &str) -> Result<ExecutionPermits, ()> {
        let mem_req = self
            .fn_configs
            .get(fn_id)
            .map(|c| c.memory_mb)
            .unwrap_or(32);

        let memory = self
            .memory_sem
            .clone()
            .try_acquire_many_owned(mem_req as u32)
            .map_err(|_| ())?;

        let global = match self.global_sem.clone().try_acquire_owned() {
            Ok(g) => g,
            Err(_) => {
                return Err(());
            }
        };

        let fn_sem = self.fn_sems.get(fn_id).unwrap_or(&self.unknown_sem).clone();

        let fn_permit = match fn_sem.try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                return Err(());
            }
        };

        Ok(ExecutionPermits {
            _global: global,
            _function: fn_permit,
            _memory: memory,
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
        .route("/metrics", axum::routing::get(handle_metrics))
        .route("/fn/:id", any(handle_function_root))
        .route("/fn/:id/*path", any(handle_function_subpath))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
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
    let start = Instant::now();
    state.scheduler.telemetry.record_start();

    let _permits = match state.scheduler.try_acquire(&id) {
        Ok(p) => p,
        Err(_) => {
            state.scheduler.telemetry.record_reject(&id, start);
            return (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests").into_response();
        }
    };

    state.scheduler.telemetry.record_accept();

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
            state
                .scheduler
                .telemetry
                .record_complete(&id, res.status, start);

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
        Err(err) => {
            state.scheduler.telemetry.record_trap(&id, &err, start);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invocation error: {err}"),
            )
                .into_response()
        }
    }
}

async fn handle_metrics(State(state): State<Arc<GatewayState>>) -> String {
    state.scheduler.telemetry.prometheus_metrics()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_limits_per_function_and_releases() {
        let mut fn_configs = HashMap::new();
        fn_configs.insert(
            "fn1".to_string(),
            FnConfig {
                concurrency: 2,
                memory_mb: 32,
            },
        );
        let scheduler = Scheduler::new(10, 1200, fn_configs, 5);

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
        let scheduler = Scheduler::new(2, 1200, HashMap::new(), 10);

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

    #[test]
    fn scheduler_memory_limit() {
        let mut fn_configs = HashMap::new();
        fn_configs.insert(
            "huge".to_string(),
            FnConfig {
                concurrency: 10,
                memory_mb: 1000,
            },
        );
        let scheduler = Scheduler::new(10, 1200, fn_configs, 5);

        let p1 = scheduler.try_acquire("huge");
        assert!(p1.is_ok());

        let p2 = scheduler.try_acquire("huge");
        assert!(p2.is_err()); // 1000 + 1000 > 1200

        // Release p1
        drop(p1);
        let p3 = scheduler.try_acquire("huge");
        assert!(p3.is_ok());
    }
}
mod gateway_tests;
