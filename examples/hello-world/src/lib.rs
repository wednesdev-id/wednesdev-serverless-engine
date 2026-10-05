use wednes_sdk::{export, Guest, Request, Response};

struct Component;

impl Guest for Component {
    fn handle(req: Request) -> Response {
        let body = format!(
            r#"{{"message":"hello from wednes engine","path":"{}"}}"#,
            req.path
        );
        Response::json(200, body)
    }
}

export!(Component);
