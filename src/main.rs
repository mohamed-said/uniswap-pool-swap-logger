pub mod logger;
pub mod converters;

//use futures::StreamExt;
use logger::swap_logger::SwapLogger;

// amount0/DAI  -> 10^-18
// amount1/USDC -> 10*-6

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
	const WEBSOCKET_INFURA_ENDPOINT: &str =
		"wss://mainnet.infura.io/ws/v3/d9d5c206ac70483cb1187ceefcefb191";

	let web3 =
		web3::Web3::new(web3::transports::ws::WebSocket::new(WEBSOCKET_INFURA_ENDPOINT).await?);
	let contract_address = web3::types::H160::from_slice(
		&hex::decode("5777d92f208679db4b9778590fa3cab3ac9e2168").unwrap()[..],
	);

	let contract = web3::contract::Contract::from_json(
		web3.eth(),
		contract_address,
		include_bytes!("contracts/uniswap_pool_abi.json"),
	)?;

	let logger = SwapLogger::new(contract.clone(), web3.clone());
	let _ = logger.display_logs().await;

	Ok(())
}
