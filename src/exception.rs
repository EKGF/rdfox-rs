// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

// extern crate libc;

use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::panic::catch_unwind;
use std::str::Utf8Error;

use crate::root::{CException, CException_getExceptionName, CException_what};
use crate::Error;
use crate::Error::UNKNOWN;

impl CException {
    pub fn handle<F>(f: F) -> Result<(), Error>
    where
        F: FnOnce() -> *const CException + std::panic::UnwindSafe,
    {
        unsafe {
            let did_panic = catch_unwind(|| {
                let exception = f();
                if exception.is_null() {
                    return Ok(());
                }
                log::error!("{:}", *exception);
                Err(UNKNOWN)
            });
            if did_panic.is_err() {
                log::error!("RDFox panicked");
                panic!("RDFox panicked 2");
            };
            Ok(())
        }
    }

    pub fn name(&self) -> Result<&'static str, Utf8Error> {
        let name = unsafe { CStr::from_ptr(CException_getExceptionName(self)) };
        name.to_str()
    }

    pub fn what(&self) -> Result<&'static str, Utf8Error> {
        let what = unsafe { CStr::from_ptr(CException_what(self)) };
        what.to_str()
    }
}

impl Display for CException {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Ok(name) = self.name() {
            if let Ok(what) = self.what() {
                return writeln!(f, "{:}: {:}", name, what);
            };
        };
        f.write_str("Could not show exception, unicode error")
    }
}
