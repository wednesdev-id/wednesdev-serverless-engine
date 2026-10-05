wit_bindgen::generate!({
    world: "function",
    path: "../../wit",
});

struct Component;

impl Guest for Component {
    fn handle(_req: Request) -> Response {
        let mut vecs = Vec::new();
        // Allocate chunks of 10 MB until it fails (32 MB limit)
        for i in 0..10 {
            let chunk = vec![i as u8; 10 * 1024 * 1024];
            vecs.push(chunk);
        }

        let body = format!("OOM test survived with {} buffers", vecs.len());
        Response {
            status: 200,
            headers: vec![],
            body: body.into_bytes(),
        }
    }
}

export!(Component);
