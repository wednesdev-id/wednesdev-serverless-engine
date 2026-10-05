use wednes_sdk::{export, Guest, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        let payload_len = req.body.len();
        let body = format!(
            r#"{{"received":true,"method":"{}","payload_bytes":{}}}"#,
            req.method, payload_len
        );
        Response::json(200, body)
    }
}

export!(Component);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ack_returns_200() {
        let result = Component::handle(Request {
            method: "POST".to_string(),
            path: "/webhook".to_string(),
            headers: vec![],
            body: br#"{"event":"ping"}"#.to_vec(),
        });
        assert_eq!(result.status, 200);
        let body = String::from_utf8(result.body).unwrap();
        assert!(body.contains("received"));
    }
}
