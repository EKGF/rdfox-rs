// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use crate::{
    database_call,
    error::Error,
    root::{
        CDataStoreConnection_beginTransaction,
        CDataStoreConnection_commitTransaction,
        CDataStoreConnection_rollbackTransaction,
        CTransactionType,
    },
    DataStoreConnection,
};

#[derive(Debug)]
pub struct Transaction<'a> {
    pub connection: &'a DataStoreConnection<'a>,
    committed:      bool,
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if !self.committed {
            panic!("Transaction was not committed nor rolled back");
        } else {
            log::debug!("Ended transaction");
        }
    }
}

impl<'a> Transaction<'a> {
    fn begin(
        connection: &'a DataStoreConnection,
        tx_type: CTransactionType,
    ) -> Result<Self, Error> {
        database_call!(
            "starting a transaction",
            CDataStoreConnection_beginTransaction(connection.inner, tx_type)
        )?;
        log::debug!("Started transaction");
        Ok(Self {
            connection,
            committed: false,
        })
    }

    pub fn begin_read_only(connection: &'a DataStoreConnection) -> Result<Self, Error> {
        Self::begin(connection, CTransactionType::TRANSACTION_TYPE_READ_ONLY)
    }

    pub fn begin_read_write(connection: &'a DataStoreConnection) -> Result<Self, Error> {
        Self::begin(connection, CTransactionType::TRANSACTION_TYPE_READ_WRITE)
    }

    pub fn begin_read_write_do<T, F>(
        connection: &'a DataStoreConnection,
        f: F,
    ) -> Result<T, Error>
    where
        F: FnOnce(&mut Self) -> Result<T, Error>,
    {
        let mut tx = Self::begin_read_write(connection)?;
        let result = f(&mut tx);
        tx.commit()?;
        result
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        if !self.committed {
            self.committed = true; // May have to be made more thread-safe?
            database_call!(
                "committing a transaction",
                CDataStoreConnection_commitTransaction(self.connection.inner)
            )?;
        }
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), Error> {
        if !self.committed {
            self.committed = true; // May have to be made more thread-safe?
            database_call!(
                "rolling back a transaction",
                CDataStoreConnection_rollbackTransaction(self.connection.inner)
            )?;
        }
        Ok(())
    }

    pub fn update_and_commit<T, F>(&mut self, f: F) -> Result<T, Error>
    where F: FnOnce(&mut Transaction) -> Result<T, Error> {
        let result = f(self);
        if result.is_ok() {
            self.commit()?;
        } else {
            self.rollback()?;
        }
        result
    }

    pub fn execute_and_rollback<T, F>(&mut self, f: F) -> Result<T, Error>
    where F: FnOnce(&mut Transaction) -> Result<T, Error> {
        let result = f(self);
        match &result {
            Err(err) => {
                log::error!("Error occurred during transaction: {err}");
            },
            Ok(..) => {
                log::debug!("Readonly-transaction was successful");
            },
        }
        self.rollback()?;
        result
    }
}
