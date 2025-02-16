use error_mapper::{create_new_error, TheResult};
use mysql::{
    prelude::{FromRow, Queryable},
    PooledConn,
};
use rust_decimal::Decimal;

use crate::{datatypes::WalletsIdType, row_to_data};

use super::Wallet;

impl Wallet {
    pub(super) fn select_all(conn: &mut PooledConn) -> TheResult<Vec<Wallet>> {
        conn.query::<Wallet, _>("SELECT * FROM wallets")
            .map_err(|error| create_new_error!(error.to_string()))
    }
    pub(in crate::modules) fn select_by_id(
        conn: &mut PooledConn,
        wallet_id: WalletsIdType,
    ) -> TheResult<Option<Wallet>> {
        let query = "SELECT * FROM wallets WHERE ID = ?;";
        let stmt = conn
            .prep(query)
            .map_err(|error| create_new_error!(error.to_string()))?;
        conn.exec_first(stmt, (wallet_id,))
            .map_err(|error| create_new_error!(error.to_string()))
    }
    pub(in crate::modules) fn affect_balance(
        &mut self,
        conn: &mut PooledConn,
        requested_amount: Decimal,
    ) -> TheResult<bool> {
        if self.id == 0 {
            return Err(create_new_error!(
                "Wallet cannot have an ID of zero when affecting its balance"
            ));
        }

        self.balance += requested_amount;
        let query = "UPDATE `wallets` SET `balance` = ? WHERE ID = ?";
        let stmt = conn
            .prep(query)
            .map_err(|error| create_new_error!(error.to_string()))?;
        let params = vec![self.balance.to_string(), self.id.to_string()];

        conn.exec_drop(stmt, params)
            .map_err(|error| create_new_error!(error.to_string()))?;

        Ok(conn.affected_rows() > 0)
    }
}

impl FromRow for Wallet {
    fn from_row(row: mysql::Row) -> Self
    where
        Self: Sized,
    {
        Self {
            id: row_to_data!(row, "ID", "wallets", WalletsIdType),
            balance: row_to_data!(row, "balance", "wallets", Decimal),
        }
    }

    fn from_row_opt(_row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        unimplemented!()
    }
}
