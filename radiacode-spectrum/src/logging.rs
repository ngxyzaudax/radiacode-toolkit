use tracing_subscriber::EnvFilter;

pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("radiacode_spectrum=info,radiacode_lib=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_writer(std::io::stderr)
        .init();
}
