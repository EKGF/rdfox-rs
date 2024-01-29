// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{
        Cursor,
        database_call,
        rdfox_api::{
            CCursor,
            CCursor_advance,
            CCursor_getAnswerVariableName,
            CCursor_getArity,
            CCursor_open,
        },
        Transaction,
    },
    ekg_namespace::consts::LOG_TARGET_DATABASE,
    std::{ptr, sync::Arc},
};

#[derive(Debug)]
pub struct OpenedCursor<'a> {
    pub tx: Arc<Transaction>,
    pub cursor: &'a Cursor,
    /// the arity (i.e., the number of columns) of the answers that the
    /// cursor computes.
    pub arity: usize,
}

impl<'a> OpenedCursor<'a> {
    /// Open the cursor, get the details like arity and argument info and
    /// return it as a tuple with all the details (except multiplicity)
    /// as an `OpenedCursor` and the multiplicity of the first row.
    pub(crate) fn new(
        cursor: &'a mut Cursor,
        tx: Arc<Transaction>,
    ) -> Result<(Self, usize), ekg_error::Error> {
        let c_cursor = cursor.inner;
        let multiplicity = Self::open(cursor.inner)?;
        let arity = Self::arity(c_cursor)?;
        let opened_cursor = OpenedCursor { tx, cursor, arity };
        Ok((opened_cursor, multiplicity))
    }

    fn open(c_cursor: *mut CCursor) -> Result<usize, ekg_error::Error> {
        let skip_to_offset = 0_usize;
        let mut multiplicity = 0_usize;
        database_call!(
            "opening a cursor",
            CCursor_open(c_cursor, skip_to_offset, &mut multiplicity)
        )?;
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "CCursor_open ok multiplicity={multiplicity}"
        );
        Ok(multiplicity)
    }

    /// Returns the arity (i.e., the number of columns) of the answers that the
    /// cursor computes.
    fn arity(c_cursor: *mut CCursor) -> Result<usize, ekg_error::Error> {
        let mut arity = 0_usize;
        database_call!(
            "getting the arity",
            CCursor_getArity(c_cursor, &mut arity)
        )?;
        Ok(arity)
    }

    /// TODO: Check why this panics when called after previous call returned
    /// zero
    pub fn advance(&mut self) -> Result<usize, ekg_error::Error> {
        let mut multiplicity = 0_usize;
        database_call!(
            "advancing the cursor",
            CCursor_advance(self.cursor.inner, &mut multiplicity)
        )?;
        tracing::trace!(
            target: LOG_TARGET_DATABASE,
            "cursor {:?} advanced, multiplicity={multiplicity}",
            self.cursor.inner
        );
        Ok(multiplicity)
    }

    pub fn update_and_commit<T, U>(&mut self, f: T) -> Result<U, ekg_error::Error>
        where T: FnOnce(&mut OpenedCursor) -> Result<U, ekg_error::Error> {
        Transaction::begin_read_write(&self.cursor.connection)?.update_and_commit(|_tx| f(self))
    }

    pub fn execute_and_rollback<T, U>(&mut self, f: T) -> Result<U, ekg_error::Error>
        where T: FnOnce(&mut OpenedCursor) -> Result<U, ekg_error::Error> {
        Transaction::begin_read_only(&self.cursor.connection)?.execute_and_rollback(|_tx| f(self))
    }

    /// Get the variable name used in the executed SPARQL statement representing
    /// the given column in the output.
    pub fn get_answer_variable_name(&self, index: usize) -> Result<String, ekg_error::Error> {
        let mut c_buf: *const std::os::raw::c_char = ptr::null();
        database_call!(
            "getting a variable name",
            CCursor_getAnswerVariableName(self.cursor.inner, index, &mut c_buf)
        )?;
        let c_name = unsafe { std::ffi::CStr::from_ptr(c_buf) };
        Ok(c_name.to_str().unwrap().to_owned())
    }
}
