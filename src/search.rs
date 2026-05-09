pub fn find_all(haystack: &str, needle: &str) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    let h = haystack.to_lowercase();
    let n = needle.to_lowercase();
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(idx) = h[start..].find(&n) {
        out.push(start + idx);
        start += idx + n.len();
    }
    out
}
