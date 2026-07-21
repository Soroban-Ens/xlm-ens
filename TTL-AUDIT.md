# Persistent Storage TTL Audit

> Issue #604 — Security: Add persistent storage TTL audit to prevent data loss

Soroban persistent storage entries age out after their TTL (time-to-live) expires
unless explicitly extended. This audit documents every persistent storage write
across all 7 contracts and whether TTL extension has been applied.

## Constants

All contracts use these Soroban ledger-count constants (same as resolver #146):

| Constant                    | Value     | Approx Duration |
|-----------------------------|-----------|-----------------|
| `PERSISTENT_LEDGER_TTL`     | 6,312,000 | ~1 year         |
| `PERSISTENT_LEDGER_THRESHOLD` | 3,156,000 | ~6 months     |

For **registry entry** writes a dynamic TTL is used based on the entry's
`expires_at` to ensure the storage survives the full registration lifecycle
(max 10 years + grace period).

## Audit Results

### ✅ Resolver (`contracts/resolver/src/lib.rs`)
*Reference implementation — already has TTL extension from #146.*

| DataKey | Storage Type | Write Location | TTL Extended? | Notes |
|---------|-------------|----------------|---------------|-------|
| `Forward(name)` | persistent | `put_record()`, `set_record()`, `set_address()`, `set_text_record()`, `batch_set()` | ✅ Yes | Via `extend_persistent_ttl` |
| `Reverse(address)` | persistent | `set_record()`, `set_address()`, `batch_set()` | ✅ Yes | Via `extend_persistent_ttl` |
| `Primary(address)` | persistent | `set_primary_name()` | ✅ Yes | Via `extend_persistent_ttl` |
| `Wildcard(name)` | persistent | `set_wildcard_resolution()` | ✅ Yes | Via `extend_persistent_ttl` |
| `ContractVersion` | persistent | `initialize()`, `upgrade()` | ✅ Yes | Via `extend_persistent_ttl` |
| `Registry` | instance | `initialize()` | ✅ Yes | Via `extend_instance_ttl` |
| `Admin` | instance | `initialize()` | ✅ Yes | Via `extend_instance_ttl` |
| `SubdomainContract` | instance | `set_subdomain_contract()` | ✅ Yes | Via `extend_instance_ttl` |

### ✅ Registry (`contracts/registry/src/lib.rs`)

| DataKey | Storage Type | Write Location | TTL Extended? | Notes |
|---------|-------------|----------------|---------------|-------|
| `Entry(name)` | persistent | `put_entry()` → `register()`, `transfer()`, `set_resolver()`, `set_target_address()`, `set_metadata()`, `update_owner()`, `renew()` | ✅ Yes | Dynamic TTL based on `expires_at` |
| `Lock(name)` | persistent | `put_lock()` → `lock_name()` | ✅ Yes | Fixed TTL |
| `OwnerNames(owner)` | persistent | `add_owner_name()`, `remove_owner_name()` | ✅ Yes | Fixed TTL |
| `ContractVersion` | persistent | `initialize()`, `upgrade()` | ✅ Yes | Fixed TTL |
| `Admin` | instance | `initialize()` | ✅ Yes | Instance TTL |
| `DisputeAdmin` | instance | `initialize()`, `set_dispute_admin()` | ✅ Yes | Instance TTL |
| `NftContract` | instance | `set_nft_contract()` | ✅ Yes | Instance TTL |

State-mutating functions that extend instance TTL: `initialize`, `set_nft_contract`,
`set_dispute_admin`, `register`, `transfer`, `set_resolver`, `set_target_address`,
`set_metadata`, `update_owner`, `renew`, `lock_name`, `unlock_name`.

### ✅ Registrar (`contracts/registrar/src/lib.rs`)

| DataKey | Storage Type | Write Location | TTL Extended? | Notes |
|---------|-------------|----------------|---------------|-------|
| `Registration(name)` | persistent | `register()`, `renew()`, `extend_during_grace()` | ✅ Yes | Dynamic TTL based on `expires_at` |
| `Reserved(label)` | persistent | `reserve_label()`, `load_reserved_manifest()` | ✅ Yes | Fixed TTL |
| `Treasury` | persistent | `register()`, `renew()`, `extend_during_grace()` | ✅ Yes | Fixed TTL |
| `RegistrationCount` | persistent | `register()` | ✅ Yes | Fixed TTL |
| `RenewalCount` | persistent | `renew()`, `extend_during_grace()` | ✅ Yes | Fixed TTL |
| `RateLimitConfig` | persistent | `initialize()`, `set_rate_limit_config()` | ✅ Yes | Fixed TTL |
| `GracePeriodSeconds` | persistent | `initialize()`, `set_grace_period()` | ✅ Yes | Fixed TTL |
| `WhitelistedAddress(address)` | persistent | `whitelist_address()` | ✅ Yes | Fixed TTL |
| `RegistrationWindow(addr,start)` | persistent | `record_registration()` | ✅ Yes | Fixed TTL |
| `ContractVersion` | persistent | `initialize()`, `upgrade()` | ✅ Yes | Fixed TTL |
| `Registry` | instance | `initialize()` | ✅ Yes | Instance TTL |
| `Admin` | instance | `initialize()` | ✅ Yes | Instance TTL |

State-mutating functions that extend instance TTL: `initialize`, `upgrade`,
`reserve_label`, `load_reserved_manifest`, `register`, `renew`, `extend_during_grace`,
`set_rate_limit_config`, `set_grace_period`, `whitelist_address`,
`remove_whitelist_address`.

### ✅ Auction (`contracts/auction/src/lib.rs`)

| DataKey | Storage Type | Write Location | TTL Extended? | Notes |
|---------|-------------|----------------|---------------|-------|
| `Auction(name)` | persistent | `put_auction()` → `create_auction()`, `place_bid()` | ✅ Yes | Fixed TTL |
| `AuctionNames` | persistent | `create_auction()` | ✅ Yes | Fixed TTL |
| `Settlement(name)` | persistent | `settle()` | ✅ Yes | Fixed TTL |
| `ContractVersion` | persistent | `initialize()`, `upgrade()` | ✅ Yes | Fixed TTL |
| `Admin` | instance | `initialize()` | ✅ Yes | Instance TTL |
| `ReentrancyLock` | instance | `with_reentrancy_lock()` | ✅ Yes | Instance TTL |

State-mutating functions that extend instance TTL: `initialize`, `upgrade`,
`create_auction`, `place_bid`, `settle`.

### ✅ NFT (`contracts/nft/src/lib.rs`)

| DataKey | Storage Type | Write Location | TTL Extended? | Notes |
|---------|-------------|----------------|---------------|-------|
| `Token(token_id)` | persistent | `mint()`, `approve()`, `approve_clear()`, `transfer()`, `transfer_from()`, `sync_expiry()`, `sync_owner()` | ✅ Yes | Fixed TTL |
| `NameData(token_id)` | persistent | `mint()`, `refresh_name_data()` | ✅ Yes | Fixed TTL |
| `TokenIds` | persistent | `append_token_id()`, `burn()` | ✅ Yes | Fixed TTL |
| `OwnerTokens(owner)` | persistent | `add_owner_token()`, `remove_owner_token()` | ✅ Yes | Fixed TTL |
| `ContractVersion` | persistent | `initialize()`, `upgrade()` | ✅ Yes | Fixed TTL |
| `Admin` | instance | `initialize()` | ✅ Yes | Instance TTL |
| `Registry` | instance | `set_registry()` | ✅ Yes | Instance TTL |

State-mutating functions that extend instance TTL: `initialize`, `upgrade`,
`set_registry`, `mint`, `refresh_name_data`, `approve`, `approve_clear`, `transfer`,
`transfer_from`, `burn`, `sync_expiry`, `sync_owner`.

### ✅ Bridge (`contracts/bridge/src/lib.rs`)

| DataKey | Storage Type | Write Location | TTL Extended? | Notes |
|---------|-------------|----------------|---------------|-------|
| `Route(chain)` | persistent | `register_chain()` | ✅ Yes | Fixed TTL |
| `SupportedChain(chain_id)` | persistent | `add_supported_chain()` | ✅ Yes | Fixed TTL |
| `SupportedChainIds` | persistent | `add_supported_chain()`, `remove_supported_chain()` | ✅ Yes | Fixed TTL |
| `ContractVersion` | persistent | `initialize()`, `upgrade()` | ✅ Yes | Fixed TTL |
| `Admin` | instance | `initialize()` | ✅ Yes | Instance TTL |

State-mutating functions that extend instance TTL: `initialize`, `upgrade`,
`register_chain`, `add_supported_chain`, `remove_supported_chain`.

### ✅ Subdomain (`contracts/subdomain/src/lib.rs`)

| DataKey | Storage Type | Write Location | TTL Extended? | Notes |
|---------|-------------|----------------|---------------|-------|
| `Parent(parent)` | persistent | `register_parent()`, `add_controller()`, `remove_controller()` | ✅ Yes | Fixed TTL |
| `Subdomain(fqdn)` | persistent | `create()`, `transfer()` | ✅ Yes | Fixed TTL |
| `ParentSubdomains(parent)` | persistent | `add_parent_subdomain()`, `remove_parent_subdomain()` | ✅ Yes | Fixed TTL |
| `OwnerSubdomains(owner)` | persistent | `add_owner_subdomain()`, `remove_owner_subdomain()` | ✅ Yes | Fixed TTL |
| `ContractVersion` | persistent | `initialize()` | ✅ Yes | Fixed TTL |
| `MaxDepth` | persistent | `initialize()`, `set_max_depth()` | ✅ Yes | Fixed TTL |
| `Admin` | instance | `initialize()` | ✅ Yes | Instance TTL |
| `RegistryContract` | instance | `set_registry_contract()` | ✅ Yes | Instance TTL |

State-mutating functions that extend instance TTL: `initialize`, `set_registry_contract`,
`set_max_depth`, `register_parent`, `add_controller`, `remove_controller`, `create`,
`transfer`, `delete`, `revoke`.

## Summary

| Contract | Persistent Writes | Instance Writes | Status |
|----------|------------------|-----------------|--------|
| Resolver | 5                 | 3               | ✅ Reference (was already done) |
| Registry | 4                 | 3               | ✅ Fixed |
| Registrar | 10               | 2               | ✅ Fixed |
| Auction | 4                 | 2               | ✅ Fixed |
| NFT     | 5                 | 2               | ✅ Fixed |
| Bridge  | 4                 | 1               | ✅ Fixed |
| Subdomain | 6               | 2               | ✅ Fixed |

All 7 contracts now consistently extend TTLs on every persistent storage write
and extend instance TTLs in state-mutating functions.
