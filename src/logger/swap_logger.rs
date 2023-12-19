use std::collections::BTreeSet;

use crate::{
	converters::dai_usdc::DaiUsdc,
	logger::{AmountError, AmountType, LoggerError},
};
use futures::StreamExt;
use web3::{contract::Contract, transports::WebSocket, Web3};

pub struct SwapLogger {
	web3_instance: Web3<WebSocket>,
	contract: Contract<WebSocket>,
}

impl SwapLogger {
	pub fn new(contract: Contract<WebSocket>, web3_instance: Web3<WebSocket>) -> Self {
		Self { web3_instance, contract }
	}

    async fn filter_swap_events(&self, block_hash: web3::types::H256, swap_event_signature: web3::types::H256) -> Result<Vec<web3::types::Log>, Box<dyn std::error::Error>> {
        let contract_address = self.contract.address();

        let res: Vec<web3::types::Log> = self.web3_instance
            .eth()
            .logs(
                web3::types::FilterBuilder::default()
                .block_hash(block_hash)
                .address(vec![contract_address])
                .from_block(web3::types::BlockNumber::Number(18741851.into()))
                .to_block(web3::types::BlockNumber::Number(18742400.into()))
                .topics(Some(vec![swap_event_signature]), None, None, None)
                .build(),
            ).await?;

        Ok(res)
    }

	pub async fn display_logs(&self) -> Result<(), Box<dyn std::error::Error>> {
		let mut reorg_check: BTreeSet<web3::types::H256> = BTreeSet::new();
        let mut check_max_reorg = |block_hash: web3::types::H256| -> Result<(), Box<dyn std::error::Error>> {
            if reorg_check.contains(&block_hash) {
                return Ok(());
            } else {
                reorg_check.insert(block_hash);
                if reorg_check.len() >= 5 {
                    return Err(Box::new(LoggerError::ReorgBlocksExceededLimit));
                }
            }
            Ok(())
        };

		let swap_event = self.contract.abi().events_by_name("Swap")?.first().unwrap();
        let swap_event_signature = swap_event.signature();

		let latest_block_number = self.web3_instance.eth().block_number().await.unwrap();
		println!("Latest block number: {}", latest_block_number);

        let mut block_stream =
            self.web3_instance.clone().eth_subscribe().subscribe_new_heads().await?;

		while let Some(Ok(block)) = block_stream.next().await {
            let swap_logs_in_block = self.filter_swap_events(block.hash.unwrap(), swap_event_signature).await?;

			for log in swap_logs_in_block {
				if let Ok(parsed_log) = swap_event.parse_log(web3::ethabi::RawLog {
					topics: log.clone().topics,
					data: log.clone().data.0,
				}) {
					Self::print_log_formatted(parsed_log)?;
				} else {
					println!("Log error in block: {:?}", &block.hash);
                    check_max_reorg(block.hash.unwrap())?;
				}
			}
		}

		Ok(())
	}

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
						16,
						&AmountType::DAI,
					)?;
					println!("\t{}: {}", param.name, amount0);
				},
				"amount1" => {
					amount1 = DaiUsdc::amount_to_decimal(
						param.value.to_string().as_str(),
						16,
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
