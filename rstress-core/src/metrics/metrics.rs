use hdrhistogram::Histogram;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Metrics {
    pub sent: u64,
    pub ok: u64,
    pub err: u64,
    pub hist: Histogram<u64>,      // microseconds
    pub codes: BTreeMap<u16, u64>, // 0 == transport error
}

impl Metrics {
    pub fn new() -> Self {
        let mut h = Histogram::<u64>::new_with_bounds(1, 10_000_000, 3).unwrap(); // 1us..10s
        h.auto(true);
        Self { sent: 0, ok: 0, err: 0, hist: h, codes: BTreeMap::new() }
    }

    pub fn record(&mut self, ok: bool, lat_us: u64, code: Option<u16>) {
        self.sent += 1;
        if ok { self.ok += 1; } else { self.err += 1; }
        let _ = self.hist.record(lat_us.min(10_000_000));
        *self.codes.entry(code.unwrap_or(0)).or_insert(0) += 1;
    }

    pub fn merge(&mut self, other: &Metrics) {
        self.sent += other.sent;
        self.ok += other.ok;
        self.err += other.err;
        self.hist.add(&other.hist).ok();
        for (k, v) in &other.codes {
            *self.codes.entry(*k).or_insert(0) += v;
        }
    }
}