# Architecture and Contract Interaction

The `xlm-ns` system is composed of several independent contracts, each with specific state ownership and responsibilities. This design allows for modular upgrades and separation of concerns.

## State Ownership

- **Registry (`contracts/registry`)**: Owns the canonical truth of who owns a base name (e.g., `timmy.xlm`) and its lifecycle (registration, expiry, grace period).
- **Resolver (`contracts/resolver`)**: Owns the forward resolution mapping (name -> address), reverse resolution (address -> name), and metadata (text records).
- **Registrar (`contracts/registrar`)**: Manages pricing, duration policies, and treasury. It does not own name state directly but instructs the Registry to materialize state upon payment.
- **Subdomain (`contracts/subdomain`)**: Owns delegated namespaces (e.g., `pay.timmy.xlm`), tracking parent domain owners, controllers, and child subdomain owners.
- **NFT (`contracts/nft`)**: Owns the tokenized representation and approvals of a name for secondary markets.
- **Auction (`contracts/auction`)**: Owns the temporary state for premium name sales (bids, reserves, timestamps).
- **Bridge (`contracts/bridge`)**: Owns the configuration for cross-chain resolution routes.

## Cross-Contract Flows

### 1. Registration Flow
1. User requests a quote from the **Registrar**.
2. User submits payment and registration to the **Registrar**.
3. **Registrar** validates the payment and calls the **Registry** to create the official ownership entry.

### 2. Resolution Flow
1. Client queries the **Registry** to check if the name is active and to get the associated resolver address.
2. Client queries the **Resolver** (found in step 1) to get the target Stellar address or text records.

### 3. Subdomain Creation Flow
1. Parent owner registers their domain in the **Subdomain** contract.
2. Parent owner (or a delegated controller) creates a subdomain.
3. The **Subdomain** contract records the new child ownership. The new owner can now set records in the **Resolver**.

## Registry-to-Resolver Synchronization Rules

Because the Resolver maintains its own mapping of names to records, ownership between the Registry and Resolver could theoretically drift. To prevent this, the following synchronization rules apply:

1. **Resolver Authorization**: The Resolver must authorize writes against a single ownership source. By default, the Resolver queries the Registry (or the Subdomain contract for subdomains) to verify the caller is the current active owner before allowing any modifications to resolution records.
2. **Transfer Invalidation**: When a name is transferred in the Registry, the Registry pushes an ownership update to the Resolver. This immediately invalidates the previous owner's authority in the Resolver and updates it to the new owner, keeping both contracts in perfect sync.
3. **Subdomain Independence**: Subdomain records are resolved directly against the Subdomain contract's state, isolating their ownership lifecycle from base names in the main Registry.