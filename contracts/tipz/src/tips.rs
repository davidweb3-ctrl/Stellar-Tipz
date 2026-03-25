//! Tip record storage and transfer logic for the Tipz contract.
//!
//! Tips are stored in temporary storage so they expire automatically after a
//! bounded lifetime, while aggregate counters remain in persistent contract
//! state.

use soroban_sdk::{token, Address, Env, String, Vec};

use crate::errors::ContractError;
use crate::events::emit_tip_sent;
use crate::storage::{self, DataKey};
use crate::token;
use crate::types::Tip;

/// Approximate TTL for tip records in ledgers.
///
/// 7 days × 86 400 s/day ÷ 5 s/ledger = 120 960 ledgers.
pub const TIP_TTL_LEDGERS: u32 = 120_960;

/// Create a new [`Tip`] record and store it in temporary storage.
pub fn store_tip(
    env: &Env,
    tipper: &Address,
    creator: &Address,
    amount: i128,
    message: String,
) -> u32 {
    let tip_id = storage::increment_tip_count(env);
    let tip = Tip {
        id: tip_id,
        tipper: tipper.clone(),
        creator: creator.clone(),
        amount,
        message,
        timestamp: env.ledger().timestamp(),
    };

    env.storage().temporary().set(&DataKey::Tip(tip_id), &tip);
    env.storage()
        .temporary()
        .extend_ttl(&DataKey::Tip(tip_id), TIP_TTL_LEDGERS, TIP_TTL_LEDGERS);

    tip_id
}

/// Retrieve a single tip by its ID.
pub fn get_tip(env: &Env, tip_id: u32) -> Option<Tip> {
    env.storage().temporary().get(&DataKey::Tip(tip_id))
}

/// Return up to `count` recent tips received by `creator`, newest first.
pub fn get_recent_tips(env: &Env, creator: &Address, count: u32) -> Vec<Tip> {
    let tip_count = storage::get_tip_count(env);
    let mut result = Vec::new(env);
    let mut found = 0_u32;
    let mut index = tip_count;

    while index > 0 && found < count {
        index -= 1;
        if let Some(tip) = env
            .storage()
            .temporary()
            .get::<DataKey, Tip>(&DataKey::Tip(index))
        {
            if tip.creator == *creator {
                result.push_back(tip);
                found += 1;
            }
        }
    }

    result
}

/// Send an XLM tip from `tipper` to a registered `creator`.
pub fn send_tip(
    env: &Env,
    tipper: &Address,
    creator: &Address,
    amount: i128,
    message: &String,
) -> Result<(), ContractError> {
    tipper.require_auth();

    if !storage::has_profile(env, creator) {
        return Err(ContractError::NotRegistered);
    }

    if tipper == creator {
        return Err(ContractError::CannotTipSelf);
    }

    if amount <= 0 {
        return Err(ContractError::InvalidAmount);
    }

    if message.len() > 280 {
        return Err(ContractError::MessageTooLong);
    }

    let contract_address = env.current_contract_address();
    token::transfer_xlm(env, tipper, &contract_address, amount)?;

    let mut profile = storage::get_profile(env, creator);
    profile.balance += amount;
    profile.total_tips_received += amount;
    profile.total_tips_count += 1;
    storage::set_profile(env, &profile);

    store_tip(env, tipper, creator, amount, message.clone());

    storage::add_to_tips_volume(env, amount);

    emit_tip_sent(env, tipper, creator, amount);

    Ok(())
}

// TODO: Implement withdraw_tips in issue #10
