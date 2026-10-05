#[cfg(test)]
mod tests {
    use super::super::*;
    use async_trait::async_trait;
    use axum::http::StatusCode;
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;
    use wednes_core::{Request, Response};
    use wednes_runtime::RuntimeBackend;

    struct MockBackend;

    #[async_trait]
    impl RuntimeBackend for MockBackend {
        fn init() -> anyhow::Result<Self> {
            Ok(Self)
        }
        fn load_module(&mut self, _id: &str, _wasm_path: &Path) -> anyhow::Result<()> {
            Ok(())
        }
        fn load_manifest(
            &mut self,
            _manifest: &wednes_core::manifest::Manifest,
            _wasm_path: &Path,
        ) -> anyhow::Result<()> {
            Ok(())
        }
        fn precompile(&mut self, _id: &str, _wasm_path: &Path) -> anyhow::Result<()> {
            Ok(())
        }
        fn load_precompiled(&mut self, _id: &str) -> anyhow::Result<()> {
            Ok(())
        }
        async fn execute(&self, _id: &str, _req: Request) -> anyhow::Result<Response> {
            Ok(Response::ok_json("ok"))
        }
    }

    #[tokio::test]
    async fn limits_request_body_size() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        let scheduler = Arc::new(Scheduler::new(10, 1200, HashMap::new(), 5));
        let backend = Arc::new(MockBackend);
        let app = create_router(backend, scheduler);

        // 2 MiB + 1 byte
        let body = vec![0u8; 2 * 1024 * 1024 + 1];

        let req = Request::builder()
            .uri("/fn/test")
            .method("POST")
            .body(Body::from(body))
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}
