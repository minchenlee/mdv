#[test]
fn finds_all_case_insensitive() {
    let m = mdv::search::find_all("Hello hello HELLO", "hello");
    assert_eq!(m, vec![0, 6, 12]);
}

#[test]
fn empty_needle_returns_no_matches() {
    let m = mdv::search::find_all("abc", "");
    assert!(m.is_empty());
}

#[test]
fn no_match_returns_empty() {
    let m = mdv::search::find_all("abc", "xyz");
    assert!(m.is_empty());
}
