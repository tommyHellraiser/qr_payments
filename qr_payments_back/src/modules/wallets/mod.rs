use mysql::prelude::FromRow;
use rust_decimal::Decimal;

use crate::{datatypes::WalletsIdType, row_to_data};

#[derive(Debug, Default, Clone, Copy)]
pub struct Wallets {
    id: WalletsIdType,
    balance: Decimal
}

impl FromRow for Wallets {
    fn from_row(row: mysql::Row) -> Self
        where
            Self: Sized, {
        Self {
            id: row_to_data!(row, "ID", "wallets", WalletsIdType),
            balance: row_to_data!(row, "balance", "wallets", Decimal)
        }
    }

    fn from_row_opt(_row: mysql::Row) -> Result<Self, mysql::FromRowError>
        where
            Self: Sized {
        unimplemented!()
    }
}