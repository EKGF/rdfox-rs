// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{ffi::CString, ptr};

use super::{CursorRow, OpenedCursor};
use crate::{
    database_call,
    error::Error,
    root::{CCursor, CCursor_destroy, CDataStoreConnection_createCursor},
    DataStoreConnection,
    Parameters,
    Statement,
    Transaction,
};

#[derive(Debug)]
pub struct Cursor<'a> {
    #[allow(dead_code)]
    pub inner:             *mut CCursor,
    pub(crate) connection: &'a DataStoreConnection<'a>,
    statement:             Statement<'a>,
}

impl<'a> Drop for Cursor<'a> {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.is_null() {
                CCursor_destroy(self.inner);
                self.inner = ptr::null_mut();
                log::debug!("Destroyed cursor");
            }
        }
    }
}

impl<'a> Cursor<'a> {
    pub fn create(
        connection: &'a DataStoreConnection,
        parameters: &Parameters,
        statement: Statement<'a>,
    ) -> Result<Self, Error> {
        assert!(!connection.inner.is_null());
        assert!(!statement.prefixes.inner.is_null());
        assert!(!statement.prefixes.inner.is_null());
        let mut c_cursor: *mut CCursor = ptr::null_mut();
        // let base_iri: *const std::os::raw::c_char = ptr::null();
        let c_query = CString::new(statement.text.as_str()).unwrap();
        let c_query_len: u64 = c_query.as_bytes().len() as u64;
        log::trace!("Starting cursor for {:?}", c_query);
        database_call!(
            "creating a cursor",
            CDataStoreConnection_createCursor(
                connection.inner,
                ptr::null(),
                statement.prefixes.inner,
                c_query.as_ptr(),
                c_query_len,
                parameters.inner,
                &mut c_cursor,
            )
        )?;
        let cursor = Cursor {
            inner: c_cursor,
            connection,
            statement,
        };
        log::debug!("Created cursor for {:}", &cursor.statement);
        log::debug!("Cursor {:?}", cursor);
        Ok(cursor)
    }

    pub fn count(&mut self) -> Result<u64, Error> {
        self.execute_and_rollback(1000000000, |_row| Ok(()))
    }

    pub fn count_in_transaction(&mut self, tx: &Transaction) -> Result<u64, Error> {
        self.consume(tx, 1000000000, |_row| Ok(()))
    }

    pub fn consume<T>(&mut self, tx: &Transaction, max_row: u64, mut f: T) -> Result<u64, Error>
    where T: FnMut(CursorRow) -> Result<(), Error> {
        let (mut opened_cursor, mut multiplicity) = OpenedCursor::new(self, &tx)?;
        let mut rowid = 0_u64;
        let mut count = 0_u64;
        while multiplicity > 0 {
            if multiplicity >= max_row {
                return Err(Error::MultiplicityExceededMaximumNumberOfRows {
                    maxrow: max_row,
                    multiplicity,
                    query: self.statement.text.clone(),
                })
            }
            rowid += 1;
            if rowid >= max_row {
                return Err(Error::ExceededMaximumNumberOfRows {
                    maxrow: max_row,
                    query:  self.statement.text.clone(),
                })
            }
            count += multiplicity;
            let row = CursorRow {
                opened: &opened_cursor,
                multiplicity,
                count,
                rowid,
            };
            f(row)?;
            multiplicity = opened_cursor.advance()?;
        }
        Ok(count)
    }

    pub fn update_and_commit<T, U>(&mut self, maxrow: u64, f: T) -> Result<u64, Error>
    where T: FnMut(CursorRow) -> Result<(), Error> {
        let mut tx = Transaction::begin_read_write(self.connection)?;
        self.update_and_commit_in_transaction(&mut tx, maxrow, f)
    }

    pub fn execute_and_rollback<T>(&mut self, maxrow: u64, f: T) -> Result<u64, Error>
    where T: FnMut(CursorRow) -> Result<(), Error> {
        let mut tx = Transaction::begin_read_only(self.connection)?;
        self.execute_and_rollback_in_transaction(&mut tx, maxrow, f)
    }

    pub fn execute_and_rollback_in_transaction<T>(
        &mut self,
        tx: &mut Transaction,
        maxrow: u64,
        f: T,
    ) -> Result<u64, Error>
    where
        T: FnMut(CursorRow) -> Result<(), Error>,
    {
        tx.execute_and_rollback(|tx| self.consume(tx, maxrow, f))
    }

    pub fn update_and_commit_in_transaction<T>(
        &mut self,
        tx: &mut Transaction,
        maxrow: u64,
        f: T,
    ) -> Result<u64, Error>
    where
        T: FnMut(CursorRow) -> Result<(), Error>,
    {
        tx.update_and_commit(|tx| self.consume(tx, maxrow, f))
    }
}
