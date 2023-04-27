import { Server, Address } from "soroban-client";
import { randomBytes } from "crypto";
import { Config } from "../config.js";
import { createTxBuilder, signPrepareAndSubmitTransaction } from "../utils.js";
import * as backstop from "../operations/backstop.js";
import * as token from "../operations/token.js";
import * as pool from "../operations/pool.js";
import * as poolFactory from "../operations/poolFactory.js";

/**
 * @param {Server} stellarRpc
 * @param {Config} config
 * @param {string} poolName
 */
export async function deployAndSetupPool(stellarRpc, config, poolName) {
  let bombadil = config.getAddress("bombadil");
  let network = config.network.passphrase;
  let backstopTakeRate = "10000000"; // 10% - 9 decimals

  console.log("START Create Pool: ", poolName);
  let txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  txBuilder.addOperation(
    poolFactory.createDeployPool(
      config,
      bombadil.publicKey(),
      randomBytes(32),
      backstopTakeRate,
      poolName
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  config.writeToFile();
  console.log("deployed ", poolName, "\n");

  console.log("START Initialize Reserves");
  txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  let reserveMetaXLM = pool.createDefaultReserveMetadata();
  reserveMetaXLM.c_factor = 850_0000;
  reserveMetaXLM.c_factor = 800_0000;
  reserveMetaXLM.util = 500_0000;
  txBuilder.addOperation(
    pool.createInitReserve(
      poolName,
      config,
      bombadil.publicKey(),
      "XLM",
      reserveMetaXLM
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  config.writeToFile();
  console.log("created reserve for XLM in ", poolName, "\n");

  txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  let reserveMetaUSDC = pool.createDefaultReserveMetadata();
  reserveMetaUSDC.c_factor = 975_0000;
  reserveMetaUSDC.l_factor = 950_0000;
  reserveMetaUSDC.util = 850_0000;
  reserveMetaUSDC.r_one = 30_0000;
  reserveMetaUSDC.r_two = 200_0000;
  reserveMetaUSDC.r_three = 1_000_0000;
  txBuilder.addOperation(
    pool.createInitReserve(
      poolName,
      config,
      bombadil.publicKey(),
      "USDC",
      reserveMetaUSDC
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  config.writeToFile();
  console.log("created reserve for USDC in ", poolName, "\n");

  txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  let reserveMetaETH = pool.createDefaultReserveMetadata();
  reserveMetaETH.util = 700_0000;
  txBuilder.addOperation(
    pool.createInitReserve(
      poolName,
      config,
      bombadil.publicKey(),
      "WETH",
      reserveMetaETH
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  config.writeToFile();
  console.log("created reserve for WETH in ", poolName, "\n");

  console.log("DONE: deployed pool ", poolName, "\n");

  console.log("START: Enable emissions to both supplied XLM and borrowed USDC");
  txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  let reserveEmissionsMetadata = [
    {
      res_index: 0, // XLM
      res_type: 1, // b_token
      share: 0.4e7, // 40%
    },
    {
      res_index: 1, // USDC
      res_type: 0, // d_token
      share: 0.6e7, // 60%
    },
  ];
  txBuilder.addOperation(
    pool.createSetEmissions(
      config,
      poolName,
      bombadil.publicKey(),
      reserveEmissionsMetadata
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  console.log("DONE: Setup pool emissions");
}

/**
 * Deposit funds into the pools backstop, activate the pool,
 * and add it to the reward zone for the backstop
 * @param {Server} stellarRpc
 * @param {Config} config
 * @param {string} poolName
 */
export async function setupPoolBackstop(stellarRpc, config, poolName) {
  let network = config.network.passphrase;
  let bombadil = config.getAddress("bombadil");
  let frodo = config.getAddress("frodo");
  let backstopToken = config.getContractId("BLNDUSDC");

  console.log("Starting pool backstop setup\n");
  console.log("START: Mint frodo required tokens...");
  let txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  txBuilder.addOperation(
    token.createMint(
      backstopToken,
      bombadil.publicKey(),
      frodo.publicKey(),
      BigInt(2_000_000e7)
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  console.log("minted backstop tokens...");
  console.log("DONE: minted frodo required tokens\n");

  console.log("START: Deposit into backstop");
  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(
    backstop.createDeposit(
      config,
      poolName,
      frodo.publicKey(),
      BigInt(1_000_000e7)
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("DONE: Deposited into backstop\n");

  console.log("START: Active pool");
  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(pool.createUpdateState(config, poolName));
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("DONE: Activated Pool\n");

  console.log("START: Move pool into reward zone");
  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(
    backstop.createAddToRewardZone(
      config,
      config.getContractId(poolName),
      config.getContractId(poolName)
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("DONE: Moved pool into reward zone\n");
}

/**
 * @param {Server} stellarRpc
 * @param {Config} config
 * @param {string} poolName
 */
export async function distribute(stellarRpc, config, poolName) {
  let network = config.network.passphrase;
  let bombadil = config.getAddress("bombadil");
  let blndToken = config.getContractId("BLND");
  let backstopId = config.getContractId("backstop");

  console.log("START: Start distribution for backstop and pool\n");
  let txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  txBuilder.addOperation(backstop.createDistribute(config));
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  console.log("backstop distributed...");

  txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  txBuilder.addOperation(
    token.createTransfer(
      blndToken,
      bombadil.publicKey(),
      Address.contract(Buffer.from(backstopId, "hex")).toString(),
      BigInt(500_000e7)
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  console.log("extra tokens given to backstop...");

  txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  txBuilder.addOperation(pool.createUpdateEmissions(config, poolName));
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  console.log("pool distributed...");

  console.log("DONE: backstop and pool emissions started\n");
}

/**
 * @param {Server} stellarRpc
 * @param {Config} config
 * @param {string} poolName
 */
export async function addWhale(stellarRpc, config, poolName) {
  let network = config.network.passphrase;
  let bombadil = config.getAddress("bombadil");
  let frodo = config.getAddress("frodo");

  console.log("START: Setting up pool with USDC, WETH XLM positions");
  let txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  txBuilder.addOperation(
    token.createMint(
      config.getContractId("USDC"),
      bombadil.publicKey(),
      frodo.publicKey(),
      BigInt(1_000_000e7)
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  console.log("minted USDC...\n");

  txBuilder = await createTxBuilder(stellarRpc, network, bombadil);
  txBuilder.addOperation(
    token.createMint(
      config.getContractId("WETH"),
      bombadil.publicKey(),
      frodo.publicKey(),
      BigInt(100e7)
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    bombadil
  );
  console.log("minted WETH...\n");

  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(
    pool.createSupply(
      config,
      poolName,
      frodo.publicKey(),
      "XLM",
      BigInt(5000e7)
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("supplied XLM...\n");

  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(
    pool.createSupply(config, poolName, frodo.publicKey(), "WETH", BigInt(5e7))
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("supplied WETH...\n");

  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(
    pool.createSupply(
      config,
      poolName,
      frodo.publicKey(),
      "USDC",
      BigInt(10_000e7)
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("supplied USDC...\n");

  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(
    pool.createBorrow(
      config,
      poolName,
      frodo.publicKey(),
      "XLM",
      BigInt(3000e7),
      frodo.publicKey()
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("borrowed XLM...\n");

  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(
    pool.createBorrow(
      config,
      poolName,
      frodo.publicKey(),
      "USDC",
      BigInt(8500e7),
      frodo.publicKey()
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("borrowed USDC...\n");

  txBuilder = await createTxBuilder(stellarRpc, network, frodo);
  txBuilder.addOperation(
    pool.createBorrow(
      config,
      poolName,
      frodo.publicKey(),
      "WETH",
      BigInt(2e7),
      frodo.publicKey()
    )
  );
  await signPrepareAndSubmitTransaction(
    stellarRpc,
    network,
    txBuilder.build(),
    frodo
  );
  console.log("borrowed WETH...\n");

  console.log("DONE: Set up pool with USDC, WETH and XLM positions\n");
}
