# Uniswap Pool Swap Logger
Simple logger for the swap events between crypto currencies using the open
source uniswap protocol on the Ethereum Blockchain.

Currently it only supports the [DAI/USDC](https://app.uniswap.org/explore/pools/ethereum/0x5777d92f208679DB4b9778590Fa3CAB3aC9e2168) pool.

I will be experimenting and adding more pool to learn more about Blockchain
development.

## Requirements
- Local Rust setup
- Infura endpoint

## Workflow
- Create a `web3` object using your Infura endpoint
- Create a contract object using the provided contract address (In this case it's the [DAI/USDC contract](https://etherscan.io/address/0x5777d92f208679DB4b9778590Fa3CAB3aC9e2168)
- Create a `SwapLogger` object, and inject the web3 and the contract objects into it.
- call the async function `display_logs`, which subscribes to the latest updates on the Ethereum block chain.

## Usage
* Run `cargo run` to run the app

## Test
* Run `cargo test` to run the test cases
