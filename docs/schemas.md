# Bridge Payload & Resolver Record Schemas

This document describes the data shapes exposed by the current contracts so integrators can build
against a stable surface without reading contract source.

## Bridge

### `BridgeRoute`

Stored under a `Route(<chain>)` persistent key in the Bridge contract.

Fields:

- `destination_chain` (string): Canonical chain identifier (e.g. `base`, `ethereum`, `arbitrum`).
- `destination_resolver` (string): Destination resolver identifier for the target chain.
- `gateway` (string): Gateway identifier used by the destination system.

### `build_message(name, chain) -> string`

Returns a JSON string shaped like:

```json
{
  "type": "xlm-ns-resolution",
  "name": "<fqdn>",
  "destination_chain": "<chain>",
  "resolver": "<destination_resolver>"
}
```

Notes:

- `name` must be a fully-qualified `.xlm` name (validated on-chain).
- `chain` must be a valid chain identifier (validated on-chain) and must have been registered.
- The returned message is deterministic for a given `(name, route)` pair.

## Resolver

### `ResolutionRecord`

Stored under a `Forward(<name>)` persistent key in the Resolver contract.

Fields:

- `owner` (Address): Account authorized to mutate the record.
- `address` (string): Resolved target address for forward resolution.
- `text_records` (map<string, string>): Bounded set of text records (max `MAX_TEXT_RECORDS`).
- `updated_at` (u64): Unix timestamp supplied by the caller on write.

### Forward resolution

- `resolve(name) -> Option<ResolutionRecord>`
- Storage key: `Forward(<name>)`

### Reverse resolution

- `reverse(address) -> Option<string>`
- Storage keys:
  - `Primary(<address>)` (preferred when set)
  - `Reverse(<address>)` (fallback)

### Text records

- `set_text_record(name, caller, key, value, now_unix) -> Result<(), ResolverError>`
- Writes mutate the `text_records` map inside the `Forward(<name>)` record.

