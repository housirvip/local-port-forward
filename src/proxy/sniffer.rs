const PREFIXES: &[&[u8]] = &[
    b"GET ", b"POST", b"PUT ", b"DELE", b"HEAD", b"PATC", b"OPTI", b"CONN",
];

/// Returns true if the first 4 bytes of `buf` match a known HTTP method prefix.
pub fn is_http(buf: &[u8]) -> bool {
    if buf.len() < 4 {
        return false;
    }
    PREFIXES.iter().any(|p| buf.starts_with(p))
}
