// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::sync::{atomic::AtomicBool, Arc};

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
    committed:      AtomicBool,
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            log::debug!("Ended transaction");
        } else {
            panic!("Transaction was not committed nor rolled back");
        }
    }
}

impl<'a> Transaction<'a> {
    fn begin(
        connection: &'a DataStoreConnection,
        tx_type: CTransactionType,
    ) -> Result<Arc<Self>, Error> {
        assert!(!connection.inner.is_null());
        database_call!(
            "starting a transaction",
            CDataStoreConnection_beginTransaction(connection.inner, tx_type)
        )?;
        log::debug!("Started transaction");
        Ok(Arc::new(Self {
            connection,
            committed: AtomicBool::new(false),
        }))
    }

    pub fn begin_read_only(connection: &'a DataStoreConnection) -> Result<Arc<Self>, Error> {
        Self::begin(connection, CTransactionType::TRANSACTION_TYPE_READ_ONLY)
    }

    pub fn begin_read_write(connection: &'a DataStoreConnection) -> Result<Arc<Self>, Error> {
        Self::begin(connection, CTransactionType::TRANSACTION_TYPE_READ_WRITE)
    }

    pub fn begin_read_write_do<T, F>(
        connection: &'a DataStoreConnection,
        f: F,
    ) -> Result<T, Error>
    where
        F: FnOnce(Arc<Transaction>) -> Result<T, Error>,
    {
        let tx = Self::begin_read_write(connection)?;
        let result = f(tx.clone());
        tx.commit()?;
        result
    }

    pub fn commit(self: &Arc<Self>) -> Result<(), Error> {
        if !self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            self.committed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            database_call!(
                "committing a transaction",
                CDataStoreConnection_commitTransaction(self.connection.inner)
            )?;
            log::debug!("Committed transaction");
        }
        Ok(())
    }

    pub fn rollback(self: &Arc<Self>) -> Result<(), Error> {
        if !self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            self.committed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            assert!(!self.connection.inner.is_null());
            database_call!(
                "rolling back a transaction",
                CDataStoreConnection_rollbackTransaction(self.connection.inner)
            )?;
            log::debug!("Rolled back transaction");
        }
        Ok(())
    }

    pub fn update_and_commit<T, F>(self: &Arc<Self>, f: F) -> Result<T, Error>
    where F: FnOnce(Arc<Transaction>) -> Result<T, Error> {
        let result = f(self.clone());
        if result.is_ok() {
            self.commit()?;
        } else {
            self.rollback()?;
        }
        result
    }

    pub fn execute_and_rollback<T, F>(self: &Arc<Self>, f: F) -> Result<T, Error>
    where F: FnOnce(Arc<Transaction>) -> Result<T, Error> {
        let result = f(self.clone());
        match &result {
            Err(err) => {
                log::error!("Error occurred during transaction: {err}");
            },
            Ok(..) => {
                log::debug!("Readonly-transaction was successful (but still rolling back)");
            },
        }
        self.rollback()?;
        result
    }
}
