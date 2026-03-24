//! Admin operations for the Tipz contract.
//!
//! - Contract initialization
//! - Fee management
//! - Admin role transfer

use soroban_sdk::{Address, Env};

use crate::errors::ContractError;
use crate::storage;

/// Initialize the contract. Can only be called once.
pub fn initialize(
    env: &Env,
    admin: &Address,
    fee_collector: &Address,
    fee_bps: u32,
    native_token: &Address,
) -> Result<(), ContractError> {
    if storage::is_initialized(env) {
        return Err(ContractError::AlreadyInitialized);
    }

    if fee_bps > 1000 {
        return Err(ContractError::InvalidFee);
    }

    storage::set_initialized(env);
    storage::set_admin(env, admin);
    storage::set_fee_collector(env, fee_collector);
    storage::set_fee_bps(env, fee_bps);
    storage::set_native_token(env, native_token);

    Ok(())
}

// TODO: Implement set_fee, set_fee_collector, set_admin in issues #20, #21, #22
