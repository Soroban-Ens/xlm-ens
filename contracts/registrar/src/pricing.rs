pub fn price_for_label_length(length: usize) -> u64 {
    match length {
        0..=3 => 1_000_000_000,
        4..=6 => 250_000_000,
        _ => 100_000_000,
    }
}
