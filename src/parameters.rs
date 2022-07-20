// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

extern crate alloc;

use crate::error::Error;
use crate::root::{
    CException, CParameters, CParameters_destroy, CParameters_newEmptyParameters,
    CParameters_setString,
};
use alloc::ffi::CString;
use std::fmt::{Display, Formatter};
use std::panic::AssertUnwindSafe;
use std::ptr;

pub enum FactDomain {
    ASSERTED,
    INFERRED,
    ALL
}

pub struct Parameters {
    pub(crate) inner: *mut CParameters,
}

impl Display for Parameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parameters[]") // TODO: show keys and values (currently not possible)
    }
}

impl Drop for Parameters {
    fn drop(&mut self) {
        unsafe {
            CParameters_destroy(self.inner);
            log::debug!("Destroyed params");
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

    pub fn fact_domain(self, fact_domain: FactDomain) -> Result<Self, Error> {
        match fact_domain {
            FactDomain::ASSERTED=> self.set_string("fact-domain", "explicit")?,
            FactDomain::INFERRED=> self.set_string("fact-domain", "derived")?,
            FactDomain::ALL=> self.set_string("fact-domain", "all")?,
        };
        Ok(self)
    }

    pub fn switch_off_file_access_sandboxing(self) -> Result<Self, Error> {
        self.set_string("sandbox-directory", "")?;
        Ok(self)
    }
}
