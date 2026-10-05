wit_bindgen::generate!({
    world: "function",
    path: "../../wit",
});

use wednes::function::types::Header;

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        let body = format!(
            r#"{{"message":"hello from wednes engine","path":"{}"}}"#,
            req.path
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
