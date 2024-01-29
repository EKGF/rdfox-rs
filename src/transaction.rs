// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{
        database_call,
        DataStoreConnection,
        rdfox_api::{
            CDataStoreConnection_beginTransaction,
            CDataStoreConnection_commitTransaction,
            CDataStoreConnection_rollbackTransaction,
            CTransactionType,
        },
    }
    ,
    std::{
        fmt::{Display, Formatter},
        sync::{Arc, atomic::AtomicBool},
    },
};

#[derive(Debug)]
pub struct Transaction {
    pub connection: Arc<DataStoreConnection>,
    committed: AtomicBool,
    tx_type: CTransactionType,
    number: usize,
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::debug!(
                target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                txno = self.number,
                conn = self.connection.number,
                "Ended {self:}"
            );
        } else if let Err(err) = self._rollback() {
            panic!("{self:} could not be rolled back: {err}", );
        }
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.get_title()) }
}

impl Transaction {
    fn begin(
        connection: &Arc<DataStoreConnection>,
        tx_type: CTransactionType,
    ) -> Result<Arc<Self>, ekg_error::Error> {
        assert!(!connection.inner.is_null());
        let number = Self::get_number();
        tracing::trace!(
            target: ekg_namespace::consts::LOG_TARGET_DATABASE,
            txno = number,
            conn = connection.number,
            "Starting {}",
            Self::get_title_for(tx_type, number, connection.number)
        );
        database_call!(CDataStoreConnection_beginTransaction(
            connection.inner,
            tx_type
        ))?;
        let tx = Arc::new(Self {
            connection: connection.clone(),
            committed: AtomicBool::new(false),
            number,
            tx_type,
        });
        tracing::debug!(
            target: ekg_namespace::consts::LOG_TARGET_DATABASE,
            txno = tx.number,
            conn = tx.connection.number,
            "Started {tx:}",
        );
        Ok(tx)
    }

    fn get_title(&self) -> String {
        Self::get_title_for(self.tx_type, self.number, self.connection.number)
    }

    fn get_title_for(tx_type: CTransactionType, number: usize, connection_number: usize) -> String {
        match tx_type {
            #[cfg(not(feature = "rdfox-7-0"))]
            CTransactionType::TRANSACTION_TYPE_EXCLUSIVE => {
                format!("Exclusive Transaction #{number} on connection #{connection_number}", )
            }
            CTransactionType::TRANSACTION_TYPE_READ_ONLY => {
                format!("R/O Transaction #{number} on connection #{connection_number}", )
            }
            CTransactionType::TRANSACTION_TYPE_READ_WRITE => {
                format!("R/W Transaction #{number} on connection #{connection_number}", )
            }
        }
    }

    fn get_number() -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    pub fn begin_read_only(
        connection: &Arc<DataStoreConnection>,
    ) -> Result<Arc<Self>, ekg_error::Error> {
        Self::begin(
            connection,
            CTransactionType::TRANSACTION_TYPE_READ_ONLY,
        )
    }

    pub fn begin_read_write(
        connection: &Arc<DataStoreConnection>,
    ) -> Result<Arc<Self>, ekg_error::Error> {
        Self::begin(
            connection,
            CTransactionType::TRANSACTION_TYPE_READ_WRITE,
        )
    }

    pub fn begin_read_write_do<T, F>(
        connection: &Arc<DataStoreConnection>,
        f: F,
    ) -> Result<T, ekg_error::Error>
        where
            F: FnOnce(Arc<Transaction>) -> Result<T, ekg_error::Error>,
    {
        let tx = Self::begin_read_write(connection)?;
        let result = f(tx.clone());
        tx.commit()?;
        result
    }

    pub fn commit(self: &Arc<Self>) -> Result<(), ekg_error::Error> {
        if !self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            self.committed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            tracing::trace!(
                target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                "Committing {self:}"
            );
            database_call!(CDataStoreConnection_commitTransaction(
                self.connection.inner
            ))?;
            tracing::trace!(
                target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                "Committed {self:}",
            );
        }
        Ok(())
    }

    pub fn rollback(self: &Arc<Self>) -> Result<(), ekg_error::Error> {
        if !self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            self.committed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            assert!(!self.connection.inner.is_null());
            tracing::trace!(
                target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                txno = self.number,
                conn = self.connection.number,
                "Rolling back {self:}"
            );
            database_call!(CDataStoreConnection_rollbackTransaction(
                self.connection.inner
            ))?;
            tracing::debug!(
                target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                txno = self.number,
                conn = self.connection.number,
                "Rolled back {self:}",
            );
        }
        Ok(())
    }

    /// A duplicate of `rollback()` that takes a `&mut Transaction` rather than
    /// an `Arc<Transaction>`, only to be used by `drop()`
    fn _rollback(&mut self) -> Result<(), ekg_error::Error> {
        if !self.committed.load(std::sync::atomic::Ordering::Relaxed) {
            self.committed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            assert!(!self.connection.inner.is_null());
            tracing::trace!(
                target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                txno = self.number,
                conn = self.connection.number,
                "Rolling back {self:}"
            );
            database_call!(CDataStoreConnection_rollbackTransaction(
                self.connection.inner
            ))?;
            tracing::debug!(
                target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                txno = self.number,
                conn = self.connection.number,
                "Rolled back {self:}",
            );
        }
        Ok(())
    }

    pub fn update_and_commit<T, E: From<ekg_error::Error>, F>(self: &Arc<Self>, f: F) -> Result<T, E>
        where F: FnOnce(Arc<Transaction>) -> Result<T, E> {
        let result = f(self.clone());
        if result.is_ok() {
            self.commit()?;
        } else {
            self.rollback()?;
        }
        result
    }

    pub fn execute_and_rollback<T, F>(self: &Arc<Self>, f: F) -> Result<T, ekg_error::Error>
        where F: FnOnce(Arc<Transaction>) -> Result<T, ekg_error::Error> {
        let result = f(self.clone());
        match &result {
            Err(err) => {
                tracing::error!(
                    target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                    txno = self.number,
                    conn = self.connection.number,
                    "Error occurred during {self:}: {err}",
                );
            }
            Ok(..) => {
                tracing::debug!(
                    target: ekg_namespace::consts::LOG_TARGET_DATABASE,
                    txno = self.number,
                    conn = self.connection.number,
                    "{self:} was successful (but rolling it back anyway)",
                );
            }
        }
        self.rollback()?;
        result
    }
}
