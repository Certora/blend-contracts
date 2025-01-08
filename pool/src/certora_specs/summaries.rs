use soroban_sdk::{Env, Address, Vec};
use crate::certora_specs::model;
use crate::{Request, Positions};
use crate::pool::{
    User,
    Pool,
    actions::Actions
};
use nondet::*;

/// This will not replace any recursive calls to the summarized function
macro_rules! apply_summary {
    (@mk_orig_module $id:ident, [$($prototype:tt)*], $( -> $ret:ty )?, $body:block) => {
        #[cfg(feature="certora")]
        pub(crate) mod $id {
            use super::*;
            #[allow(dead_code)]
            #[allow(unused_variables)]
            pub(crate) fn $id($($prototype)*) $( -> $ret )? $body
        }
    };
    (@mk_orig $( #[$meta:meta] )*, $id:ident, [$($prototype:tt)*], $( -> $ret:ty )?, $body:block) => {
        $( #[$meta] )*
        $vis fn $id($($prototype)*) $( -> $ret )? $body
    };
    (
        $( #[$meta:meta]  )*
        $vis:vis fn $id:ident ($($arg:ident : $arg_ty:ty),* $(,)?) $( -> $ret:ty )?
        $body:block
    ) => {
        #[cfg(feature="certora")]
        pub(crate) fn $id($($arg : $arg_ty),*) $( -> $ret )? {
            $crate::certora_specs::summaries::$id($($arg),*)
        }

        $crate::certora_specs::summaries::apply_summary!(
            @mk_orig_module $id, [$($arg : $arg_ty),*], $( -> $ret  )?, $body
        );

        #[cfg(not(feature="certora"))]
        $crate::certora_specs::summaries::apply_summary!(@mk_orig $( #[$meta] )*, $id, [$($arg : $arg_ty),*], $( -> $ret  )?, $body);
    };

    ($spec:ident, $old:ident,
        $( #[$meta:meta]  )*
        $vis:vis fn $id:ident (&mut $self:ident $( , )? $($arg:ident : $arg_ty:ty),* $(,)?) $( -> $ret:ty )?
        $body:block
    ) => {
        #[cfg(feature="certora")]
        pub(crate) fn $id(&mut $self, $($arg : $arg_ty),*) $( -> $ret )? {
            $self.$spec($($arg),*)
        }

        pub(crate) fn $old(&mut $self, $($arg : $arg_ty),*) $( -> $ret )? $body

        #[cfg(not(feature="certora"))]
        $crate::certora_specs::summaries::apply_summary!(@mk_orig $( #[$meta] )*, $id, [&mut $self, $($arg : $arg_ty),*], $( -> $ret  )?, $body);
    };

    ($spec:ident, $old:ident,
        $( #[$meta:meta]  )*
        $vis:vis fn $id:ident (&$self:ident $( , )? $($arg:ident : $arg_ty:ty),* $(,)?) $( -> $ret:ty )?
        $body:block
    ) => {
        #[cfg(feature="certora")]
        pub(crate) fn $id(&$self, $($arg : $arg_ty),*) $( -> $ret )? {
            $self.$spec($($arg),*)
        }

        pub(crate) fn $old(&$self, $($arg : $arg_ty),*) $( -> $ret )? $body

        #[cfg(not(feature="certora"))]
        $crate::certora_specs::summaries::apply_summary!(@mk_orig $( #[$meta] )*, $id, [&$self, $($arg : $arg_ty),*], $( -> $ret  )?, $body);
    };

    ($spec:ident, $old:ident,
        $( #[$meta:meta]  )*
        $vis:vis fn $id:ident ($self:ident $( , )? $($arg:ident : $arg_ty:ty),* $(,)?) $( -> $ret:ty )?
        $body:block
    ) => {
        #[cfg(feature="certora")]
        pub(crate) fn $id($self, $($arg : $arg_ty),*) $( -> $ret )? {
            $self.$spec($($arg),*)
        }

        pub(crate) fn $old($self, $($arg : $arg_ty),*) $( -> $ret )? $body

        #[cfg(not(feature="certora"))]
        $crate::certora_specs::summaries::apply_summary!(@mk_orig $( #[$meta] )*, $id, [$self, $($arg : $arg_ty),*], $( -> $ret  )?, $body);
    };
}
pub(crate) use apply_summary;

/// N.B. these summaries do not model storage outside of positions,
/// so they are unsound when considering any properties that depend on
/// other storage

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
actions_summary!(build_supply_action, from_state, {
    arb_positions!(from_state.positions, supply, amount, amount);
});

actions_summary!(build_withdraw_action, from_state, {
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

pub fn build_delete_liquidation_auction(e: &Env, from: &Address) {
}

pub(crate) fn positions_hf_under(e: &Env, pool: &mut Pool, positions: &Positions, hf: i128) -> bool {
    model::set_checked();
    nondet::nondet()
}

/// This spec is proved sound (modulo notes above) in spec::build_actions_from_request
pub fn build_actions_from_request(
    e: &Env,
    pool: &mut Pool,
    from: &Address,
    requests: Vec<Request>,
) -> (Actions, User, bool) {
    let mut actions = Actions::nondet();
    let state = User::load(e, from);
    let check_health = nondet::nondet();
    let mut new_state = User::nondet();

    // Auctions write positions
    if nondet::nondet() {
        new_state.store(e);
    }

    certora::require!(
        check_health || !crate::certora_specs::spec::should_check(&state, &new_state),
        "build action invariant"
    );

    (actions, new_state, check_health)
}
