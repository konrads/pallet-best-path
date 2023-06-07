# Best Path pallet

[![test](https://github.com/konrads/pallet-best-path/workflows/test/badge.svg)](https://github.com/konrads/pallet-best-path/actions/workflows/test.yml)

Pallet sourcing of best (most profitable) trade paths, even when no direct path is supplied.

- parametrizable longest path algorithm
- OCW worker fetching latest price data, calculating the longest (best) path algorithm, recording results via extrinsics
- extrinsic interface for:
  - internal usage, eg. keeping record of latest price data
  - admin/root, eg. addition/deletion of price pairs by provider

## Details

This pallet comprises following structure:

```
  *
  |
  +---- lib.rs
  |
  +---- types.rs
  |
  +---- utils.rs
  |
  +---- benchmarking.rs
  |
  +---- weights.rs
  |
  +----+ price_provider
       |
       +----- crypto_compare
```

- [lib.rs](src/lib.rs) - OCW mechanisms and extrinsic APIs
- [types.rs](src/types.rs) - types utilized throughout
- [utils.rs](src/utils.rs) - common utils
- [benchmarking.rs](src/benchmarking.rs) and [weights.rs](src/weights.rs) - weights produced by benchmarking
- [crypto-compare price provider](src/price_provider/crypto_compare.rs) - sample price data oracle, based on OCW example

### Longest path algorithm

For longest path calculations, Floyd-Warshall algorithm was chosen for its ability to calculate both shortest and longest paths across all vertices.

For multiplication based weights, it's been noticed that product maximisation is equivalent to maximisation of log of weights, as per: $x*y = 2^{log2(x) + log2(y)}$.

For longest paths, weights have been multiplied by `-1` and hence reused in shortest path algorithm.

_NOTE:_ Floyd-Warshall can detect negative path cycles (ie. infinite arbitrage opportunities), which cause the latest price update to be ignored. Potential FIXME - remove offending edge to remove negative cycles...

### OCW

OCW triggers price fetching, best path calculation, compares with currently stored best path and issues updates via unsigned root origin extrinsic.

OCW trigger is guarded by an `OffchainTriggerFreq` constant ensuring price fetching doesn't happen too frequently, as well as `AcceptNextOcwTxAt` storage / `UnsignedTxAcceptFreq` constant ensuring unsigned transactions are received too frequently.

### API

- whitelisted (none origin)
  - `ocw_submit_best_paths_changes()` - for price change delta submissions from onchain
- admin (root origin)
  - `add_whitelisted_offchain_authority()` - to record whitelisted address that identifies as a known OCW worker issuing the unsigned transactions
  - `submit_monitored_pairs()` - for submission of to-be-monitored price pairs by provider

### Constants

- `OffchainTriggerDelay` - rate limits OCW trigger
- `UnsignedPriority` - sets unsigned transaction priority (to play nicely with other pallets)
- `PriceChangeTolerance` - sets acceptable price change tolerance, if not breached, on-chain prices aren't updated

## Usage

```bash
make build                      # build pallet/runtime
make test                       # verify build
make clippy                     # ensure code quality

make run                        # start the project
make run-node                   # start the project, from pre-compiled node
sleep 15 && make populate-keys  # in another window, upload whitelist keys once the node stabilizes

```

Go to [https://polkadot.js.org/apps/#/explorer](https://polkadot.js.org/apps/#/explorer). Ensure you've switched to local node:

<img src="/docs/img/switch-network.png" alt="Switch to local node" width="30%">

Add offchain authority (potentially temporary step, might get automated). Click on `+ Add Account`, enter the mnemonic `clip organ olive upper oak void inject side suit toilet stick narrow`:

<img src="/docs/img/add-offchain-authority-account.png" alt="Add offchain authority account" width="70%">

Proceed by clicking `Next`, name the newly created to eg. `OCW_ADMIN`, click `Next`, `Save`.

Go to extrinsic menu:

<img src="/docs/img/extrinsic-menu.png" alt="Go to extrinsic menu" width="40%">

Add the newly created authority to the whitelist. Note, this is done via sudo call (requires `sudo` pallet): <img src="/docs/img/add-whitelisted-offchain-authority.png" alt="Add whitelisted offchain authority" width="70%">

Submit monitored currency-provider pairs via a sudo call:

<img src="/docs/img/add-monitored-pairs.png" alt="Add BTC-USDT" width="70%">

Make sure to submit transaction:

<img src="/docs/img/submit-transaction.png" alt="Submit transaction" width="60%">

Validate algorithm produces DOT-USDT pair. Note, in case of negative graph cycles (which produces infinite arbitrage opportunities), the algorithm discards the price updates.

Firstly, monitor logs for price updates.

![View price update logs](/docs/img/price-update-logs.png)

Check onchain storage for DOT-USDT price update. Go to chain state menu:

<img src="/docs/img/chain-state-menu.png" alt="Go to chain state menu" width="20%">

And validate new trading path for DOT-USDT pair:

<img src="/docs/img/chain-state-DOT-USDT.png" alt="DOT-USDT chain state" width="70%">

## Snags/TODOs

| Stage | Description                                                                                                                                                | Status |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| 1     | Benchmark weights, including API allowing for extrinsics with vector parameters                                                                            | 𐄂      |
| 1     | Consider abstracting Cost (aka Amount) from Balance to allow for more elaborate cost calculations, including transaction fees, slippage, etc               | 𐄂      |
| 1     | Bootstrap storage to allow for configuration for price pairs per provider (currently needs root origin extrinsic invocations)                              | 𐄂      |
| 1     | Investigate keys bootstrap (currently done with curl, see above)                                                                                           | 𐄂      |
| 2     | Construct Typescript client lib                                                                                                                            | 𐄂      |
| 3     | Revise mechanisms for submission of internal price data, ie. with what origin, signed/unsigned transaction, signed/unsigned payload, signed with a refund? | 𐄂      |

## Outstanding questions

- Benchmarking utilizes `--wasm-execution interpreted-i-know-what-i-do` as default `compiled` isn't available...

## Substrate runtime wiring

Runtime utilizing this pallet is exemplified in [substrate-node-playground](https://github.com/konrads/substrate-node-playground).

License: Unlicensed
