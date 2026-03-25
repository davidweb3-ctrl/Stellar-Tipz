//! Event emission helpers for the Tipz contract.
//!
//! Every on-chain action that mutates meaningful state emits an event so that
//! off-chain indexers can follow contract activity without replaying every
//! transaction.
//!
//! ## Naming convention
//! Topic tuple  → `(Symbol, Symbol)`   – identifies the event type
//! Data tuple   → `(field, field, …)`  – the payload

use soroban_sdk::{symbol_short, Address, Env, String};

// ── Profile events ────────────────────────────────────────────────────────────

/// Topics : `("profile", "registered")`
/// Data   : `(owner: Address, username: String)`
pub fn emit_profile_registered(env: &Env, owner: &Address, username: &String) {
    env.events().publish(
        (symbol_short!("profile"), symbol_short!("register")),
        (owner.clone(), username.clone()),
    );
}

/// Topics : `("profile", "updated")`
/// Data   : `(owner: Address,)`
pub fn emit_profile_updated(env: &Env, owner: &Address) {
    env.events().publish(
        (symbol_short!("profile"), symbol_short!("updated")),
        (owner.clone(),),
    );
}

// ── Tip events ────────────────────────────────────────────────────────────────

/// Topics : `("tip", "sent")`
/// Data   : `(from: Address, to: Address, amount: i128)`
pub fn emit_tip_sent(env: &Env, from: &Address, to: &Address, amount: i128) {
    env.events().publish(
        (symbol_short!("tip"), symbol_short!("sent")),
        (from.clone(), to.clone(), amount),
    );
}

/// Topics : `("tip", "withdrawn")`
/// Data   : `(creator: Address, amount: i128, fee: i128)`
pub fn emit_tips_withdrawn(env: &Env, creator: &Address, amount: i128, fee: i128) {
    env.events().publish(
        (symbol_short!("tip"), symbol_short!("withdrawn")),
        (creator.clone(), amount, fee),
    );
}

// ── Credit score events ───────────────────────────────────────────────────────

/// Topics : `("credit", "updated")`
/// Data   : `(creator: Address, old_score: u32, new_score: u32)`
pub fn emit_credit_score_updated(env: &Env, creator: &Address, old_score: u32, new_score: u32) {
    env.events().publish(
        (symbol_short!("credit"), symbol_short!("updated")),
        (creator.clone(), old_score, new_score),
    );
}

// ── Admin events ──────────────────────────────────────────────────────────────

/// Topics : `("admin", "changed")`
/// Data   : `(old_admin: Address, new_admin: Address)`
pub fn emit_admin_changed(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (symbol_short!("admin"), symbol_short!("changed")),
        (old_admin.clone(), new_admin.clone()),
    );
}

// ── Fee events ────────────────────────────────────────────────────────────────

/// Topics : `("fee", "updated")`
/// Data   : `(old_bps: u32, new_bps: u32)`
pub fn emit_fee_updated(env: &Env, old_bps: u32, new_bps: u32) {
    env.events().publish(
        (symbol_short!("fee"), symbol_short!("updated")),
        (old_bps, new_bps),
    );
}

/// Topics : `("fee", "collector")`
/// Data   : `(new_collector: Address,)`
pub fn emit_fee_collector_updated(env: &Env, new_collector: &Address) {
    env.events().publish(
        (symbol_short!("fee"), symbol_short!("collector")),
        (new_collector.clone(),),
    );
}

// ── Batch events ──────────────────────────────────────────────────────────────

/// Topics : `("batch", "skipped")`
/// Data   : `(creator: Address,)`
pub fn emit_x_metrics_batch_skipped(env: &Env, creator: &Address) {
    env.events().publish(
        (symbol_short!("batch"), symbol_short!("skipped")),
        (creator.clone(),),
    );
}
