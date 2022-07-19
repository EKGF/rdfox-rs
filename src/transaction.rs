// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use crate::error::Error;
use crate::{
    root::{
        CDataStoreConnection_beginTransaction, CDataStoreConnection_rollbackTransaction,
        CException, CTransactionType,
    },
    DataStoreConnection,
};

pub struct Transaction<'a> {
    pub(crate) connection: &'a DataStoreConnection,
}

impl<'a> Transaction<'a> {
    pub fn begin(
        connection: &'a DataStoreConnection,
        tx_type: CTransactionType,
    ) -> Result<Self, Error> {
        CException::handle(|| unsafe {
            CDataStoreConnection_beginTransaction(connection.inner, tx_type)
        })?;
        log::debug!("Started transaction");
        Ok(Self { connection })
    }

    pub fn begin_read_only(connection: &'a DataStoreConnection) -> Result<Self, Error> {
        Self::begin(connection, CTransactionType::TRANSACTION_TYPE_READ_ONLY)
    }

    pub fn rollback(&self) -> Result<(), Error> {
        CException::handle(|| unsafe {
            CDataStoreConnection_rollbackTransaction(self.connection.inner)
        })
    }

    pub fn execute_and_rollback<T, F>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce() -> Result<T, Error>,
    {
        let result = f();
        self.rollback()?;
        result
    }
}
