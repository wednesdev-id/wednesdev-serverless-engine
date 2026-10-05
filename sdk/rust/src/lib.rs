pub mod bindings {
    wit_bindgen::generate!({
        world: "function",
        path: "../../wit",
        pub_export_macro: true,
    });
}

#[macro_export]
macro_rules! export {
    ($ty:ident) => {
        $crate::bindings::export!($ty with_types_in $crate::bindings);
    };
}

pub use bindings::wednes::function::types::Header;
pub use bindings::wednes::function::types::Request;
pub use bindings::wednes::function::types::Response;
pub use bindings::Guest;
pub use bindings::Guest as Handler;

impl Request {
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    pub fn text_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.as_str())
    }
}

impl Response {
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    pub fn text(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: "text/plain".to_string(),
            }],
            body: body.into().into_bytes(),
        }
    }

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

    pub fn json_val<T: serde::Serialize>(status: u16, val: &T) -> Result<Self, serde_json::Error> {
        let body = serde_json::to_vec(val)?;
        Ok(Self {
            status,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            body,
        })
    }

    pub fn bytes(status: u16, content_type: impl Into<String>, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: vec![Header {
                name: "content-type".to_string(),
                value: content_type.into(),
            }],
            body: body.into(),
        }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push(Header {
            name: name.into(),
            value: value.into(),
        });
        self
    }
}
