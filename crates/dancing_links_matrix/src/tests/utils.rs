pub(super) fn create_row<const N: usize>(v: [&str; N]) -> Vec<String> {
    v.iter().map(|v| v.to_string()).collect()
}
