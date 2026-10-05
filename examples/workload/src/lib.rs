wit_bindgen::generate!({ world: "function", path: "../../wit" });
use wednes::function::types::Header;

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        let (status, body) = match req.path.as_str() {
            "/cpu" => {
                let mut acc: u64 = 0;
                for i in 0..10_000 {
                    acc = acc.wrapping_add(i);
                }
                (200, format!(r#"{{"result":{}}}"#, acc).into_bytes())
            }
            "/memory" => {
                let mut data = Vec::with_capacity(1024 * 64);
                data.resize(1024 * 64, 42u8);
                (
                    200,
                    format!(r#"{{"allocated":{}}}"#, data.len()).into_bytes(),
                )
            }
            "/large-response" => {
                let body = vec![b'a'; 1024 * 128];
                (200, body)
            }
            "/trap" => panic!("explicit guest trap"),
            _ => (200, br#"{"ok":true}"#.to_vec()),
        };
        Response {
            status,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            body,
        }
    }
}

export!(Component);
