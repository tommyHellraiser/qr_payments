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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, strum::Display, PartialEq, PartialOrd)]
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

    fn validate_previous_status(&self, previous_status: TransactionStatus) -> Option<bool> {
        let previous_valid_status = self.status.previous_state();
        if let Some(previous_valid) = previous_valid_status {
            if previous_status != previous_valid {
                return Some(false)
            }
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

    fn previous_state(&self) -> Option<Self> {
        match self {
            Self::Initialized => None,
            Self::Confirmed => Some(Self::Initialized),
            Self::Declined => Some(Self::Initialized),
            Self::Cancelled => Some(Self::Initialized),
            Self::InternalError => Some(Self::Initialized),
            Self::Log => None,
        }
    }
}
