use certora;
use certora_soroban_macros::rule;
use crate::{
    certora::model,
    pool::{self, User},
    Request
};
use soroban_sdk::{Address, Env};

pub fn should_check(orig_state: &User, state: &User) -> bool {
    let idx = model::skolem_i();
    let collateral_decrease_idx =
        state.positions.collateral.get(idx).unwrap_or(0) < orig_state.positions.collateral.get(idx).unwrap_or(0);

    let liabilities_increase_idx =
        state.positions.liabilities.get(idx).unwrap_or(0) > orig_state.positions.liabilities.get(idx).unwrap_or(0);

    state.has_liabilities() && (collateral_decrease_idx || liabilities_increase_idx)
}

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

    certora::assert!(check_health || !should_check(&orig_state, &new_state));
}


#[rule]
pub fn build_actions_from_request_sanity(env: &Env, from: Address, requests: soroban_sdk::Vec<Request>) {
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

    certora::satisfy!(true);
}

#[rule]
pub fn user_health(env: &Env, user: Address, req: soroban_sdk::Vec<Request>) {
    use pool::submit::execute_submit;
    model::init();

    let orig_state = User::load(env, &user);
    let addr = &user.clone();
    execute_submit(
        env,
        addr,
        addr,
        addr,
        req,
    );
    let new_state = User::load(env, &user);
    certora::assert!(model::get_checked() || !should_check(&orig_state, &new_state));
}

#[rule]
pub fn user_health_sanity(env: &Env, user: Address, req: soroban_sdk::Vec<Request>) {
    use pool::submit::execute_submit;
    model::init();

    let orig_state = User::load(env, &user);
    let addr = &user.clone();
    execute_submit(
        env,
        addr,
        addr,
        addr,
        req,
    );
    let new_state = User::load(env, &user);
    certora::satisfy!(true);
}
