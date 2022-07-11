// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

extern crate alloc;

use crate::root::{
    CException, CParameters, CParameters_destroy, CParameters_newEmptyParameters,
    CParameters_setString,
};
use crate::Error;
use alloc::ffi::CString;
use std::panic::AssertUnwindSafe;
use std::ptr;

pub struct Parameters {
    pub(crate) inner: *mut CParameters,
}

impl Drop for Parameters {
    fn drop(&mut self) {
        unsafe {
            if !self.inner.is_null() {
                CParameters_destroy(self.inner);
                self.inner = ptr::null_mut();
                log::debug!("Destroyed params");
            }
        }
    }
}

impl Parameters {
    pub fn empty() -> Result<Self, Error> {
        let mut parameters: *mut CParameters = ptr::null_mut();
        CException::handle(AssertUnwindSafe(|| unsafe {
            CParameters_newEmptyParameters(&mut parameters)
        }))?;
        Ok(Parameters { inner: parameters })
    }

    pub fn set_string(&self, key: &str, value: &str) -> Result<(), Error> {
        let c_key = CString::new(key).unwrap();
        let c_value = CString::new(value).unwrap();
        CException::handle(AssertUnwindSafe(|| unsafe {
            CParameters_setString(self.inner, c_key.as_ptr(), c_value.as_ptr())
        }))?;
        log::debug!("param {key}={value}");
        Ok(())
    }

    pub fn fact_domain_all(&self) -> Result<(), Error> {
        self.set_string("fact-domain", "all")
    }
}
