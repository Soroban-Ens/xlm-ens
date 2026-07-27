# Fuzz targets

Fuzz targets for `xlm-ens`, run with [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) on nightly Rust.

```sh
cargo install cargo-fuzz
cargo +nightly fuzz run <target> -- -max_total_time=60
```

Run from the `fuzz/` directory (or pass `--fuzz-dir fuzz` from the repo root).

## Targets

### `fuzz_parse_fqdn`

Feeds arbitrary UTF-8 strings to `xlm_ns_common::validation::parse_fqdn`. Asserts only that the
function never panics.

### `fuzz_validate_label`

Feeds arbitrary UTF-8 strings to `xlm_ns_common::validation::validate_label`. Asserts only that the
function never panics.

### `fuzz_pricing_calculation`

Feeds arbitrary `(label_length: usize, years: u64)` pairs to
`xlm_ns_registrar::pricing::price_for_label_length` and checks:

- the fee is always positive,
- the fee matches the expected tier for the given label length,
- `fee.saturating_mul(years)` is never smaller than the annual fee for `years >= 1` (i.e.
  multi-year pricing never silently undercharges due to overflow),
- when the true product overflows `u64`, the saturating multiplication caps at `u64::MAX` rather
  than wrapping to an incorrect small value.

### `fuzz_quote_construction`

Feeds arbitrary `(label_length: usize, years: u64, now_unix: u64)` to `build_quote_pure` — the
env-free core of the registrar's `build_quote` — and checks the temporal and pricing invariants of
the resulting quote:

- `expiry_unix >= now_unix`, and strictly greater when `years >= 1` (barring the `u64::MAX`
  saturation boundary),
- `grace_period_ends_at >= expiry_unix`,
- `valid_until >= now_unix`, and strictly greater unless `now_unix == u64::MAX`,
- `fee_stroops` equals the exact `annual_fee * years` product, or `u64::MAX` when that product
  would overflow.

Run for at least 10 million iterations (`-runs=10000000`) before treating these targets as clean;
see issues #453, #456, and #470 for related overflow-audit and premium-tier fuzzing work.
