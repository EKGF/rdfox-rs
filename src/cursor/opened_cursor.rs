// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::{Cursor, Error, Error::Unknown, root::{
    CArgumentIndex,
    CCursor,
    CCursor_advance,
    CCursor_getArgumentIndexes,
    CCursor_getArgumentsBuffer,
    CCursor_getArity,
    CCursor_open,
    CException,
    CResourceID
}, Transaction};

#[derive(Debug)]
pub struct OpenedCursor<'a> {
    pub tx: &'a Transaction<'a>,
    pub cursor: &'a Cursor<'a>,
    /// the arity (i.e., the number of columns) of the answers that the
    /// cursor computes.
    pub arity: u16,
    pub arguments_buffer: Vec<u64>,
    pub argument_indexes: Vec<u32>
}

impl<'a> OpenedCursor<'a> {
    /// Open the cursor, get the details like arity and argument info and
    /// return it as a tuple with all the details (except multiplicity)
    /// as an `OpenedCursor` and the multiplicity of the first row.
    pub(crate) fn new(cursor: &'a mut Cursor, tx: &'a Transaction<'a>) -> Result<(Self, u64), Error> {
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
        log::info!("Created OpenedCursor");
        Ok((opened_cursor, multiplicity))
    }

    fn open(c_cursor: *mut CCursor) -> Result<u64, Error> {
        let mut multiplicity = 0 as std::os::raw::c_ulong;
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_open(c_cursor, &mut multiplicity)
        }))?;
        log::info!("CCursor_open ok multiplicity={multiplicity}");
        Ok(multiplicity)
    }

    /// Returns the arity (i.e., the number of columns) of the answers that the
    /// cursor computes.
    fn arity(c_cursor: *mut CCursor) -> Result<u16, Error> {
        let mut arity = 0_u64;
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_getArity(c_cursor, &mut arity)
        }))?;
        // Trimming it down, we don't expect more than 2^16 columns do we?
        Ok(arity as u16)
    }

    pub fn arguments_buffer(c_cursor: *mut CCursor) -> Result<Vec<u64>, Error> {
        let mut buffer: *const CResourceID = ptr::null_mut();
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_getArgumentsBuffer(c_cursor, &mut buffer)
        }))?;
        let mut count = 0_usize;
        unsafe {
            let mut p = buffer;
            while !p.is_null() {
                count += 1;
                let resource_id: CResourceID = *p as CResourceID;
                if resource_id == 0 {
                    break;
                }
                log::trace!("{count} resource_id={:?}", resource_id);
                p = p.offset(1);
            }
        }
        let mut result = Vec::new();
        if count > 1 {
            unsafe {
                result.extend(std::slice::from_raw_parts(buffer, count - 1));
            }
        }
        log::info!("CCursor_getArgumentsBuffer: {result:?}");
        Ok(result)
    }

    fn argument_indexes(c_cursor: *mut CCursor, arity: u16) -> Result<Vec<u32>, Error> {
        let mut indexes: *const CArgumentIndex = ptr::null_mut();
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_getArgumentIndexes(c_cursor, &mut indexes)
        }))?;
        if indexes.is_null() { return Err(Unknown) }
        let mut result = Vec::new();
        unsafe {
            result.extend(std::slice::from_raw_parts(indexes, arity as usize));
        }
        log::trace!("CCursor_getArgumentIndexes: {result:?}");
        Ok(result)
    }

    /// TODO: Check why this panics when called after previous call returned zero
    pub fn advance(&mut self) -> Result<u64, Error> {
        let mut multiplicity = 0 as std::os::raw::c_ulong;
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_advance(self.cursor.inner, &mut multiplicity)
        }))?;
        log::debug!("cursor {:?} advanced, multiplicity={multiplicity}", self.cursor.inner);
        Ok(multiplicity)
    }

    pub fn update_and_commit<T, U>(&mut self, f: T) -> Result<U, Error>
        where T: FnOnce(&mut OpenedCursor) -> Result<U, Error> {
        Transaction::begin_read_write(self.cursor.connection)?.update_and_commit(|_tx| {
            f(self)
        })
    }

    pub fn execute_and_rollback<T, U>(&mut self, f: T) -> Result<U, Error>
        where T: FnOnce(&mut OpenedCursor) -> Result<U, Error> {
        Transaction::begin_read_only(self.cursor.connection)?.execute_and_rollback(|_tx| {
            f(self)
        })
    }
}