use soroban_sdk::{Env, Address, Vec};
use crate::certora::model;
use crate::{Request, Positions};
use crate::pool::{
    User,
    Pool,
    actions::Actions
};
use nondet::*;

/// N.B. these summaries do not model storage outside of positions,
/// so they are unsound when considering any properties that depend on
/// other storage
///
macro_rules! arb_positions {
    ($positions:expr, $map:ident, $amount:ident, $orig:ident, $new:expr) => {
        {
            let reserve: u32 = nondet::nondet();
            let $amount: i128  = nondet::nondet();
            certora::require!($amount > 0, "summary");
            let b = &mut $positions;
            let $orig = b.$map.get(reserve).unwrap_or(0);
            b.$map.set(reserve, $new);
        }
    };

    ($positions:expr, $map:ident, $amount:ident, $new:expr) => {
        {
            let reserve: u32 = nondet::nondet();
            let $amount: i128  = nondet::nondet();
            certora::require!($amount > 0, "summary");
            let b = &mut $positions;
            b.$map.set(reserve, $new);
        }
    };
}

macro_rules! actions_summary {
    ($f:ident, $state:ident, $body:block) => {
        #[allow(unused)]
        pub fn $f(_e: &Env,
                  _pool: &mut Pool,
                  _from: &Address,
                  _address: &Address,
                  _amount: i128,
                  $state: &mut User,
                  _actions: &mut Actions)
            $body
    };
    ($f:ident, $e:ident, $state:ident, $body:block) => {
        #[allow(unused)]
        pub fn $f($e: &Env,
                  _pool: &mut Pool,
                  _from: &Address,
                  _address: &Address,
                  _amount: i128,
                  $state: &mut User,
                  _actions: &mut Actions)
            $body
    }
}

//TODO havoc actions vs. nondet
//TODO need to havoc supply at all for repay etc etc?
actions_summary!(build_supply, from_state, {
    arb_positions!(from_state.positions, supply, amount, amount);
});

actions_summary!(build_withdraw, from_state, {
    arb_positions!(from_state.positions, supply, amount, amount);
});

actions_summary!(build_supply_collateral, from_state, {
    arb_positions!(from_state.positions, collateral, amount, orig, orig + amount);
});

actions_summary!(build_withdraw_collateral, from_state, {
    arb_positions!(from_state.positions, collateral, amount, orig, orig - amount);
});

actions_summary!(build_borrow, from_state, {
    arb_positions!(from_state.positions, liabilities, amount, orig, orig + amount);
});

actions_summary!(build_repay, from_state, {
    arb_positions!(from_state.positions, liabilities, amount, orig, orig - amount);
});

// NB not modeling pool cache effects here
actions_summary!(build_fill_user_liquidation_auction, e, from_state, {
    from_state.positions.collateral = nondet::nondet();
    from_state.positions.liabilities = nondet::nondet();
    from_state.store(e);
});

actions_summary!(build_fill_bad_debt_auction, e, from_state, {
    from_state.positions.collateral = nondet::nondet();
    from_state.positions.liabilities = nondet::nondet();
    from_state.store(e);
});

actions_summary!(build_fill_interest_auction, _from_state, {
});

pub(crate) fn build_delete_liquidation_auction(_e: &Env, _from: &Address) {
}

pub(crate) fn positions_hf_under(_e: &Env, _pool: &mut Pool, _positions: &Positions, _hf: i128) -> bool {
    model::set_checked();
    nondet::nondet()
}

/// This spec is proved sound (modulo notes above) in spec::build_actions_from_request
pub fn build_actions_from_request(
    e: &Env,
    _pool: &mut Pool,
    from: &Address,
    _requests: Vec<Request>,
) -> (Actions, User, bool) {
    let actions = Actions::nondet();
    let state = User::load(e, from);
    let check_health = nondet::nondet();
    let new_state = User::nondet();

    // Auctions write positions
    if nondet::nondet() {
        new_state.store(e);
    }

    certora::require!(
        check_health || !crate::certora::spec::should_check(&state, &new_state),
        "build action invariant"
    );

    (actions, new_state, check_health)
}
