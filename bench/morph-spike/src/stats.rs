use serde::Serialize;
#[derive(Serialize, Clone, Debug, Default)]
pub struct Stats { pub n: usize, pub median_ms: f64, pub p95_ms: f64, pub p99_ms: f64, pub max_ms: f64, pub mean_ms: f64 }
pub fn stats(mut v: Vec<f64>) -> Stats {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let q = |p: f64| v[(((n - 1) as f64) * p).round() as usize];
    Stats { n, median_ms: q(0.5), p95_ms: q(0.95), p99_ms: q(0.99), max_ms: v[n - 1], mean_ms: v.iter().sum::<f64>() / n as f64 }
}
pub fn rss_mb() -> f64 { memory_stats::memory_stats().map(|m| m.physical_mem as f64 / 1048576.0).unwrap_or(0.0) }
