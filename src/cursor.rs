// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

extern crate alloc;

use alloc::ffi::CString;
use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::{DataStoreConnection, Error, Parameters, Statement};
use crate::root::{
    CCursor,
    CCursor_advance,
    CCursor_destroy,
    CCursor_open,
    CDataStoreConnection_createCursor,
    CException,
};

pub struct Cursor {
    #[allow(dead_code)]
    pub(crate) inner: *mut CCursor,
}

impl Drop for Cursor {
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

impl Cursor {
    pub fn create(
        connection: &DataStoreConnection,
        parameters: &Parameters,
        statement: &Statement,
    ) -> Result<Self, Error> {
        assert!(!connection.inner.is_null());
        assert!(!statement.prefixes.inner.is_null());
        assert!(!statement.prefixes.inner.is_null());
        let mut cursor: *mut CCursor = ptr::null_mut();
        // let base_iri: *const std::os::raw::c_char = ptr::null();
        let c_query = CString::new(statement.text.as_str()).unwrap();
        let c_query_len: u64 = c_query.as_bytes().len() as u64;
        log::debug!("Starting cursor for {:?} ({} bytes)", c_query, c_query_len);
        CException::handle(AssertUnwindSafe(|| unsafe {
            CDataStoreConnection_createCursor(
                connection.inner,
                ptr::null(),
                statement.prefixes.inner,
                c_query.as_ptr(),
                c_query_len,
                parameters.inner,
                &mut cursor,
            )
        }))?;
        log::debug!("Created cursor for {:}", statement);
        Ok(Cursor { inner: cursor })
    }

    pub fn open(&self) -> Result<std::os::raw::c_ulong, Error> {
        let mut multiplicity = 0 as std::os::raw::c_ulong;
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_open(self.inner, &mut multiplicity)
        }))?;
        log::debug!("Cursor opened");
        Ok(multiplicity)
    }

    pub fn advance(&self) -> Result<std::os::raw::c_ulong, Error> {
        let mut multiplicity = 0 as std::os::raw::c_ulong;
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_advance(self.inner, &mut multiplicity)
        }))?;
        Ok(multiplicity)
    }
}
