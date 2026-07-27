#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use xlm_ns_registrar::pricing::price_for_label_length;
use xlm_ns_registrar::{build_quote_pure, DEFAULT_GRACE_PERIOD_SECONDS};

#[derive(Debug, Arbitrary)]
struct QuoteInput {
    label_length: usize,
    years: u64,
    now_unix: u64,
}

fuzz_target!(|input: QuoteInput| {
    let quote = build_quote_pure(
        input.label_length,
        input.years,
        input.now_unix,
        DEFAULT_GRACE_PERIOD_SECONDS,
    );

    // saturating_add never returns less than either operand for unsigned ints.
    assert!(quote.expiry_unix >= input.now_unix);
    assert!(quote.grace_period_ends_at >= quote.expiry_unix);
    assert!(quote.valid_until >= input.now_unix);

    // A real (>= 1 year) registration strictly extends past "now", except at
    // the u64::MAX boundary where saturation collapses expiry back to now.
    if input.years >= 1 && input.now_unix < u64::MAX {
        assert!(quote.expiry_unix > input.now_unix);
    }
    if input.now_unix < u64::MAX {
        assert!(quote.valid_until > input.now_unix);
    }

    // fee_stroops must be the saturating product of the annual fee and years
    // -- never a wrapped/truncated value that would undercharge.
    let annual_fee = price_for_label_length(input.label_length);
    assert_eq!(quote.pricing.annual_fee_stroops, annual_fee);
    assert_eq!(quote.pricing.duration_years, input.years);
    match annual_fee.checked_mul(input.years) {
        Some(exact) => assert_eq!(quote.fee_stroops, exact),
        None => assert_eq!(quote.fee_stroops, u64::MAX),
    }
});
