use wednes_sdk::{export, Guest, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        match req.json::<serde_json::Value>() {
            Ok(v) => {
                let out = serde_json::json!({ "echo": v, "engine": "wednes" });
                Response::json(200, out.to_string())
            }
            Err(_) => Response::json(400, r#"{"error":"invalid json"}"#),
        }
    }
}

fn transform(body: &[u8]) -> (u16, Vec<u8>) {
    let req = Request {
        method: "POST".to_string(),
        path: "/".to_string(),
        headers: vec![],
        body: body.to_vec(),
    };
    let res = Component::handle(req);
    (res.status, res.body)
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
