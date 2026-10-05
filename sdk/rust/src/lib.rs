pub mod bindings {
    wit_bindgen::generate!({
        world: "function",
        path: "../../wit",
        pub_export_macro: true,
    });
}

pub use bindings::wednes::function::types::Header;
pub use bindings::wednes::function::types::Request;
pub use bindings::wednes::function::types::Response;
pub use bindings::export;

impl Response {
    pub fn json(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            body: body.into().into_bytes(),
        }
    }
}
