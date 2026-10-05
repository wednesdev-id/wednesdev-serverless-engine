wit_bindgen::generate!({ world: "function", path: "../../wit" });
use wednes::function::types::Header;
struct Component;
impl Guest for Component {
    fn handle(req: Request) -> Response {
        let payload_len = req.body.len();
        let body = format!(
            r#"{{"received":true,"method":"{}","payload_bytes":{}}}"#,
            req.method, payload_len
        );
        Response {
            status: 200,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            body: body.into_bytes(),
        }
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
