// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{ptr, sync::Arc};

use crate::{
    database_call,
    root::{
        CArgumentIndex,
        CCursor,
        CCursor_advance,
        CCursor_getArgumentIndexes,
        CCursor_getArgumentsBuffer,
        CCursor_getArity,
        CCursor_open,
        CResourceID,
    },
    Cursor,
    Error,
    Error::Unknown,
    Transaction,
};

#[derive(Debug)]
pub struct OpenedCursor<'a> {
    pub tx:               Arc<Transaction<'a>>,
    pub cursor:           &'a Cursor<'a>,
    /// the arity (i.e., the number of columns) of the answers that the
    /// cursor computes.
    pub arity:            u16,
    pub arguments_buffer: &'a [u64],
    pub argument_indexes: &'a [u32],
}

impl<'a> OpenedCursor<'a> {
    /// Open the cursor, get the details like arity and argument info and
    /// return it as a tuple with all the details (except multiplicity)
    /// as an `OpenedCursor` and the multiplicity of the first row.
    pub(crate) fn new(
        cursor: &'a mut Cursor,
        tx: Arc<Transaction<'a>>,
    ) -> Result<(Self, u64), Error> {
        let c_cursor = cursor.inner;
        let multiplicity = Self::open(cursor.inner)?;
        let arity = Self::arity(c_cursor)?;
        let arguments_buffer = Self::arguments_buffer(c_cursor)?;
        let argument_indexes = Self::argument_indexes(c_cursor, arity)?;
        let opened_cursor = OpenedCursor {
            tx,
            cursor,
            arity,
            arguments_buffer,
            argument_indexes,
        };
        Ok((opened_cursor, multiplicity))
    }

    fn open(c_cursor: *mut CCursor) -> Result<u64, Error> {
        let mut multiplicity = 0 as usize;
        database_call!(
            "opening a cursor",
            CCursor_open(c_cursor, &mut multiplicity)
        )?;
        log::debug!("CCursor_open ok multiplicity={multiplicity}");
        Ok(multiplicity as u64)
    }

    /// Returns the arity (i.e., the number of columns) of the answers that the
    /// cursor computes.
    fn arity(c_cursor: *mut CCursor) -> Result<u16, Error> {
        let mut arity = 0_usize;
        database_call!("getting the arity", CCursor_getArity(c_cursor, &mut arity))?;
        // Trimming it down, we don't expect more than 2^16 columns do we?
        Ok(arity as u16)
    }

    pub fn arguments_buffer(c_cursor: *mut CCursor) -> Result<&'a [u64], Error> {
        let mut buffer: *const CResourceID = ptr::null_mut();
        database_call!(
            "getting the arguments buffer",
            CCursor_getArgumentsBuffer(c_cursor, &mut buffer)
        )?;
        let mut count = 0_usize;
        unsafe {
            let mut p = buffer;
            while !p.is_null() {
                count += 1;
                let resource_id: CResourceID = *p as CResourceID;
                if resource_id == 0 {
                    break
                }
                log::trace!("{count} resource_id={:?}", resource_id);
                p = p.offset(1);
            }
        }
        unsafe { Ok(std::slice::from_raw_parts(buffer, count - 1)) }
    }

    fn argument_indexes(c_cursor: *mut CCursor, arity: u16) -> Result<&'a [u32], Error> {
        let mut indexes: *const CArgumentIndex = ptr::null_mut();
        database_call!(
            "getting the argument-indexes",
            CCursor_getArgumentIndexes(c_cursor, &mut indexes)
        )?;
        if indexes.is_null() {
            return Err(Unknown)
        }
        unsafe { Ok(std::slice::from_raw_parts(indexes, arity as usize)) }
    }

    /// Get the resource ID from the arguments buffer which dynamically changes
    /// after each cursor advance.
    pub(crate) fn resource_id(&self, term_index: u16) -> Result<Option<u64>, Error> {
        if let Some(argument_index) = self.argument_indexes.get(term_index as usize) {
            if let Some(resource_id) = self.arguments_buffer.get(*argument_index as usize) {
                Ok(Some(*resource_id))
            } else {
                // log::error!(
                //     "Could not get the resource ID from the arguments buffer with argument
                // index \      {argument_index} and term index {term_index}"
                // );
                // Err(Unknown)
                Ok(None)
            }
        } else {
            log::error!("Could not get the argument index for term index {term_index}");
            Err(Unknown)
        }
    }

    /// TODO: Check why this panics when called after previous call returned
    /// zero
    pub fn advance(&mut self) -> Result<u64, Error> {
        let mut multiplicity = 0_usize;
        database_call!(
            "advancing the cursor",
            CCursor_advance(self.cursor.inner, &mut multiplicity)
        )?;
        log::trace!(
            "cursor {:?} advanced, multiplicity={multiplicity}",
            self.cursor.inner
        );
        Ok(multiplicity as u64)
    }

    pub fn update_and_commit<T, U>(&mut self, f: T) -> Result<U, Error>
    where T: FnOnce(&mut OpenedCursor) -> Result<U, Error> {
        Transaction::begin_read_write(self.cursor.connection)?.update_and_commit(|_tx| f(self))
    }

    pub fn execute_and_rollback<T, U>(&mut self, f: T) -> Result<U, Error>
    where T: FnOnce(&mut OpenedCursor) -> Result<U, Error> {
        Transaction::begin_read_only(self.cursor.connection)?.execute_and_rollback(|_tx| f(self))
    }
}
