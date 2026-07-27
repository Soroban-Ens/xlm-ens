#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use xlm_ns_registrar::pricing::price_for_label_length;

#[derive(Debug, Arbitrary)]
struct PricingInput {
    label_length: usize,
    years: u64,
}

fuzz_target!(|input: PricingInput| {
    let fee = price_for_label_length(input.label_length);

    // Every tier must charge something; a free registration is never valid.
    assert!(
        fee > 0,
        "fee must be positive for length {}",
        input.label_length
    );

    // Tier boundaries from price_for_label_length must hold exactly.
    let expected = match input.label_length {
        0..=3 => 1_000_000_000,
        4..=6 => 250_000_000,
        _ => 100_000_000,
    };
    assert_eq!(
        fee, expected,
        "unexpected fee {} for length {}",
        fee, input.label_length
    );

    // saturating_mul must never let a multi-year fee come out smaller than a
    // single year's fee -- that would silently undercharge on overflow.
    let total = fee.saturating_mul(input.years);
    if input.years == 0 {
        assert_eq!(total, 0);
    } else {
        assert!(
            total >= fee,
            "fee {} * years {} = {} is less than the annual fee (overflow bug)",
            fee,
            input.years,
            total
        );
    }

    // When the true product overflows u64, saturating_mul must saturate to
    // u64::MAX rather than wrap around to a small, incorrect value.
    if fee.checked_mul(input.years).is_none() {
        assert_eq!(total, u64::MAX);
    }
});
