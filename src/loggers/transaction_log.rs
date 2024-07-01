use std::f32::RADIX;
use super::AmountType;
use anyhow::Result;

use crate::converters::{dai_usdc::DaiUsdc, Radix};

struct TransactionLog {
    sender: String,
    recepient: String,
    amount0: String,
    amount1: String,
    direction: String,
}

impl TransactionLog {
    // TODO
    // move SwapLogger::print_log_formatted to this type
    pub fn new(log: web3::ethabi::Log) {

    }

    pub fn set_amount_0(&mut self, amount0: String) {
        self.amount0 = amount0;
    }

    pub fn set_amount_1(&mut self, amount1: String) {
        self.amount0 = amount1;
    }

    pub fn set_sender(&mut self, sender: String) {
        self.sender = sender;
    }

    pub fn set_recepient(&mut self, recepient: String) {
        self.recepient = recepient;
    }

    pub fn print_formatted(&self) {

    }


}
