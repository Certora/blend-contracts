use crate::{
    dependencies::{BackstopClient, BlendTokenClient, TokenClient},
    emissions,
    errors::PoolError,
    reserve::Reserve,
    storage::{self, PoolConfig, ReserveConfig, ReserveData, ReserveMetadata},
};
use soroban_sdk::{Address, BytesN, Env, IntoVal, Symbol};

/// Initialize the pool
pub fn execute_initialize(
    e: &Env,
    admin: &Address,
    name: &Symbol,
    oracle: &BytesN<32>,
    bstop_rate: &u64,
    backstop_id: &BytesN<32>,
    b_token_hash: &BytesN<32>,
    d_token_hash: &BytesN<32>,
    blnd_id: &BytesN<32>,
    usdc_id: &BytesN<32>,
) -> Result<(), PoolError> {
    if storage::has_admin(e) {
        return Err(PoolError::AlreadyInitialized);
    }

    // ensure backstop is [0,1)
    if bstop_rate.clone() >= 1_000_000_000 {
        return Err(PoolError::InvalidPoolInitArgs);
    }

    storage::set_admin(e, admin);
    storage::set_name(e, name);
    storage::set_backstop(e, backstop_id);
    storage::set_pool_config(
        e,
        &PoolConfig {
            oracle: oracle.clone(),
            bstop_rate: bstop_rate.clone(),
            status: 1,
        },
    );
    storage::set_token_hashes(e, b_token_hash, d_token_hash);
    storage::set_blnd_token(e, blnd_id);
    storage::set_usdc_token(e, usdc_id);
    Ok(())
}

/// Initialize a reserve for the pool
pub fn initialize_reserve(
    e: &Env,
    from: &Address,
    asset: &BytesN<32>,
    metadata: &ReserveMetadata,
) -> Result<(), PoolError> {
    if from.clone() != storage::get_admin(e) {
        return Err(PoolError::NotAuthorized);
    }

    if storage::has_res(e, asset) {
        return Err(PoolError::AlreadyInitialized);
    }

    validate_reserve_metadata(e, metadata)?;

    let (b_token_hash, d_token_hash) = storage::get_token_hashes(e);
    // force consistent d and b token addresses based on underlying asset
    let deployer = e.deployer();
    let mut b_token_salt: BytesN<32> = asset.clone();
    let mut d_token_salt: BytesN<32> = asset.clone();
    b_token_salt.set(0, 0);
    d_token_salt.set(0, 1);
    let b_token_id = deployer
        .with_current_contract(&b_token_salt)
        .deploy(&b_token_hash);
    let d_token_id = deployer
        .with_current_contract(&d_token_salt)
        .deploy(&d_token_hash);

    let index = storage::push_res_list(e, asset);
    let reserve_config = ReserveConfig {
        b_token: b_token_id.clone(),
        d_token: d_token_id.clone(),
        index,
        decimals: metadata.decimals,
        c_factor: metadata.c_factor,
        l_factor: metadata.l_factor,
        util: metadata.util,
        max_util: metadata.max_util,
        r_one: metadata.r_one,
        r_two: metadata.r_two,
        r_three: metadata.r_three,
        reactivity: metadata.reactivity,
    };
    storage::set_res_config(e, asset, &reserve_config);
    let init_data = ReserveData {
        d_rate: 1_000_000_000,
        ir_mod: 1_000_000_000,
        d_supply: 0,
        b_supply: 0,
        last_time: e.ledger().timestamp(),
    };
    storage::set_res_data(e, asset, &init_data);

    // initialize tokens
    let asset_client = TokenClient::new(e, asset);
    let asset_symbol = asset_client.symbol();

    let b_token_client = BlendTokenClient::new(e, &b_token_id);
    let mut b_token_symbol = asset_symbol.clone();
    b_token_symbol.insert_from_bytes(0, "b".into_val(e));
    let mut b_token_name = asset_symbol.clone();
    b_token_name.insert_from_bytes(0, "Blend supply token for ".into_val(e));
    b_token_client.initialize(
        &e.current_contract_address(),
        &7,
        &b_token_name,
        &b_token_symbol,
    );
    b_token_client.init_asset(
        &e.current_contract_address(),
        &e.current_contract_id(),
        &asset,
        &index,
    );

    let d_token_client = BlendTokenClient::new(e, &d_token_id);
    let mut d_token_symbol = asset_symbol.clone();
    d_token_symbol.insert_from_bytes(0, "d".into_val(e));
    let mut d_token_name = asset_symbol.clone();
    d_token_name.insert_from_bytes(0, "Blend debt token for ".into_val(e));
    d_token_client.initialize(
        &e.current_contract_address(),
        &7,
        &b_token_name,
        &b_token_symbol,
    );
    d_token_client.init_asset(
        &e.current_contract_address(),
        &e.current_contract_id(),
        &asset,
        &index,
    );

    Ok(())
}

/// Update a reserve in the pool
pub fn execute_update_reserve(
    e: &Env,
    from: &Address,
    asset: &BytesN<32>,
    metadata: &ReserveMetadata,
) -> Result<(), PoolError> {
    if from.clone() != storage::get_admin(e) {
        return Err(PoolError::NotAuthorized);
    }

    validate_reserve_metadata(e, metadata)?;

    let pool_config = storage::get_pool_config(e);

    if pool_config.status == 2 {
        return Err(PoolError::InvalidPoolStatus);
    }

    let mut reserve = Reserve::load(&e, asset.clone());
    reserve.update_rates_and_mint_backstop(e, &pool_config)?;

    // only change metadata based configuration
    reserve.config.decimals = metadata.decimals;
    reserve.config.c_factor = metadata.c_factor;
    reserve.config.l_factor = metadata.l_factor;
    reserve.config.util = metadata.util;
    reserve.config.max_util = metadata.max_util;
    reserve.config.r_one = metadata.r_one;
    reserve.config.r_two = metadata.r_two;
    reserve.config.r_three = metadata.r_three;
    reserve.config.reactivity = metadata.reactivity;

    storage::set_res_config(e, asset, &reserve.config);
    reserve.set_data(e);

    Ok(())
}

// Update the pool emission information from the backstop
pub fn update_pool_emissions(e: &Env) -> Result<u64, PoolError> {
    let backstop_id = storage::get_backstop(e);
    let backstop_client = BackstopClient::new(e, &backstop_id);
    let next_exp = backstop_client.next_dist();
    let pool_eps = backstop_client.pool_eps(&e.current_contract_id()) as u64;
    emissions::update_emissions(e, next_exp, pool_eps)
}

fn validate_reserve_metadata(e: &Env, metadata: &ReserveMetadata) -> Result<(), PoolError> {
    if metadata.decimals > 18
        || metadata.c_factor > 1_0000000
        || metadata.l_factor > 1_0000000
        || metadata.util > 0_9500000
        || (metadata.max_util > 1_0000000 || metadata.max_util <= metadata.util)
        || (metadata.r_one > metadata.r_two || metadata.r_two > metadata.r_three)
        || (metadata.reactivity > 0_0005000)
    {
        return Err(PoolError::InvalidReserveMetadata);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        dependencies::{B_TOKEN_WASM, D_TOKEN_WASM},
        testutils::{create_reserve, create_token_contract, setup_reserve},
    };

    use super::*;
    use soroban_sdk::testutils::{Address as _, BytesN as _, Ledger, LedgerInfo};

    #[test]
    fn test_execute_initialize() {
        let e = Env::default();
        let pool_id = BytesN::<32>::random(&e);

        let admin = Address::random(&e);
        let name = Symbol::new(&e, "pool_name");
        let oracle = BytesN::<32>::random(&e);
        let bstop_rate = 0_100_000_000u64;
        let backstop_id = BytesN::<32>::random(&e);
        let b_token_hash = BytesN::<32>::random(&e);
        let d_token_hash = BytesN::<32>::random(&e);
        let blnd_id = BytesN::<32>::random(&e);
        let usdc_id = BytesN::<32>::random(&e);

        e.as_contract(&pool_id, || {
            let result = execute_initialize(
                &e,
                &admin,
                &name,
                &oracle,
                &1_000_000_000,
                &backstop_id,
                &b_token_hash,
                &d_token_hash,
                &blnd_id,
                &usdc_id,
            );
            assert_eq!(result, Err(PoolError::InvalidPoolInitArgs));

            execute_initialize(
                &e,
                &admin,
                &name,
                &oracle,
                &bstop_rate,
                &backstop_id,
                &b_token_hash,
                &d_token_hash,
                &blnd_id,
                &usdc_id,
            )
            .unwrap();

            assert_eq!(storage::get_admin(&e), admin);
            assert_eq!(storage::get_name(&e), name);
            let pool_config = storage::get_pool_config(&e);
            assert_eq!(pool_config.oracle, oracle);
            assert_eq!(pool_config.bstop_rate, bstop_rate);
            assert_eq!(pool_config.status, 1);
            assert_eq!(storage::get_backstop(&e), backstop_id);
            assert_eq!(
                storage::get_token_hashes(&e),
                (b_token_hash.clone(), d_token_hash.clone())
            );
            assert_eq!(storage::get_blnd_token(&e), blnd_id);
            assert_eq!(storage::get_usdc_token(&e), usdc_id);

            let result = execute_initialize(
                &e,
                &Address::random(&e),
                &name,
                &oracle,
                &bstop_rate,
                &backstop_id,
                &b_token_hash,
                &d_token_hash,
                &blnd_id,
                &usdc_id,
            );
            assert_eq!(result, Err(PoolError::AlreadyInitialized));
        });
    }

    #[test]
    fn test_initialize_reserve() {
        let e = Env::default();
        let pool_id = BytesN::<32>::random(&e);
        let bombadil = Address::random(&e);
        let sauron = Address::random(&e);
        let (asset_id_0, _) = create_token_contract(&e, &bombadil);
        let (asset_id_1, _) = create_token_contract(&e, &bombadil);

        let b_token_hash = e.install_contract_wasm(B_TOKEN_WASM);
        let d_token_hash = e.install_contract_wasm(D_TOKEN_WASM);

        let metadata = ReserveMetadata {
            decimals: 7,
            c_factor: 0_7500000,
            l_factor: 0_7500000,
            util: 0_5000000,
            max_util: 0_9500000,
            r_one: 0_0500000,
            r_two: 0_5000000,
            r_three: 1_5000000,
            reactivity: 100,
        };
        let mut bad_metadata = metadata.clone();
        bad_metadata.util = 1_0000000;
        e.as_contract(&pool_id, || {
            storage::set_token_hashes(&e, &b_token_hash, &d_token_hash);
            storage::set_admin(&e, &bombadil);

            initialize_reserve(&e, &bombadil, &asset_id_0, &metadata).unwrap();

            // if already exists blocks
            let result = initialize_reserve(&e, &bombadil, &asset_id_0, &metadata);
            assert_eq!(result, Err(PoolError::AlreadyInitialized));

            // only admin
            let result = initialize_reserve(&e, &sauron, &asset_id_1, &metadata);
            assert_eq!(result, Err(PoolError::NotAuthorized));

            // validates metadata
            let result = initialize_reserve(&e, &bombadil, &asset_id_1, &bad_metadata);
            assert_eq!(result, Err(PoolError::InvalidReserveMetadata));

            initialize_reserve(&e, &bombadil, &asset_id_1, &metadata).unwrap();

            let res_config_0 = storage::get_res_config(&e, &asset_id_0);
            let res_config_1 = storage::get_res_config(&e, &asset_id_1);
            assert_eq!(res_config_0.decimals, metadata.decimals);
            assert_eq!(res_config_0.c_factor, metadata.c_factor);
            assert_eq!(res_config_0.l_factor, metadata.l_factor);
            assert_eq!(res_config_0.util, metadata.util);
            assert_eq!(res_config_0.max_util, metadata.max_util);
            assert_eq!(res_config_0.r_one, metadata.r_one);
            assert_eq!(res_config_0.r_two, metadata.r_two);
            assert_eq!(res_config_0.r_three, metadata.r_three);
            assert_eq!(res_config_0.reactivity, metadata.reactivity);
            assert_eq!(res_config_0.index, 0);
            assert_eq!(res_config_1.index, 1);

            assert_ne!(res_config_0.b_token, res_config_1.b_token);
            assert_ne!(res_config_0.d_token, res_config_1.d_token);
        });
    }

    #[test]
    fn test_execute_update_reserve() {
        let e = Env::default();
        e.ledger().set(LedgerInfo {
            timestamp: 500,
            protocol_version: 1,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
        });

        let pool_id = BytesN::<32>::random(&e);
        let backstop_id = BytesN::<32>::random(&e);
        let bombadil = Address::random(&e);
        let sauron = Address::random(&e);

        let mut reserve_0 = create_reserve(&e);
        reserve_0.data.b_supply = 100_0000000;
        reserve_0.data.d_supply = 50_0000000;
        setup_reserve(&e, &pool_id, &bombadil, &mut reserve_0);

        let new_metadata = ReserveMetadata {
            decimals: 7,
            c_factor: 0_7500000,
            l_factor: 0_7500000,
            util: 0_7777777,
            max_util: 0_9500000,
            r_one: 0_0500000,
            r_two: 0_5000000,
            r_three: 1_5000000,
            reactivity: 105,
        };
        let mut bad_metadata = new_metadata.clone();
        bad_metadata.util = 1_0000000;

        e.ledger().set(LedgerInfo {
            timestamp: 10000,
            protocol_version: 1,
            sequence_number: 100,
            network_id: Default::default(),
            base_reserve: 10,
        });

        let pool_config = PoolConfig {
            oracle: BytesN::<32>::random(&e),
            bstop_rate: 0_100_000_000,
            status: 0,
        };
        e.as_contract(&pool_id, || {
            storage::set_admin(&e, &bombadil);
            storage::set_pool_config(&e, &pool_config);
            storage::set_backstop(&e, &backstop_id);

            let res_config_old = storage::get_res_config(&e, &reserve_0.asset);

            // validates metadata
            let result = execute_update_reserve(&e, &sauron, &reserve_0.asset, &new_metadata);
            assert_eq!(result, Err(PoolError::NotAuthorized));

            // validates metadata
            let result = execute_update_reserve(&e, &bombadil, &reserve_0.asset, &bad_metadata);
            assert_eq!(result, Err(PoolError::InvalidReserveMetadata));

            execute_update_reserve(&e, &bombadil, &reserve_0.asset, &new_metadata).unwrap();

            let res_config_updated = storage::get_res_config(&e, &reserve_0.asset);
            assert_eq!(res_config_updated.decimals, new_metadata.decimals);
            assert_eq!(res_config_updated.c_factor, new_metadata.c_factor);
            assert_eq!(res_config_updated.l_factor, new_metadata.l_factor);
            assert_eq!(res_config_updated.util, new_metadata.util);
            assert_eq!(res_config_updated.max_util, new_metadata.max_util);
            assert_eq!(res_config_updated.r_one, new_metadata.r_one);
            assert_eq!(res_config_updated.r_two, new_metadata.r_two);
            assert_eq!(res_config_updated.r_three, new_metadata.r_three);
            assert_eq!(res_config_updated.reactivity, new_metadata.reactivity);
            assert_eq!(res_config_updated.index, res_config_old.index);
            assert_eq!(res_config_updated.b_token, res_config_old.b_token);
            assert_eq!(res_config_updated.d_token, res_config_old.d_token);

            // validate interest was accrued
            let res_data = storage::get_res_data(&e, &reserve_0.asset);
            assert!(res_data.b_supply > 100_0000000);
            assert!(res_data.d_rate > 1_000_000_000);
            assert_eq!(res_data.last_time, 10000);
            assert!(
                TokenClient::new(&e, &reserve_0.config.b_token)
                    .balance(&Address::from_contract_id(&e, &backstop_id))
                    > 0
            );
        });
    }

    #[test]
    fn test_validate_reserve_metadata() {
        let e = Env::default();

        // valid
        let mut metadata = ReserveMetadata {
            decimals: 18,
            c_factor: 0_7500000,
            l_factor: 0_7500000,
            util: 0_5000000,
            max_util: 0_9500000,
            r_one: 0_0500000,
            r_two: 0_5000000,
            r_three: 1_5000000,
            reactivity: 100,
        };
        assert_eq!(validate_reserve_metadata(&e, &metadata), Ok(()));

        // decimals
        metadata.decimals = 19;
        assert_eq!(
            validate_reserve_metadata(&e, &metadata),
            Err(PoolError::InvalidReserveMetadata)
        );
        metadata.decimals = 18;

        // c_factor
        metadata.c_factor = 1_0000001;
        assert_eq!(
            validate_reserve_metadata(&e, &metadata),
            Err(PoolError::InvalidReserveMetadata)
        );
        metadata.c_factor = 0_7500000;

        // l_factor
        metadata.l_factor = 1_0000001;
        assert_eq!(
            validate_reserve_metadata(&e, &metadata),
            Err(PoolError::InvalidReserveMetadata)
        );
        metadata.l_factor = 0_7500000;

        // util
        metadata.util = 0_9500001;
        assert_eq!(
            validate_reserve_metadata(&e, &metadata),
            Err(PoolError::InvalidReserveMetadata)
        );
        metadata.util = 0_5000000;

        // max_util
        metadata.max_util = 1_0000001;
        assert_eq!(
            validate_reserve_metadata(&e, &metadata),
            Err(PoolError::InvalidReserveMetadata)
        );
        metadata.max_util = 0_9500000;

        // r
        metadata.r_one = 0_0500001;
        metadata.r_two = 0_0500000;
        metadata.r_three = 1_5000000;
        assert_eq!(
            validate_reserve_metadata(&e, &metadata),
            Err(PoolError::InvalidReserveMetadata)
        );
        metadata.r_one = 0_0500000;
        metadata.r_two = 0_5000001;
        metadata.r_three = 0_5000000;
        assert_eq!(
            validate_reserve_metadata(&e, &metadata),
            Err(PoolError::InvalidReserveMetadata)
        );
        metadata.r_one = 0_0500000;
        metadata.r_two = 0_5000000;
        metadata.r_three = 1_5000000;

        // reactivity
        metadata.reactivity = 5001;
        assert_eq!(
            validate_reserve_metadata(&e, &metadata),
            Err(PoolError::InvalidReserveMetadata)
        );
        metadata.reactivity = 100;
    }
}