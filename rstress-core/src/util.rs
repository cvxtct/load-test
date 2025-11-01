use std::error::Error;

pub fn is_ok_status(code: u16) -> bool {
    // From a load-testing perspective, any HTTP code != 0 means a completed HTTP transaction.
    code > 0
}

// Keep classifier here if you want reuse; worker imports this.
pub fn classify_reqwest_error(e: &reqwest::Error) -> &'static str {
    if e.is_timeout() { return "timeout"; }
    if e.is_connect() { return "connect"; }
    if e.is_decode()  { return "decode"; }
    if e.is_redirect(){ return "redirect"; }
    if e.is_body()    { return "body"; }
    if e.is_request() { return "request"; }

    let mut src = e.source();
    while let Some(err) = src {
        if let Some(ioe) = err.downcast_ref::<std::io::Error>() {
            use std::io::ErrorKind::*;
            return match ioe.kind() {
                TimedOut => "timeout",
                ConnectionAborted | ConnectionReset | BrokenPipe => "conn_reset",
                UnexpectedEof => "unexpected_eof",
                _ => "io_other",
            };
        }
        src = err.source();
    }
    "other"
}