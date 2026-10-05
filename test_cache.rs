use wasmtime::Config;
fn main() {
    let mut config = Config::new();
    config.cache_config_load("cache.toml").unwrap();
}
