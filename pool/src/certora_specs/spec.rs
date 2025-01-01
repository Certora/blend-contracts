use soroban_sdk::{Address, Env, Vec};
use certora_soroban_macros::{declare_rules, rule};
use certora::{assert, require, satisfy};

use crate::{constants::IS_CHECK_HEALTH_TRUE, pool::{self, execute_submit, User}, Pool, PoolContract, Request};
use crate::constants::GHOST_HEALTH_IS_CHECKED;


#[rule]
pub fn submit_checks_health(env: &Env, from: Address, spender: Address, to: Address, requests: Vec<Request>) {
    let positions = execute_submit(&env, &from, &spender, &to, requests);

    certora::assert!(unsafe{GHOST_HEALTH_IS_CHECKED} == false);
    // certora::assert!(unsafe {IS_CHECK_HEALTH_TRUE} == false || unsafe{GHOST_HEALTH_IS_CHECKED} == true);
    // certora::assert!(false);
    
}



// #[rule]
// pub fn user_health(env: &Env, user: Address) {
//     let positions = PoolContract::get_positions(env.clone(), user);
//     let mut pool_state = pool::Pool::load(&env);
//     let data: pool::PositionData = pool::PositionData::calculate_from_positions(&env, &mut pool_state, &positions);
//     assert!(data.as_health_factor() > data.scalar);
//     // assert!(false);
// }

// #[rule]
// pub fn user_healthy_base(env: &Env, from: Address) {
//     let user = User::load(env, &from);

//     let sum_collateral = user.positions.collateral.iter().fold(0i128, |acc, (_, value)| acc + value);
//     let sum_liability = user.positions.liabilities.iter().fold(0i128, |acc, (_, value)| acc + value);
    
//     assert!(sum_collateral > sum_liability);
    
// }

// #[rule]
// pub fn sanity_submit(env: &Env, from: Address, spender: Address, to: Address, requests: Vec<Request>) {
//     let user = User::load(env, &from);

//     let sum_collateral_before = user.positions.collateral.iter().fold(0i128, |acc, (_, value)| acc + value);
//     let sum_liability_before = user.positions.liabilities.iter().fold(0i128, |acc, (_, value)| acc + value);
    
//     require!(sum_collateral_before > sum_liability_before, "collateral > liability before");

//     let positions = execute_submit(&env, &from, &spender, &to, requests);
    
//     let sum_collateral_after = positions.collateral.iter().fold(0i128, |acc, (_, value)| acc + value);
//     let sum_liability_after = positions.liabilities.iter().fold(0i128, |acc, (_, value)| acc + value);

//     assert!(sum_collateral_after > sum_liability_after);
    
// }
