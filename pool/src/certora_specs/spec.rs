use soroban_sdk::{Address, Env};
use certora_soroban_macros::{declare_rules, rule};
use certora::assert;

use crate::{pool, Pool, PoolContract};


#[rule]
pub fn user_health(env: &Env, user: Address) {
    let positions = PoolContract::get_positions(env.clone(), user);
    let mut pool_state = pool::Pool::load(&env);
    let data: pool::PositionData = pool::PositionData::calculate_from_positions(&env, &mut pool_state, &positions);
    assert!(data.as_health_factor() > data.scalar);
    // assert!(false);
}
