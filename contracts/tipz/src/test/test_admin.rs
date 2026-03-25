//! Tests for admin fee-management functions.
//!
//! Covers:
//!   set_fee        – happy path, boundary, above-max, non-admin
//!   set_fee_collector – happy path, non-admin
//!   set_admin      – happy path, old-admin locked-out, non-admin
//!
//! Drop this file at  contracts/tipz/src/test_admin_fee.rs
//! and add  `mod test_admin_fee;`  inside the  `#[cfg(test)]`  block in lib.rs
//! (or alongside the existing `mod test;` line).

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env,
};

use crate::errors::ContractError;
use crate::storage::DataKey;
use crate::TipzContract;
use crate::TipzContractClient;

// ── shared setup ─────────────────────────────────────────────────────────────

struct TestCtx<'a> {
    env: Env,
    client: TipzContractClient<'a>,
    admin: Address,
    fee_collector: Address,
}

/// Spin up a contract and call `initialize` so all three admin functions have
/// a valid contract state to work against.
fn setup() -> TestCtx<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TipzContract);
    let client = TipzContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let fee_collector = Address::generate(&env);

    // Register a native-token SAC so `initialize` succeeds
    let token_admin = Address::generate(&env);
    let native_token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &fee_collector, &200_u32, &native_token);

    TestCtx {
        env,
        client,
        admin,
        fee_collector,
    }
}

// ── set_fee ───────────────────────────────────────────────────────────────────

#[test]
fn test_set_fee_updates_stored_value() {
    let ctx = setup();
    ctx.client.set_fee(&ctx.admin, &500_u32);

    let stored: u32 = ctx.env.as_contract(&ctx.client.address, || {
        ctx.env
            .storage()
            .instance()
            .get(&DataKey::FeePercent)
            .unwrap()
    });
    assert_eq!(stored, 500);
}

#[test]
fn test_set_fee_boundary_1000_succeeds() {
    let ctx = setup();
    ctx.client.set_fee(&ctx.admin, &1000_u32);

    let stored: u32 = ctx.env.as_contract(&ctx.client.address, || {
        ctx.env
            .storage()
            .instance()
            .get(&DataKey::FeePercent)
            .unwrap()
    });
    assert_eq!(stored, 1000);
}

#[test]
fn test_set_fee_zero_succeeds() {
    let ctx = setup();
    ctx.client.set_fee(&ctx.admin, &0_u32);

    let stored: u32 = ctx.env.as_contract(&ctx.client.address, || {
        ctx.env
            .storage()
            .instance()
            .get(&DataKey::FeePercent)
            .unwrap()
    });
    assert_eq!(stored, 0);
}

#[test]
fn test_set_fee_above_1000_returns_invalid_fee() {
    let ctx = setup();
    let result = ctx.client.try_set_fee(&ctx.admin, &1001_u32);
    assert_eq!(result, Err(Ok(ContractError::InvalidFee)));
}

#[test]
fn test_set_fee_non_admin_returns_not_authorized() {
    let ctx = setup();
    let attacker = Address::generate(&ctx.env);
    let result = ctx.client.try_set_fee(&attacker, &100_u32);
    assert_eq!(result, Err(Ok(ContractError::NotAuthorized)));
}

#[test]
fn test_set_fee_emits_fee_updated_event() {
    let ctx = setup();
    // fee was initialised to 200; change to 300
    ctx.client.set_fee(&ctx.admin, &300_u32);

    let events = ctx.env.events().all();
    assert!(
        !events.is_empty(),
        "expected a FeeUpdated event to be emitted"
    );
}

// ── set_fee_collector ─────────────────────────────────────────────────────────

#[test]
fn test_set_fee_collector_updates_stored_address() {
    let ctx = setup();
    let new_collector = Address::generate(&ctx.env);

    ctx.client.set_fee_collector(&ctx.admin, &new_collector);

    let stored: Address = ctx.env.as_contract(&ctx.client.address, || {
        ctx.env
            .storage()
            .instance()
            .get(&DataKey::FeeCollector)
            .unwrap()
    });
    assert_eq!(stored, new_collector);
}

#[test]
fn test_set_fee_collector_non_admin_returns_not_authorized() {
    let ctx = setup();
    let attacker = Address::generate(&ctx.env);
    let new_collector = Address::generate(&ctx.env);
    let result = ctx.client.try_set_fee_collector(&attacker, &new_collector);
    assert_eq!(result, Err(Ok(ContractError::NotAuthorized)));
}

#[test]
fn test_set_fee_collector_emits_event() {
    let ctx = setup();
    let new_collector = Address::generate(&ctx.env);
    ctx.client.set_fee_collector(&ctx.admin, &new_collector);

    let events = ctx.env.events().all();
    assert!(
        !events.is_empty(),
        "expected a FeeCollectorUpdated event to be emitted"
    );
}

// ── set_admin ─────────────────────────────────────────────────────────────────

#[test]
fn test_set_admin_updates_stored_address() {
    let ctx = setup();
    let new_admin = Address::generate(&ctx.env);

    ctx.client.set_admin(&ctx.admin, &new_admin);

    let stored: Address = ctx.env.as_contract(&ctx.client.address, || {
        ctx.env.storage().instance().get(&DataKey::Admin).unwrap()
    });
    assert_eq!(stored, new_admin);
}

#[test]
fn test_set_admin_old_admin_loses_access() {
    let ctx = setup();
    let new_admin = Address::generate(&ctx.env);

    ctx.client.set_admin(&ctx.admin, &new_admin);

    // old admin can no longer call set_fee
    let result = ctx.client.try_set_fee(&ctx.admin, &100_u32);
    assert_eq!(result, Err(Ok(ContractError::NotAuthorized)));
}

#[test]
fn test_set_admin_new_admin_gains_access() {
    let ctx = setup();
    let new_admin = Address::generate(&ctx.env);

    ctx.client.set_admin(&ctx.admin, &new_admin);

    // new admin can now call set_fee
    ctx.client.set_fee(&new_admin, &100_u32);

    let stored: u32 = ctx.env.as_contract(&ctx.client.address, || {
        ctx.env
            .storage()
            .instance()
            .get(&DataKey::FeePercent)
            .unwrap()
    });
    assert_eq!(stored, 100);
}

#[test]
fn test_set_admin_non_admin_returns_not_authorized() {
    let ctx = setup();
    let attacker = Address::generate(&ctx.env);
    let new_admin = Address::generate(&ctx.env);
    let result = ctx.client.try_set_admin(&attacker, &new_admin);
    assert_eq!(result, Err(Ok(ContractError::NotAuthorized)));
}

#[test]
fn test_set_admin_emits_admin_changed_event() {
    let ctx = setup();
    let new_admin = Address::generate(&ctx.env);
    ctx.client.set_admin(&ctx.admin, &new_admin);

    let events = ctx.env.events().all();
    assert!(
        !events.is_empty(),
        "expected an AdminChanged event to be emitted"
    );
}
