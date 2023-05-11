// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    super::{CursorRow, OpenedCursor},
    crate::{
        database_call,
        root::{CCursor, CCursor_destroy, CDataStoreConnection_createCursor},
        DataStoreConnection,
        Parameters,
        Statement,
        Transaction,
    },
    rdf_store_rs::{consts::LOG_TARGET_DATABASE, RDFStoreError},
    std::{ffi::CString, fmt::Debug, ptr, sync::Arc},
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
                tracing::debug!(target: LOG_TARGET_DATABASE, "Dropped cursor");
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
    ) -> Result<Self, RDFStoreError> {
        assert!(!connection.inner.is_null());
        let mut c_cursor: *mut CCursor = ptr::null_mut();
        // let c_base_iri = if let Some(base_iri) = base_iri {
        //     CString::new(base_iri.as_str()).unwrap()
        // } else {
        //     CString::new(DEFAULT_BASE_IRI).unwrap()
        // };
        let c_query = CString::new(statement.text.as_str()).unwrap();
        let c_query_len = c_query.as_bytes().len() as u64;
        tracing::trace!(
            target: LOG_TARGET_DATABASE,
            sparql = ?c_query,
            "Starting a cursor"
        );
        // pub fn CDataStoreConnection_createCursor(
        //     dataStoreConnection: *mut root::CDataStoreConnection,
        //     queryText: *const ::std::os::raw::c_char,
        //     queryTextLength: usize,
        //     compilationParameters: *const root::CParameters,
        //     cursor: *mut *mut root::CCursor,
        // ) -> *const root::CException;
        database_call!(
            "Starting a cursor",
            CDataStoreConnection_createCursor(
                connection.inner,
                c_query.as_ptr(),
                c_query_len,
                parameters.inner.as_ref().cast_const(),
                &mut c_cursor,
            )
        )?;
        let cursor = Cursor {
            inner:      c_cursor,
            connection: connection.clone(),
            statement:  statement.clone(),
        };
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "Created cursor for {:}",
            &cursor.statement
        );
        Ok(cursor)
    }

    pub fn sparql_string(&self) -> &str { self.statement.text.as_str() }

    pub fn count(&mut self, tx: &Arc<Transaction>) -> Result<u64, RDFStoreError> {
        self.consume(tx, 1000000000, |_row| Ok(()))
    }

    #[tracing::instrument(
    target = "database",
    skip_all,
    fields(
    max.row = max_row,
    )
    )]
    pub fn consume<T, E>(&mut self, tx: &Arc<Transaction>, max_row: u64, mut f: T) -> Result<u64, E>
    where
        T: FnMut(&CursorRow) -> Result<(), E>,
        E: From<RDFStoreError> + Debug,
    {
        let sparql_str = self.statement.text.clone();
        let (mut opened_cursor, mut multiplicity) = OpenedCursor::new(self, tx.clone())?;
        let mut rowid = 0_u64;
        let mut count = 0_u64;
        while multiplicity > 0 {
            if multiplicity >= max_row {
                return Err(
                    RDFStoreError::MultiplicityExceededMaximumNumberOfRows {
                        maxrow: max_row,
                        multiplicity,
                        query: sparql_str,
                    }
                    .into(),
                )
            }
            rowid += 1;
            if rowid >= max_row {
                return Err(RDFStoreError::ExceededMaximumNumberOfRows {
                    maxrow: max_row,
                    query:  sparql_str,
                }
                .into())
            }
            count += multiplicity;
            let row = CursorRow { opened: &opened_cursor, multiplicity, count, rowid };
            if let Err(err) = f(&row) {
                tracing::error!("Error while consuming row: {:?}", err);
                Err(err)?;
            }
            multiplicity = opened_cursor.advance()?;
        }
        Ok(count)
    }

    pub fn update_and_commit<T, U>(&mut self, maxrow: u64, f: T) -> Result<u64, RDFStoreError>
    where T: FnMut(&CursorRow) -> Result<(), RDFStoreError> {
        let tx = Transaction::begin_read_write(&self.connection)?;
        self.update_and_commit_in_transaction(tx, maxrow, f)
    }

    pub fn execute_and_rollback<T>(&mut self, maxrow: u64, f: T) -> Result<u64, RDFStoreError>
    where T: FnMut(&CursorRow) -> Result<(), RDFStoreError> {
        let tx = Transaction::begin_read_only(&self.connection)?;
        self.execute_and_rollback_in_transaction(&tx, maxrow, f)
    }

    pub fn execute_and_rollback_in_transaction<T>(
        &mut self,
        tx: &Arc<Transaction>,
        maxrow: u64,
        f: T,
    ) -> Result<u64, RDFStoreError>
    where
        T: FnMut(&CursorRow) -> Result<(), RDFStoreError>,
    {
        tx.execute_and_rollback(|ref tx| self.consume(tx, maxrow, f))
    }

    pub fn update_and_commit_in_transaction<T>(
        &mut self,
        tx: Arc<Transaction>,
        maxrow: u64,
        f: T,
    ) -> Result<u64, RDFStoreError>
    where
        T: FnMut(&CursorRow) -> Result<(), RDFStoreError>,
    {
        tx.update_and_commit(|ref tx| self.consume(tx, maxrow, f))
    }
}
