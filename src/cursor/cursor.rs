// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CString;
use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::{DataStoreConnection, error::Error, Parameters, root::{
    CCursor,
    CCursor_destroy,
    CDataStoreConnection_createCursor,
    CException,
}, Statement, Transaction};

use super::{CursorRow, OpenedCursor};

#[derive(Debug)]
pub struct Cursor<'a> {
    #[allow(dead_code)]
    pub inner: *mut CCursor,
    pub(crate) connection: &'a DataStoreConnection,
    statement: Statement<'a>,
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
        CException::handle(AssertUnwindSafe(|| unsafe {
            CDataStoreConnection_createCursor(
                connection.inner,
                ptr::null(),
                statement.prefixes.inner,
                c_query.as_ptr(),
                c_query_len,
                parameters.inner,
                &mut c_cursor,
            )
        }))?;
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
        self.execute_and_rollback(|row| {
            for term_index in 0..row.opened.arity {
                let resource_id = row.resource_id(term_index)?;
                log::info!(
                "row={rowid} multiplicity={multiplicity} \
                 term_index={term_index} resource_id={resource_id}:",
                rowid = row.rowid,
                multiplicity = row.multiplicity
            );
                // let value = row.resource_value(resource_id)?;
                let value = row.resource_value_lexical_form(resource_id)?;
                log::info!("{value:?}");
                // log::info!("{}{}", value.prefix, value.value);
            }
            Ok(())
        })
    }

    fn consume_cursor<T>(&mut self, tx: &mut Transaction, mut f: T) -> Result<u64, Error>
        where T: FnMut(CursorRow) -> Result<(), Error> {
        let (mut opened_cursor, mut multiplicity) = OpenedCursor::new(self, &tx)?;
        let mut rowid = 0_u64;
        let mut count = 0_u64;
        while multiplicity > 0 {
            rowid += 1;
            count += multiplicity;
            let row = CursorRow { opened: &opened_cursor, multiplicity, count, rowid };
            f(row)?;
            multiplicity = opened_cursor.advance()?;
        }
        Ok(count)
    }

    pub fn update_and_commit<T, U>(&mut self, f: T) -> Result<u64, Error>
        where T: FnMut(CursorRow) -> Result<(), Error> {
        Transaction::begin_read_write(self.connection)?.update_and_commit(|tx| {
            self.consume_cursor(tx, f)
        })
    }

    pub fn execute_and_rollback<T>(&mut self, f: T) -> Result<u64, Error>
        where T: FnMut(CursorRow) -> Result<(), Error> {
        Transaction::begin_read_only(self.connection)?.execute_and_rollback(|tx| {
            self.consume_cursor(tx, f)
        })
    }
}
