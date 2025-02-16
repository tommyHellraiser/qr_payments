use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::datatypes::WalletsIdType;

mod db;
pub mod services;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Wallet {
    pub id: WalletsIdType,
    pub balance: Decimal,
}

impl Wallet {
    pub(in crate::modules) fn validate_balance(&self, requested_amount: Decimal) -> bool {
        //  If requested amount is positive it's a credit, authorize without further validations
        match requested_amount.cmp(&Decimal::ZERO) {
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => {}
        }

        match self.balance.abs().cmp(&requested_amount.abs()) {
            std::cmp::Ordering::Greater | std::cmp::Ordering::Equal => true,
            std::cmp::Ordering::Less => false,
        }
    }
}
