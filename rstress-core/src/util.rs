#[inline]
pub fn is_ok_status(code: u16) -> bool { code / 100 == 2 }