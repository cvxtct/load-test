use hdrhistogram::Histogram;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Metrics {
    pub sent: u64,
    pub ok: u64,
    pub err: u64,
    pub dropped: u64,
    pub hist: Histogram<u64>,                   // microseconds
    pub codes: BTreeMap<u16, u64>,              // HTTP status counts; 0 == transport error
    pub transport: BTreeMap<&'static str, u64>, // classified transport errors
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        let mut h = Histogram::<u64>::new_with_bounds(1, 10_000_000, 3).unwrap(); // 1us..10s
        h.auto(true);
        Self {
            sent: 0,
            ok: 0,
            err: 0,
            dropped: 0,
            hist: h,
            codes: BTreeMap::new(),
            transport: BTreeMap::new(),
        }
    }

    pub fn record(&mut self, ok: bool, lat_us: u64, code: Option<u16>) {
        self.sent += 1;
        if ok {
            self.ok += 1;
        } else {
            self.err += 1;
        }
        let _ = self.hist.record(lat_us.min(10_000_000));
        *self.codes.entry(code.unwrap_or(0)).or_insert(0) += 1;
    }

    pub fn record_dropped(&mut self, dropped: u64) {
        self.dropped = self.dropped + dropped;
    }

    pub fn record_transport_kind(&mut self, kind: &'static str) {
        *self.transport.entry(kind).or_insert(0) += 1;
    }

    pub fn merge(&mut self, other: &Metrics) {
        self.sent += other.sent;
        self.ok += other.ok;
        self.err += other.err;
        self.dropped += other.dropped;
        self.hist.add(&other.hist).ok();
        for (k, v) in &other.codes {
            *self.codes.entry(*k).or_insert(0) += v;
        }
        for (k, v) in &other.transport {
            *self.transport.entry(*k).or_insert(0) += v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_merge() {
        let mut a = Metrics::new();
        a.record(true, 1000, Some(200));
        a.record(false, 2000, None); // transport → code 0
        a.record_dropped(100);

        assert_eq!(a.sent, 2);
        assert_eq!(a.ok, 1);
        assert_eq!(a.err, 1);
        assert_eq!(*a.codes.get(&200).unwrap(), 1);
        assert_eq!(*a.codes.get(&0).unwrap(), 1);
        assert_eq!(a.dropped, 100);

        let mut b = Metrics::new();
        b.record(true, 1500, Some(201));
        a.merge(&b);

        assert_eq!(a.sent, 3);
        assert_eq!(*a.codes.get(&201).unwrap(), 1);
    }
}
