//! Property-based tests for the name lifecycle state machine.
//!
//! `NameState` (see `NameState` in `lib.rs`) has four states — `Missing`,
//! `Active`, `GracePeriod`, `Claimable` — reached only through `register`,
//! `renew`, and `burn`. There is no explicit "expire" call: expiry is a
//! function of the caller-supplied `now_unix` crossing `expires_at` /
//! `grace_period_ends_at`, so time passing is modeled here as an operation's
//! `now` advancing past those boundaries rather than as a distinct op.
//! "Reclaim" is likewise not a separate entry point: it is `register` called
//! again once the previous entry has become `Claimable`.
//!
//! Each generated operation sequence is replayed against both the real
//! contract and a plain-data model (`ModelEntry`) that mirrors the contract's
//! branch order exactly. After every step we assert:
//!   1. the contract's success/failure (and error variant) matches the model,
//!   2. `name_state` matches the state derived from the model,
//!   3. the owner index (`names_for_owner`) matches model ownership, and
//!   4. `audit_owner_index` reports no corruption for either party.
//!
//! Failing cases are persisted by proptest to
//! `contracts/registry/proptest-regressions/`, which is checked into git so
//! a shrunk failing sequence reproduces deterministically on the next run
//! instead of relying on a single hard-coded seed (the same convention
//! `xlm-ns-common`'s validation proptests already use).

#[cfg(test)]
mod tests {
    extern crate std;

    use proptest::prelude::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    use crate::{NameState, RegistryContract, RegistryContractClient, RegistryError};
    use xlm_ns_common::time::{is_active_at, is_claimable_at};

    #[derive(Clone, Debug)]
    struct ModelEntry {
        owner: u8,
        expires_at: u64,
        grace_period_ends_at: u64,
    }

    impl ModelEntry {
        fn is_active_at(&self, now: u64) -> bool {
            is_active_at(self.expires_at, now)
        }

        fn is_claimable_at(&self, now: u64) -> bool {
            is_claimable_at(self.grace_period_ends_at, now)
        }
    }

    #[derive(Clone, Debug)]
    enum Op {
        Register {
            owner: u8,
            time_delta: u64,
            expires_offset: i64,
            grace_offset: i64,
        },
        Renew {
            caller: u8,
            time_delta: u64,
            expires_offset: i64,
            grace_offset: i64,
        },
        Burn {
            caller: u8,
            time_delta: u64,
        },
    }

    /// Applies a signed offset to a base timestamp, saturating at zero
    /// instead of panicking on underflow so out-of-range strategy values
    /// stay well-formed `u64`s (some are intentionally negative to drive the
    /// contract's `InvalidExpiry` / `InvalidGracePeriod` paths).
    fn offset(base: u64, delta: i64) -> u64 {
        if delta < 0 {
            base.saturating_sub(delta.unsigned_abs())
        } else {
            base.saturating_add(delta as u64)
        }
    }

    fn op_strategy() -> impl Strategy<Value = Op> {
        prop_oneof![
            (
                0u8..2,
                0u64..200_000,
                -50_000i64..3_000_000,
                -50_000i64..3_000_000
            )
                .prop_map(|(owner, time_delta, expires_offset, grace_offset)| {
                    Op::Register {
                        owner,
                        time_delta,
                        expires_offset,
                        grace_offset,
                    }
                }),
            (
                0u8..2,
                0u64..200_000,
                -50_000i64..3_000_000,
                -50_000i64..3_000_000
            )
                .prop_map(|(caller, time_delta, expires_offset, grace_offset)| {
                    Op::Renew {
                        caller,
                        time_delta,
                        expires_offset,
                        grace_offset,
                    }
                }),
            (0u8..2, 0u64..200_000)
                .prop_map(|(caller, time_delta)| Op::Burn { caller, time_delta }),
        ]
    }

    /// Mirrors `RegistryContract::register`'s validation order exactly.
    fn model_register(
        model: &Option<ModelEntry>,
        now: u64,
        expires_at: u64,
        grace_period_ends_at: u64,
    ) -> Result<ModelEntry, RegistryError> {
        if !is_active_at(expires_at, now) {
            return Err(RegistryError::InvalidExpiry);
        }
        if grace_period_ends_at < expires_at {
            return Err(RegistryError::InvalidGracePeriod);
        }
        if let Some(existing) = model {
            if existing.is_active_at(now) {
                return Err(RegistryError::AlreadyRegistered);
            }
            if !existing.is_claimable_at(now) {
                return Err(RegistryError::NotYetClaimable);
            }
        }
        Ok(ModelEntry {
            owner: 0, // caller fills in the real owner index.
            expires_at,
            grace_period_ends_at,
        })
    }

    /// Mirrors `RegistryContract::renew`'s validation order exactly.
    fn model_renew(
        model: &Option<ModelEntry>,
        caller: u8,
        now: u64,
        expires_at: u64,
        grace_period_ends_at: u64,
    ) -> Result<ModelEntry, RegistryError> {
        let entry = model.as_ref().ok_or(RegistryError::NotFound)?;
        if entry.is_claimable_at(now) {
            return Err(RegistryError::NotActive);
        }
        if entry.owner != caller {
            return Err(RegistryError::Unauthorized);
        }
        if expires_at < entry.expires_at {
            return Err(RegistryError::InvalidExpiry);
        }
        if grace_period_ends_at < entry.grace_period_ends_at {
            return Err(RegistryError::InvalidGracePeriod);
        }
        if !is_active_at(expires_at, now) {
            return Err(RegistryError::InvalidExpiry);
        }
        if grace_period_ends_at < expires_at {
            return Err(RegistryError::InvalidGracePeriod);
        }
        Ok(ModelEntry {
            owner: entry.owner,
            expires_at,
            grace_period_ends_at,
        })
    }

    /// Mirrors `RegistryContract::burn`'s validation order exactly.
    fn model_burn(model: &Option<ModelEntry>, caller: u8, now: u64) -> Result<(), RegistryError> {
        let entry = model.as_ref().ok_or(RegistryError::NotFound)?;
        if entry.owner != caller && !entry.is_claimable_at(now) {
            return Err(RegistryError::Unauthorized);
        }
        Ok(())
    }

    fn model_name_state(model: &Option<ModelEntry>, now: u64) -> NameState {
        match model {
            None => NameState::Missing,
            Some(entry) => {
                if entry.is_active_at(now) {
                    NameState::Active
                } else if entry.is_claimable_at(now) {
                    NameState::Claimable
                } else {
                    NameState::GracePeriod
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn lifecycle_matches_model_and_stays_consistent(ops in prop::collection::vec(op_strategy(), 1..25)) {
            let env = Env::default();
            env.mock_all_auths();
            let contract_id = env.register(RegistryContract, ());
            let client = RegistryContractClient::new(&env, &contract_id);

            let owners = [Address::generate(&env), Address::generate(&env)];
            let name = String::from_str(&env, "proptest.xlm");

            // Starts well above zero so `saturating_sub` offsets in `offset()`
            // exercise real "before now" values instead of clamping at 0.
            let mut clock: u64 = 10_000_000;
            let mut model: Option<ModelEntry> = None;

            for op in ops {
                match op {
                    Op::Register { owner, time_delta, expires_offset, grace_offset } => {
                        clock = clock.saturating_add(time_delta);
                        let now = clock;
                        let expires_at = offset(now, expires_offset);
                        let grace_period_ends_at = offset(expires_at, grace_offset);

                        let expected = model_register(&model, now, expires_at, grace_period_ends_at);
                        let result = client.try_register(
                            &name,
                            &owners[owner as usize],
                            &None::<String>,
                            &None::<String>,
                            &now,
                            &expires_at,
                            &grace_period_ends_at,
                        );

                        match (expected, result) {
                            (Ok(_), Ok(Ok(()))) => {
                                model = Some(ModelEntry { owner, expires_at, grace_period_ends_at });
                            }
                            (Err(expected_err), Err(Ok(actual_err))) => {
                                prop_assert_eq!(expected_err, actual_err, "register at now={}", now);
                            }
                            (expected, actual) => {
                                prop_assert!(
                                    false,
                                    "register outcome mismatch at now={}: expected {:?}, got {:?}",
                                    now, expected, actual,
                                );
                            }
                        }
                    }
                    Op::Renew { caller, time_delta, expires_offset, grace_offset } => {
                        clock = clock.saturating_add(time_delta);
                        let now = clock;
                        let expires_at = offset(now, expires_offset);
                        let grace_period_ends_at = offset(expires_at, grace_offset);

                        let expected = model_renew(&model, caller, now, expires_at, grace_period_ends_at);
                        let result = client.try_renew(
                            &name,
                            &owners[caller as usize],
                            &expires_at,
                            &grace_period_ends_at,
                            &now,
                        );

                        match (expected, result) {
                            (Ok(new_entry), Ok(Ok(()))) => {
                                model = Some(new_entry);
                            }
                            (Err(expected_err), Err(Ok(actual_err))) => {
                                prop_assert_eq!(expected_err, actual_err, "renew at now={}", now);
                            }
                            (expected, actual) => {
                                prop_assert!(
                                    false,
                                    "renew outcome mismatch at now={}: expected {:?}, got {:?}",
                                    now, expected, actual,
                                );
                            }
                        }
                    }
                    Op::Burn { caller, time_delta } => {
                        clock = clock.saturating_add(time_delta);
                        let now = clock;

                        let expected = model_burn(&model, caller, now);
                        let result = client.try_burn(&name, &owners[caller as usize], &now);

                        match (expected, result) {
                            (Ok(()), Ok(Ok(()))) => {
                                model = None;
                            }
                            (Err(expected_err), Err(Ok(actual_err))) => {
                                prop_assert_eq!(expected_err, actual_err, "burn at now={}", now);
                            }
                            (expected, actual) => {
                                prop_assert!(
                                    false,
                                    "burn outcome mismatch at now={}: expected {:?}, got {:?}",
                                    now, expected, actual,
                                );
                            }
                        }
                    }
                }

                prop_assert_eq!(
                    client.name_state(&name, &clock),
                    model_name_state(&model, clock),
                    "name_state mismatch at now={}",
                    clock,
                );

                for (idx, owner) in owners.iter().enumerate() {
                    let expected_indexed = model.as_ref().map(|e| e.owner as usize == idx).unwrap_or(false);
                    prop_assert_eq!(
                        client.names_for_owner(owner).contains(&name),
                        expected_indexed,
                        "owner index mismatch for owner {} at now={}",
                        idx, clock,
                    );
                    prop_assert!(
                        client.audit_owner_index(owner).is_empty(),
                        "owner index corruption detected for owner {} at now={}",
                        idx, clock,
                    );
                }
            }
        }
    }
}
