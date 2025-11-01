use hdrhistogram::Histogram;

#[inline]
pub fn quantile_ms(hist: &Histogram<u64>, q: f64) -> f64 {
    if hist.len() == 0 { return 0.0; }
    hist.value_at_quantile(q / 100.0) as f64 / 1000.0
}