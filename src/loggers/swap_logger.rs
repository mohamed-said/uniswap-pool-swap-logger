use crate::{
    converters::{dai_usdc::DaiUsdc, Radix},
    loggers::{AmountError, AmountType, LoggerError},
};

use futures::StreamExt;
use web3::{contract::Contract, transports::WebSocket, Web3};

use anyhow::{Context, Result};

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

    // TODO make the accepted types dynamic
    // to support multiple swapping pools
    // not only DAI/USDC
    // _amount_type_0: AmountType,
    // _amount_type_1: AmountType,
}

impl SwapLogger {
    pub fn new(contract: Contract<WebSocket>, web3_instance: Web3<WebSocket>) -> Self {
        Self {
            web3_instance,
            contract,
        }
    }

    /// filter swap events for a given block
    async fn filter_swap_events(
        &self,
        block_hash: web3::types::H256,
        swap_event_signature: web3::types::H256,
    ) -> Result<Vec<web3::types::Log>> {
        let contract_address = self.contract.address();

        let res: Vec<web3::types::Log> = self
            .web3_instance
            // Eth object to get acces to the EPC functions
            .eth()
            // get all logs matching a given filter object
            .logs(
                // build the filter
                web3::types::FilterBuilder::default()
                    // sets the internal "Filter" type object's
                    // block_hash value
                    .block_hash(block_hash)
                    // takes a single address
                    // for more information about why it takes a vector
                    // refer to https://docs.rs/ethers/latest/ethers/core/types/enum.ValueOrArray.html
                    .address(vec![contract_address])
                    // An event topic is a part of the event logging mechanism
                    // used by smart contracts to record information on the blockchain.
                    // These logs are stored in transaction receipts
                    // and can be efficiently filtered and searched.
                    // Each log entry has -up to- four "topics,"
                    // which are 32-byte values used to index the log.
                    //    Topic 0: The hash of the event signature.
                    //    Topic 1: The from address (indexed).
                    //    Topic 2: The to address (indexed).
                    //    Data: The value (not indexed).
                    .topics(Some(vec![swap_event_signature]), None, None, None)
                    // returns a Filter object with all the specified fields
                    .build(),
            )
            // call the inner future which fetches the logs
            .await?;

        Ok(res)
    }

    /// subscribes to the latest updates and handles logs printing whenever a new block is ready
    pub async fn display_logs(&self) -> Result<()> {
        // create a new Event object
        // the default event name is now swap
        let swap_event = self
            .contract
            // return ethabi::Contract object, where we can get
            // access to the down layer ABI related methods
            .abi()
            // accesses the list of events in tha ABI
            // and retrieves all events by the given name
            .events_by_name("Swap")?
            // assuming we have only one event by the given name
            // we capture that event
            // returns ethabi::Event object
            .first()
            // handle error if the given event name is invalid
            .ok_or_else(|| Box::new(LoggerError::FailedToRetrieveEvent))?;

        // retrieve the signature of the event
        // an event signature is a unique identifier
        // for a specific event defined within a smart contract.
        // When an event is emitted (triggered) by a smart contract,
        // it is recorded in the transaction logs,
        // which are part of the blockchain's transaction receipts.
        // These logs can be indexed and queried to listen for specific events.
        // Which is exactly what we want to do
        let swap_event_signature = swap_event.signature();

        // create a heads subscription
        let mut block_stream = self
            .web3_instance
            // accesss the RPC subscribe methods from the eth namespace
            // retuerns web3::api::EthSubscribe object
            // including the available subscription methods
            // An Ethereum namespace subscription
            // refers to a method of listening to and receiving real-time updates
            // for certain events or data changes on the Ethereum blockchain.
            // This is commonly done through WebSocket connections
            // or other subscription mechanisms provided by Ethereum nodes or APIs
            .eth_subscribe()
            // A "new heads" subscription in  Ethereum refers to
            // subscribing to notifications about new blocks (also called "block headers")
            // being added to the blockchain.
            // This allows you to receive real-time updates whenever a new block is mined.
            //
            // Returns a subscription stream of blocks (since we subscribe to get blocks)
            // A subscription stream is a stream of notifications from a subscription.
            // Given a type deserializable from rpc::Value and a subscription id,
            // yields items of that type as notifications are delivered.
            .subscribe_new_heads()
            .await?;

        while let Some(Ok(block)) = block_stream.next().await {
            // a list of all the "Swap" events in the current block
            let swap_logs_in_block = self
                // filter the current block's events for only the swap events
                // or, in other words, the events with the given signature
                .filter_swap_events(block.hash.unwrap(), swap_event_signature)
                .await
                // anyhow error message
                .context("Filter swap events failed")?;

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
                    }
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
    fn print_log_formatted(log: web3::ethabi::Log) -> Result<()> {
        // Amounts to be transferred (given/received)
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
                }
                "amount1" => {
                    amount1 = DaiUsdc::amount_to_decimal(
                        param.value.to_string().as_str(),
                        Radix::Base16.to_uint(),
                        &AmountType::USDC,
                    )?;
                    println!("\t{}: {}", param.name, amount1);
                }
                _ => {
                    println!("\t{}: {}", param.name, param.value);
                }
            }
        }
        println!(
            "\tdirection: {}",
            Self::swap_direction(amount0, amount1).context("Swap direction failed!")?
        );
        println!("}}");

        Ok(())
    }

    /// determine swap direction (DAI -> USDC) or (USDC -> DAI)
    fn swap_direction(amount0: String, amount1: String) -> Result<String> {
        if amount0.starts_with('-') && !amount1.starts_with('-') {
            return Ok(format!("{} -> {}", AmountType::USDC, AmountType::DAI));
        } else if amount1.starts_with('-') && !amount0.starts_with('-') {
            return Ok(format!("{} -> {}", AmountType::DAI, AmountType::USDC));
        }

        Err(AmountError::AllAmountsAreNegative.into())
    }
}
