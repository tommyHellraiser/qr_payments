use error_mapper::{create_new_error, TheResult};
use mysql::{
    prelude::{FromRow, Queryable},
    PooledConn,
};
use rust_decimal::Decimal;

use crate::{
    datatypes::{TransactionsIdType, WalletsIdType},
    row_to_data,
};

use super::{Transaction, TransactionStatus};

impl Transaction {
    pub(super) fn select_by_token_and_wallets_id(
        conn: &mut PooledConn,
        wallets_id: WalletsIdType,
        token: String,
    ) -> TheResult<Option<Self>> {
        let query = "SELECT * FROM `transactions` WHERE `wallets_ID` = ? AND `token` = ?;";
        let params = vec![wallets_id.to_string(), token];

        let stmt = conn
            .prep(query)
            .map_err(|error| create_new_error!(error.to_string()))?;

        let result = conn
            .exec_first_opt::<Transaction, _, _>(stmt, params)
            .map_err(|error| create_new_error!(error.to_string()))?
            .transpose()
            .map_err(|error| create_new_error!(error.to_string()))?;

        Ok(result)
    }

    pub(super) fn insert(&mut self, conn: &mut PooledConn) -> TheResult<String> {
        let Some(token) = self.token.clone() else {
            return Err(create_new_error!(
                "Cannot proceess transaction without token"
            ));
        };

        let query = "INSERT INTO `transactions`(`wallets_ID`, `amount`, `status`, `token`) VALUES(?, ?, ?, ?);";
        let stmt = conn
            .prep(query)
            .map_err(|error| create_new_error!(error.to_string()))?;
        let params = vec![
            self.wallets_id.map(|id| id.to_string()),
            Some(self.amount.to_string()),
            Some(self.status.to_string()),
            Some(token.clone()),
        ];

        conn.exec_drop(stmt, params)
            .map_err(|error| create_new_error!(error.to_string()))?;
        let last_id = conn.last_insert_id();
        self.id = last_id;

        Ok(token)
    }

    pub(super) fn log(&self, conn: &mut PooledConn) -> TheResult<bool> {
        let query = "INSERT INTO `transactions`(`amount`, `status`, `errors`) VALUES(?, ?, ?);";
        let stmt = conn
            .prep(query)
            .map_err(|error| create_new_error!(error.to_string()))?;
        let params = vec![
            Some(self.amount.to_string()),
            Some(TransactionStatus::Log.to_string()),
            self.errors.clone(),
        ];

        conn.exec_drop(stmt, params)
            .map_err(|error| create_new_error!(error.to_string()))?;

        Ok(conn.affected_rows() > 0)
    }

    pub(super) fn update_status_and_error(&self, conn: &mut PooledConn) -> TheResult<bool> {
        if self.id == 0 {
            return Err(create_new_error!(
                "Transaction ID cannot be zero for an update operation"
            ));
        }

        let mut query = String::from("UPDATE `transactions` SET `status` = ?");
        let mut params = vec![self.status.to_string()];

        //  Optional params
        if let Some(error) = &self.errors {
            query.push_str(", `error` = ?");
            params.push(error.clone());
        }

        //  Complete query
        query.push_str(" WHERE `ID` = ?;");
        params.push(self.id.to_string());

        let stmt = conn
            .prep(query)
            .map_err(|error| create_new_error!(error.to_string()))?;

        conn.exec_drop(stmt, params)
            .map_err(|error| create_new_error!(error.to_string()))?;

        Ok(conn.affected_rows() > 0)
    }
}

impl FromRow for Transaction {
    fn from_row(_row: mysql::Row) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }
    fn from_row_opt(row: mysql::Row) -> Result<Self, mysql::FromRowError>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row_to_data!(row, "ID", "transactions", TransactionsIdType),
            wallets_id: row_to_data!(row, "wallets_ID", "transactions", Option<WalletsIdType>),
            amount: row_to_data!(row, "amount", "transactions", Decimal),
            token: row_to_data!(row, "token", "transactions", Option<String>),
            errors: row_to_data!(row, "errors", "transactions", Option<String>),
            status: TransactionStatus::from_string(row_to_data!(row, "status", "transactions", String))
                .ok_or(mysql::FromRowError(row))?,
        })
    }
}
