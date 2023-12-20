pub mod converters;
pub mod logger;

use logger::swap_logger::SwapLogger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// set the environment variable for your Infura endpoint URL
	if let Ok(websocket_infura_endpoint) = std::env::var("WEBSOCKET_INFURA_ENDPOINT") {
		let web3 = web3::Web3::new(
			web3::transports::ws::WebSocket::new(&websocket_infura_endpoint).await?,
		);
		let contract_address = web3::types::H160::from_slice(
			&hex::decode("5777d92f208679db4b9778590fa3cab3ac9e2168").unwrap()[..],
		);

		let contract = web3::contract::Contract::from_json(
			web3.eth(),
			contract_address,
			include_bytes!("contracts/uniswap_pool_abi.json"),
		)?;

		// Inject the contract and the web3 object into the logger
		let max_reorg: usize = 5;
		let logger = SwapLogger::new(contract.clone(), web3.clone(), max_reorg);
		logger.display_logs().await?;
	} else {
		panic!("INFURA endpoint env var is not set!");
	}

	Ok(())
}
