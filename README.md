# uniswap
Exchanging Crypto Currencies

* Run `cargo test` to run the unit tests

## Workflow
- Create a `web3` object using your Infura endpoint
- Create a contract object using the provided contract address
- Create a `SwapLogger` object, and inject the web3 and the contract objects
  into it.
- call the async function `display_logs`, which subscribes to the latest
  updates on the Ethereum block chain.
    - This function keep listening for updates until a new block is mined, then
      extract the required event information from the block (e.g. transactions,
      events, ..etc)
