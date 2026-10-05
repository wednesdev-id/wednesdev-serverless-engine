wit_bindgen::generate!({
    world: "function",
    path: "../../wit",
});

struct Component;

impl Guest for Component {
    fn handle(_req: Request) -> Response {
        let mut count: u64 = 0;
        loop {
            count = count.wrapping_add(1);
            std::hint::black_box(count);
        }
    }
}

export!(Component);
