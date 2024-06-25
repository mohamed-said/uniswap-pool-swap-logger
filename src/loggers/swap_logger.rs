use crate::{
	converters::{dai_usdc::DaiUsdc, Radix},
	loggers::{AmountError, AmountType, LoggerError},
};
use futures::StreamExt;
use web3::{contract::Contract, transports::WebSocket, Web3};

/// Manages the logging workflow for events of type "Swap"
pub struct SwapLogger {
	/// web3 instance holding the user's Infura endpoint.
	/// Facilitates communication to the block chain.
	web3_instance: Web3<WebSocket>,

	/// The contract you want to interact with.
	contract: Contract<WebSocket>,

	// Maximum number allowed for organization depth
    // TODO
	//_max_reorg: usize,
}

impl SwapLogger {
	pub fn new(
		contract: Contract<WebSocket>,
		web3_instance: Web3<WebSocket>,
	) -> Self {
		Self { web3_instance, contract }
	}

	/// filter swap events for a given block
	async fn filter_swap_events(
		&self,
		block_hash: web3::types::H256,
		swap_event_signature: web3::types::H256,
	) -> Result<Vec<web3::types::Log>, Box<dyn std::error::Error>> {
		let contract_address = self.contract.address();

		let res: Vec<web3::types::Log> = self
			.web3_instance
			.eth()
			.logs(
				web3::types::FilterBuilder::default()
					.block_hash(block_hash)
					.address(vec![contract_address])
					.topics(Some(vec![swap_event_signature]), None, None, None)
					.build(),
			)
			.await?;

		Ok(res)
	}

	/// subscribes to the latest updates and handles logs printing whenever a new block is ready
	pub async fn display_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
		let swap_event = self
			.contract
			.abi()
			.events_by_name("Swap")?
			.first()
			.ok_or_else(|| Box::new(LoggerError::FailedToRetrieveEvent))?;

		let swap_event_signature = swap_event.signature();

		// create a heads subscription
		let mut block_stream = self.web3_instance.eth_subscribe().subscribe_new_heads().await?;

		while let Some(Ok(block)) = block_stream.next().await {
			// returns a list of all the "Swap" events in the current block
			let swap_logs_in_block =
				self.filter_swap_events(block.hash.unwrap(), swap_event_signature).await?;
			let mut block_error = false;
			for log in swap_logs_in_block {
				// parse each Log's raw data into a more readable format
				match swap_event.parse_log(web3::ethabi::RawLog {
					topics: log.clone().topics,
					data: log.clone().data.0,
				}) {
					// print the requested log fields in the specified format
					Ok(parsed_log) => Self::print_log_formatted(parsed_log)?,
					// if data was invalid, keep a record of the incident
					Err(err) => {
						println!("In block: {:?}", &block.hash);
						println!("Error: {:?}", err);
						if !block_error {
							block_error = true;
						}
					},
				};
			}
		}

		Ok(())
	}

	/// Prints the log's data to stdout with the requested format
	/// {
	///     The amounts as decimal numbers,
	///     The "direction" of the swap,
	///     The sender address,
	///     The receiver address,
	/// }
	fn print_log_formatted(log: web3::ethabi::Log) -> Result<(), Box<dyn std::error::Error>> {
		let mut amount0: String = String::new();
		let mut amount1: String = String::new();

		println!("{{");
		for param in log
			.params
			.iter()
			.filter(|&p| ["sender", "recipient", "amount0", "amount1"].contains(&p.name.as_str()))
		{
			match param.name.as_str() {
				"amount0" => {
					amount0 = DaiUsdc::amount_to_decimal(
						param.value.to_string().as_str(),
						Radix::Base16.to_uint(),
						&AmountType::DAI,
					)?;
					println!("\t{}: {}", param.name, amount0);
				},
				"amount1" => {
					amount1 = DaiUsdc::amount_to_decimal(
						param.value.to_string().as_str(),
						Radix::Base16.to_uint(),
						&AmountType::USDC,
					)?;
					println!("\t{}: {}", param.name, amount1);
				},
				_ => {
					println!("\t{}: {}", param.name, param.value);
				},
			}
		}
		println!("\tdirection: {}", Self::swap_direction(amount0, amount1)?);
		println!("}}");

		Ok(())
	}

	/// determine swap direction (DAI -> USDC) or (USDC -> DAI)
	fn swap_direction(
		amount0: String,
		amount1: String,
	) -> Result<String, Box<dyn std::error::Error>> {
		if amount0.starts_with('-') && !amount1.starts_with('-') {
			return Ok(format!("{} -> {}", AmountType::USDC, AmountType::DAI));
		} else if amount1.starts_with('-') && !amount0.starts_with('-') {
			return Ok(format!("{} -> {}", AmountType::DAI, AmountType::USDC));
		}

		Err(Box::new(AmountError::AllAmountsAreNegative))
	}
}
