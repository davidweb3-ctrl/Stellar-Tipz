//! Tip record storage for the Tipz contract.
//!
//! Tips are stored in **temporary** storage so they are automatically evicted
//! after [`TIP_TTL_LEDGERS`] ledgers (~7 days), preventing unbounded state
//! growth. Callers that read expired entries receive `None` / an empty list
//! rather than an error.
//!
//! # Storage layout
//! | Key                    | Storage   | Value  |
//! |------------------------|-----------|--------|
//! | `DataKey::Tip(id)`     | temporary | `Tip`  |
//! | `DataKey::TipCount`    | instance  | `u32`  |
//!
//! `TipCount` is the next available tip ID (i.e. the total number of tips ever
//! created). It lives in instance storage so it persists for the lifetime of
//! the contract.

use soroban_sdk::{Address, Env, String, Vec};

use crate::storage::DataKey;
use crate::types::Tip;

/// Approximate TTL for tip records in ledgers.
///
/// 7 days × 86 400 s/day ÷ 5 s/ledger = 120 960 ledgers.
pub const TIP_TTL_LEDGERS: u32 = 120_960;

/// Create a new [`Tip`] record and store it in temporary storage.
///
/// The global tip counter (`DataKey::TipCount`) is incremented atomically so
/// each tip gets a unique, monotonically increasing ID.  The entry TTL is
/// extended to [`TIP_TTL_LEDGERS`] immediately after insertion.
///
/// Returns the ID assigned to the new tip.
///
/// # Note
/// This function will be called by `send_tip` once issue #7 is implemented.
#[allow(dead_code)]
pub fn store_tip(
    env: &Env,
    tipper: &Address,
    creator: &Address,
    amount: i128,
    message: String,
) -> u32 {
    let tip_id: u32 = env
        .storage()
        .instance()
        .get(&DataKey::TipCount)
        .unwrap_or(0);

    // Increment the global counter before storing so future reads see the
    // correct total even if the entry itself has expired.
    env.storage()
        .instance()
        .set(&DataKey::TipCount, &tip_id.saturating_add(1));

    let tip = Tip {
        id: tip_id,
        tipper: tipper.clone(),
        creator: creator.clone(),
        amount,
        message,
        timestamp: env.ledger().timestamp(),
    };

    env.storage().temporary().set(&DataKey::Tip(tip_id), &tip);

    // Extend TTL to TIP_TTL_LEDGERS from the current ledger.
    // threshold == extend_to means "always extend when below the target".
    env.storage()
        .temporary()
        .extend_ttl(&DataKey::Tip(tip_id), TIP_TTL_LEDGERS, TIP_TTL_LEDGERS);

    tip_id
}

/// Retrieve a single tip by its ID.
///
/// Returns `None` if the tip does not exist or its TTL has expired.
pub fn get_tip(env: &Env, tip_id: u32) -> Option<Tip> {
    env.storage().temporary().get(&DataKey::Tip(tip_id))
}

/// Return up to `count` recent tips sent to `creator`.
///
/// Tips are scanned backwards from the most recent global tip ID so that the
/// newest entries appear first in the returned vector.  Entries that have
/// already expired are silently skipped, meaning the returned vector may
/// contain fewer than `count` items even when the creator has received more
/// tips in total.
pub fn get_recent_tips(env: &Env, creator: &Address, count: u32) -> Vec<Tip> {
    let tip_count: u32 = env
        .storage()
        .instance()
        .get(&DataKey::TipCount)
        .unwrap_or(0);

    let mut result: Vec<Tip> = Vec::new(env);
    let mut found: u32 = 0;
    let mut i: u32 = tip_count;

    while i > 0 && found < count {
        i -= 1;

        // Expired entries are simply absent from temporary storage — skip them.
        if let Some(tip) = env
            .storage()
            .temporary()
            .get::<DataKey, Tip>(&DataKey::Tip(i))
        {
            if tip.creator == *creator {
                result.push_back(tip);
                found += 1;
            }
        }
    }

use soroban_sdk::{token, Address, Env, String};

use crate::errors::ContractError;
use crate::events::emit_tip_sent;
use crate::storage::{self, DataKey};
use crate::types::Tip;

/// Send an XLM tip from `tipper` to a registered `creator`.
pub fn send_tip(
    env: &Env,
    tipper: &Address,
    creator: &Address,
    amount: i128,
    message: &String,
) -> Result<(), ContractError> {
    // 1. Require tipper authorization
    tipper.require_auth();

    // 2. Validate creator is registered
    if !storage::has_profile(env, creator) {
        return Err(ContractError::NotRegistered);
    }

    // 3. Validate tipper != creator
    if tipper == creator {
        return Err(ContractError::CannotTipSelf);
    }

    // 4. Validate amount > 0
    if amount <= 0 {
        return Err(ContractError::InvalidAmount);
    }

    // 5. Validate message length ≤ 280 chars
    if message.len() > 280 {
        return Err(ContractError::MessageTooLong);
    }

    // 6. Transfer XLM from tipper to contract via the Stellar Asset Contract (SAC)
    let native_token = storage::get_native_token(env);
    let token_client = token::Client::new(env, &native_token);
    let contract_address = env.current_contract_address();
    token_client.transfer(tipper, &contract_address, &amount);

    // 7. Credit amount to creator's balance
    let mut profile = storage::get_profile(env, creator);
    profile.balance += amount;
    profile.total_tips_received += amount;
    profile.total_tips_count += 1;
    storage::set_profile(env, &profile);

    // 8. Create Tip record and store in temporary storage
    let tip_index = storage::increment_tip_count(env);
    let tip = Tip {
        from: tipper.clone(),
        to: creator.clone(),
        amount,
        message: message.clone(),
        timestamp: env.ledger().timestamp(),
    };
    env.storage()
        .temporary()
        .set(&DataKey::Tip(tip_index), &tip);

    // 9. Add to lifetime tip volume
    storage::add_to_tips_volume(env, amount);

    // 10. Emit TipSent event
    emit_tip_sent(env, tipper, creator, amount);

    Ok(())
}

// TODO: Implement withdraw_tips in issue #10
