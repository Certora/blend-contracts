use certora;
use certora_soroban_macros::rule;
use crate::{
    spec::{model, summaries},
    pool::{self, User},
    Request
};
use soroban_sdk::{Address, Env};

/// Return `true` iff `state` has liabilities and
/// either a collateral at `model::skolem_i()` decreased relative to `orig_state`
/// or a liability at `model::skolem_i()` increased relative to `orig_state`
pub fn should_check(orig_state: &User, state: &User) -> bool {
    let idx = model::skolem_i();
    let collateral_decrease_idx =
        state.positions.collateral.get(idx).unwrap_or(0) < orig_state.positions.collateral.get(idx).unwrap_or(0);

    let liabilities_increase_idx =
        state.positions.liabilities.get(idx).unwrap_or(0) > orig_state.positions.liabilities.get(idx).unwrap_or(0);

    state.has_liabilities() && (collateral_decrease_idx || liabilities_increase_idx)
}

/// Check the soundness of the summary for `build_actions_from_request`
#[rule]
pub fn build_actions_from_request(env: &Env, from: Address, requests: soroban_sdk::Vec<Request>) {
    model::init();

    let mut pool = pool::Pool::load(env);
    let orig_state = User::load(env, &from);

    let (_, new_state, check_health) =
        pool::actions::build_actions_from_request::build_actions_from_request(
            env,
            &mut pool,
            &from,
            requests,
        );

    // Assume the postcondition we _assume_ in the summary is in fact true
    certora::assert!(
        summaries::build_actions_from_request_postcondition(check_health, &orig_state, &new_state)
    );
}

/// The main user health property:
/// if we compare the User state of `user` before & after calling `execute_submit`,
/// then either the user's positions changed in an obviously healthy direction OR
/// we explicitly checked that they are healthy
#[rule]
pub fn user_health(env: &Env, user: Address, spender: Address, to: Address, req: soroban_sdk::Vec<Request>) {
    use pool::submit::execute_submit;
    model::init();

    let orig_state = User::load(env, &user);
    execute_submit(
        env,
        &user,
        &spender,
        &to,
        req,
    );
    let new_state = User::load(env, &user);

    certora::assert!(model::get_checked() || !should_check(&orig_state, &new_state));
}

/// Check that the rule `build_actions_from_request` is not vacuously true
#[rule]
pub fn build_actions_from_request_sanity(env: &Env, from: Address, requests: soroban_sdk::Vec<Request>) {
    model::init();

    let mut pool = pool::Pool::load(env);
    let _orig_state = User::load(env, &from);

    let (_, _new_state, _check_health) =
        pool::actions::build_actions_from_request::build_actions_from_request(
            env,
            &mut pool,
            &from,
            requests,
        );

    certora::satisfy!(true);
}

/// Check that the rule `user_sanity` is not vacuously true
#[rule]
pub fn user_health_sanity(env: &Env, user: Address, spender: Address, to: Address, req: soroban_sdk::Vec<Request>) {
    use pool::submit::execute_submit;
    model::init();

    let _orig_state = User::load(env, &user);
    execute_submit(
        env,
        &user,
        &spender,
        &to,
        req,
    );
    let _new_state = User::load(env, &user);
    certora::satisfy!(true);
}
