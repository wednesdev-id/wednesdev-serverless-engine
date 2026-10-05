use wednes_sdk::{export, Guest, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        match req.path.as_str() {
            "/cpu" => {
                let mut acc: u64 = 0;
                for i in 0..10_000 {
                    acc = acc.wrapping_add(i);
                }
                Response::json(200, format!(r#"{{"result":{}}}"#, acc))
            }
            "/memory" => {
                let mut data = Vec::with_capacity(1024 * 64);
                data.resize(1024 * 64, 42u8);
                Response::json(200, format!(r#"{{"allocated":{}}}"#, data.len()))
            }
            "/large-response" => {
                let body = vec![b'a'; 1024 * 128];
                Response::bytes(200, "application/json", body)
            }
            "/trap" => panic!("explicit guest trap"),
            _ => Response::json(200, r#"{"ok":true}"#),
        }
    }
}

export!(Component);
