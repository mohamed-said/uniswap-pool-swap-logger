pub mod converters;
pub mod loggers;

use anyhow::Result;

use loggers::swap_logger::SwapLogger;

#[tokio::main]
async fn main() -> Result<()> {
    // set the environment variable for your Infura endpoint URL
    if let Ok(websocket_infura_endpoint) = std::env::var("WEBSOCKET_INFURA_ENDPOINT") {
        // create a web3 object
        let web3 = web3::Web3::new(
            web3::transports::ws::WebSocket::new(&websocket_infura_endpoint).await?,
        );

        let contract_address = web3::types::H160::from_slice(
            &hex::decode("5777d92f208679db4b9778590fa3cab3ac9e2168")
                .unwrap_or_else(|_| panic!("Invalid Contract Address!"))
                .as_slice(),
        );

        // create a Contract object with the given contract address and Json ABI
        let contract = web3::contract::Contract::from_json(
            web3.eth(),
            contract_address,
            include_bytes!("contracts/uniswap_pool_abi.json"),
        )?;

        // Inject the contract and the web3 object into the logger
        let logger = SwapLogger::new(contract.clone(), web3.clone());
        // call the async method display logs
        logger.display_logs().await?;
    } else {
        panic!("INFURA endpoint env var is not set!");
    }

    Ok(())
}
