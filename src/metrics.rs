use once_cell::sync::Lazy;
use prometheus::{Encoder, IntCounter, Opts, Registry, TextEncoder};

pub static REGISTRY: Lazy<Registry> = Lazy::new(Registry::new);

pub static INGESTED_TOTAL: Lazy<IntCounter> = Lazy::new(|| {
    let opts = Opts::new("ingested_total", "Total number of ingested earthquakes");
    let c = IntCounter::with_opts(opts).expect("counter opts");
    REGISTRY
        .register(Box::new(c.clone()))
        .expect("register counter");
    c
});

/// Returns the metrics in the prometheus text exposition format (UTF-8 string).
pub fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("encode metrics");
    String::from_utf8(buffer).expect("metrics utf8")
}
