#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as AddressTestTrait, BytesN as _},
    Address, BytesN, Env, Symbol,
};

mod common;
use crate::common::{
    b_token, create_wasm_pool_factory, d_token, generate_contract_id,
    lending_pool::{self, PoolConfig, PoolDataKey},
    PoolInitMeta,
};

#[test]
fn test_deploy() {
    let e = Env::default();
    let (_pool_factory_address, pool_factory_client) = create_wasm_pool_factory(&e);

    let wasm_hash = e.install_contract_wasm(lending_pool::WASM);

    let bombadil = Address::random(&e);

    let oracle = generate_contract_id(&e);
    let backstop_id = generate_contract_id(&e);
    let backstop_rate: u64 = 100000;
    let b_token_hash = e.install_contract_wasm(b_token::WASM);
    let d_token_hash = e.install_contract_wasm(d_token::WASM);
    let blnd_id = generate_contract_id(&e);
    let usdc_id = generate_contract_id(&e);

    let pool_init_meta = PoolInitMeta {
        b_token_hash: b_token_hash.clone(),
        d_token_hash: d_token_hash.clone(),
        backstop: backstop_id.clone(),
        pool_hash: wasm_hash.clone(),
        blnd_id: blnd_id.clone(),
        usdc_id: usdc_id.clone(),
    };
    pool_factory_client.initialize(&pool_init_meta);
    let name1 = Symbol::new(&e, "pool1");
    let name2 = Symbol::new(&e, "pool2");

    let salt = BytesN::<32>::random(&e);
    let deployed_pool_address_1 =
        pool_factory_client.deploy(&bombadil, &name1, &salt, &oracle, &backstop_rate);

    let salt = BytesN::<32>::random(&e);
    let deployed_pool_address_2 =
        pool_factory_client.deploy(&bombadil, &name2, &salt, &oracle, &backstop_rate);

    let zero_address = BytesN::from_array(&e, &[0; 32]);
    e.as_contract(&deployed_pool_address_1, || {
        let storage = e.storage();
        assert_eq!(
            storage
                .get::<_, Address>(&PoolDataKey::Admin)
                .unwrap()
                .unwrap(),
            bombadil.clone()
        );
        assert_eq!(
            storage
                .get::<_, BytesN<32>>(&PoolDataKey::Backstop)
                .unwrap()
                .unwrap(),
            backstop_id.clone()
        );
        assert_eq!(
            storage
                .get::<_, PoolConfig>(&PoolDataKey::PoolConfig)
                .unwrap()
                .unwrap(),
            PoolConfig {
                oracle: oracle,
                bstop_rate: backstop_rate,
                status: 1
            }
        );
        assert_eq!(
            storage
                .get::<_, (BytesN<32>, BytesN<32>)>(&PoolDataKey::TokenHash)
                .unwrap()
                .unwrap(),
            (b_token_hash, d_token_hash)
        );
        assert_eq!(
            storage
                .get::<_, BytesN<32>>(&PoolDataKey::BLNDTkn)
                .unwrap()
                .unwrap(),
            blnd_id.clone()
        );
        assert_eq!(
            storage
                .get::<_, BytesN<32>>(&PoolDataKey::USDCTkn)
                .unwrap()
                .unwrap(),
            usdc_id.clone()
        );
    });
    assert_ne!(deployed_pool_address_1, zero_address);
    assert_ne!(deployed_pool_address_2, zero_address);
    assert_ne!(deployed_pool_address_1, deployed_pool_address_2);
    assert!(pool_factory_client.is_pool(&deployed_pool_address_1));
    assert!(pool_factory_client.is_pool(&deployed_pool_address_2));
    assert!(!pool_factory_client.is_pool(&zero_address));
}
