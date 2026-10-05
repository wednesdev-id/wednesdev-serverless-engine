wit_bindgen::generate!({ world: "function", path: "../../wit" });
use wednes::function::types::Header;

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        let (status, body) = transform(&req.body);
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

fn transform(body: &[u8]) -> (u16, Vec<u8>) {
    match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(v) => {
            let out = serde_json::json!({ "echo": v, "engine": "wednes" });
            (200, serde_json::to_vec(&out).unwrap())
        }
        Err(_) => (400, br#"{"error":"invalid json"}"#.to_vec()),
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    #[test]
    fn echo_valid_json() {
        let (status, body) = super::transform(br#"{"hello":1}"#);
        assert_eq!(status, 200);
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["echo"]["hello"], 1);
        assert_eq!(v["engine"], "wednes");
    }
    #[test]
    fn reject_invalid_json() {
        assert_eq!(super::transform(b"invalid").0, 400);
    }
}
