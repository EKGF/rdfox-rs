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
    LOG_TARGET_DATABASE,
};

#[derive(Debug)]
pub struct Transaction {
    pub connection: Arc<DataStoreConnection>,
    committed:      AtomicBool,
    tx_type:        CTransactionType,
    pub number:     usize,
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::debug!(
                target: LOG_TARGET_DATABASE,
                "Ended transaction #{} on connection #{}",
                self.number,
                self.connection.number
            );
        } else {
            if let Err(err) = self._rollback() {
                panic!(
                    "Transaction #{} could not be rolled back: {err}",
                    self.number
                );
            }
        }
    }
}

impl Transaction {
    fn begin(
        connection: &Arc<DataStoreConnection>,
        tx_type: CTransactionType,
    ) -> Result<Arc<Self>, Error> {
        assert!(!connection.inner.is_null());
        let number = Self::get_number();
        database_call!(
            format!(
                "starting transaction #{number} on connection #{}",
                connection.number
            )
            .as_str(),
            CDataStoreConnection_beginTransaction(connection.inner, tx_type)
        )?;
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "Started transaction #{number} on connection #{}",
            connection.number
        );
        let tx = Arc::new(Self {
            connection: connection.clone(),
            committed: AtomicBool::new(false),
            number,
            tx_type,
        });
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "Started {}",
            tx.get_title().as_str()
        );
        Ok(tx)
    }

    fn get_title(&self) -> String {
        match self.tx_type {
            CTransactionType::TRANSACTION_TYPE_READ_ONLY => {
                format!(
                    "Read-Only Transaction #{} on connection #{}",
                    self.number, self.connection.number
                )
            },
            CTransactionType::TRANSACTION_TYPE_READ_WRITE => {
                format!(
                    "Read-Write Transaction #{} on connection #{}",
                    self.number, self.connection.number
                )
            },
        }
    }

    fn get_number() -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn begin_read_only(connection: &Arc<DataStoreConnection>) -> Result<Arc<Self>, Error> {
        Self::begin(&connection, CTransactionType::TRANSACTION_TYPE_READ_ONLY)
    }

    pub fn begin_read_write(connection: &Arc<DataStoreConnection>) -> Result<Arc<Self>, Error> {
        Self::begin(connection, CTransactionType::TRANSACTION_TYPE_READ_WRITE)
    }

    pub fn begin_read_write_do<T, F>(
        connection: &Arc<DataStoreConnection>,
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
                format!("committing {}", self.get_title().as_str()).as_str(),
                CDataStoreConnection_commitTransaction(self.connection.inner)
            )?;
            tracing::debug!(
                target: LOG_TARGET_DATABASE,
                "Committed {}",
                self.get_title().as_str()
            );
        }
        Ok(())
    }

    pub fn rollback(self: &Arc<Self>) -> Result<(), Error> {
        if !self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            self.committed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            assert!(!self.connection.inner.is_null());
            database_call!(
                format!("rolling back {}", self.get_title().as_str()).as_str(),
                CDataStoreConnection_rollbackTransaction(self.connection.inner)
            )?;
            tracing::debug!(
                target: LOG_TARGET_DATABASE,
                "Rolled back {}",
                self.get_title().as_str()
            );
        }
        Ok(())
    }

    /// A duplicate of `rollback()` that takes a `&mut Transaction` rather than
    /// an `Arc<Transaction>`, only to be used by `drop()`
    fn _rollback(&mut self) -> Result<(), Error> {
        if !self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            self.committed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            assert!(!self.connection.inner.is_null());
            database_call!(
                format!("rolling back {}", self.get_title().as_str()).as_str(),
                CDataStoreConnection_rollbackTransaction(self.connection.inner)
            )?;
            tracing::debug!(
                target: LOG_TARGET_DATABASE,
                "Rolled back {}",
                self.get_title().as_str()
            );
        }
        Ok(())
    }

    pub fn update_and_commit<T, E: From<Error>, F>(self: &Arc<Self>, f: F) -> Result<T, E>
    where F: FnOnce(Arc<Transaction>) -> Result<T, E> {
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
                tracing::error!(
                    target: LOG_TARGET_DATABASE,
                    "Error occurred during {}: {err}",
                    self.get_title().as_str()
                );
            },
            Ok(..) => {
                tracing::debug!(
                    target: LOG_TARGET_DATABASE,
                    "{} was successful (but rolling it back anyway)",
                    self.get_title().as_str()
                );
            },
        }
        self.rollback()?;
        result
    }
}
