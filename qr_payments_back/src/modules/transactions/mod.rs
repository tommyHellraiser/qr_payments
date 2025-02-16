use crate::datatypes::{TransactionsIdType, WalletsIdType};
use rand::Rng;
use rand_distr::Alphanumeric;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

mod db;
pub mod services;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Transaction {
    id: TransactionsIdType,
    wallets_id: Option<WalletsIdType>,
    amount: Decimal,
    status: TransactionStatus,
    token: Option<String>,
    errors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, strum::Display)]
enum TransactionStatus {
    #[default]
    Initialized,
    Confirmed,
    Declined,
    Cancelled,
    InternalError,
    Log,
}

impl Transaction {
    fn new(amount: Decimal) -> Self {
        Self {
            id: TransactionsIdType::default(),
            wallets_id: None,
            amount,
            token: None,
            status: TransactionStatus::default(),
            errors: None,
        }
    }

    fn validate(&self) -> Option<String> {
        if self.amount == Decimal::ZERO {
            return Some(String::from("Amount cannot be zero for transaction"));
        }

        None
    }

    fn generate_token(&mut self) {
        self.token = Some(
            rand::rng()
                .sample_iter(&Alphanumeric)
                .take(32)
                .map(char::from)
                .collect(),
        )
    }
}

impl TransactionStatus {
    fn from_string(input: String) -> Option<Self> {
        match input.as_str() {
            "Initialized" => Some(Self::Initialized),
            "Confirmed" => Some(Self::Confirmed),
            "Declined" => Some(Self::Declined),
            "Cancelled" => Some(Self::Cancelled),
            "InternalError" => Some(Self::InternalError),
            "Log" => Some(Self::Log),
            _ => None,
        }
    }
}
