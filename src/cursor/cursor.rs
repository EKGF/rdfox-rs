// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{ffi::CString, fmt::Debug, ptr, sync::Arc};

use iref::Iri;

use super::{CursorRow, OpenedCursor};
use crate::{
    database_call,
    error::Error,
    namespace::DEFAULT_BASE_IRI,
    root::{CCursor, CCursor_destroy, CDataStoreConnection_createCursor},
    DataStoreConnection,
    Parameters,
    Statement,
    Transaction,
};

#[derive(Debug)]
pub struct Cursor {
    pub inner:             *mut CCursor,
    pub(crate) connection: Arc<DataStoreConnection>,
    statement:             Statement,
}

impl Drop for Cursor {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.is_null() {
                CCursor_destroy(self.inner);
                self.inner = ptr::null_mut();
                tracing::debug!("Destroyed cursor");
            }
        }
    }
}

impl Cursor {
    // noinspection DuplicatedCode
    pub fn create(
        connection: &Arc<DataStoreConnection>,
        parameters: &Parameters,
        statement: &Statement,
        base_iri: Option<Iri>,
    ) -> Result<Self, Error> {
        assert!(!connection.inner.is_null());
        let mut c_cursor: *mut CCursor = ptr::null_mut();
        let c_base_iri = if let Some(base_iri) = base_iri {
            CString::new(base_iri.as_str()).unwrap()
        } else {
            CString::new(DEFAULT_BASE_IRI).unwrap()
        };
        let c_query = CString::new(statement.text.as_str()).unwrap();
        let c_query_len = c_query.as_bytes().len();
        tracing::trace!("Starting cursor for {:?}", c_query);
        database_call!(
            "creating a cursor",
            CDataStoreConnection_createCursor(
                connection.inner,
                c_base_iri.as_ptr(),
                statement.prefixes.c_mut_ptr(),
                c_query.as_ptr(),
                c_query_len,
                parameters.inner,
                &mut c_cursor,
            )
        )?;
        let cursor = Cursor {
            inner:      c_cursor,
            connection: connection.clone(),
            statement:  statement.clone(),
        };
        tracing::debug!("Created cursor for {:}", &cursor.statement);
        tracing::debug!("Cursor {:?}", cursor);
        Ok(cursor)
    }

    pub fn count(&mut self, tx: &Arc<Transaction>) -> Result<u64, Error> {
        self.consume(tx, 1000000000, |_row| Ok(()))
    }

    pub fn consume<T, E>(&mut self, tx: &Arc<Transaction>, max_row: u64, mut f: T) -> Result<u64, E>
    where
        T: FnMut(CursorRow) -> Result<(), E>,
        E: From<Error> + Debug,
    {
        let sparql_str = self.statement.text.clone();
        let (mut opened_cursor, mut multiplicity) = OpenedCursor::new(self, tx.clone())?;
        let mut rowid = 0_u64;
        let mut count = 0_u64;
        while multiplicity > 0 {
            if multiplicity >= max_row {
                return Err(Error::MultiplicityExceededMaximumNumberOfRows {
                    maxrow: max_row,
                    multiplicity,
                    query: sparql_str,
                }
                .into())
            }
            rowid += 1;
            if rowid >= max_row {
                return Err(Error::ExceededMaximumNumberOfRows {
                    maxrow: max_row,
                    query:  sparql_str,
                }
                .into())
            }
            count += multiplicity;
            let row = CursorRow {
                opened: &opened_cursor,
                multiplicity,
                count,
                rowid,
            };
            if let Err(err) = f(row) {
                tracing::error!("Error while consuming row: {:?}", err);
                Err(err)?;
            }
            multiplicity = opened_cursor.advance()?;
        }
        Ok(count)
    }

    pub fn update_and_commit<T, U>(&mut self, maxrow: u64, f: T) -> Result<u64, Error>
    where T: FnMut(CursorRow) -> Result<(), Error> {
        let tx = Transaction::begin_read_write(&self.connection)?;
        self.update_and_commit_in_transaction(tx, maxrow, f)
    }

    pub fn execute_and_rollback<T>(&mut self, maxrow: u64, f: T) -> Result<u64, Error>
    where T: FnMut(CursorRow) -> Result<(), Error> {
        let tx = Transaction::begin_read_only(&self.connection)?;
        self.execute_and_rollback_in_transaction(&tx, maxrow, f)
    }

    pub fn execute_and_rollback_in_transaction<T>(
        &mut self,
        tx: &Arc<Transaction>,
        maxrow: u64,
        f: T,
    ) -> Result<u64, Error>
    where
        T: FnMut(CursorRow) -> Result<(), Error>,
    {
        tx.execute_and_rollback(|ref tx| self.consume(tx, maxrow, f))
    }

    pub fn update_and_commit_in_transaction<T>(
        &mut self,
        tx: Arc<Transaction>,
        maxrow: u64,
        f: T,
    ) -> Result<u64, Error>
    where
        T: FnMut(CursorRow) -> Result<(), Error>,
    {
        tx.update_and_commit(|ref tx| self.consume(tx, maxrow, f))
    }
}
