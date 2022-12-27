// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

// extern crate libc;

use std::{
    ffi::CStr,
    fmt::{Display, Formatter},
    panic::catch_unwind,
    str::Utf8Error,
};

pub use crate::root::CException;
use crate::{
    root::{CException_getExceptionName, CException_what},
    Error::{self},
};

impl CException {
    pub fn handle<F>(action: &str, f: F) -> Result<(), Error>
    where F: FnOnce() -> *const CException + std::panic::UnwindSafe {
        unsafe {
            let result = catch_unwind(|| {
                let c_exception = f();
                if c_exception.is_null() {
                    Ok(())
                } else {
                    Err(Error::Exception {
                        action:  action.to_string(),
                        message: format!("{:}", *c_exception).replace("RDFoxException: ", ""),
                    })
                }
            });
            match result {
                Ok(res) => {
                    match res {
                        Ok(..) => Ok(()),
                        Err(err) => {
                            panic!("{err:}")
                        },
                    }
                },
                Err(err) => {
                    panic!("RDFox panicked while {action}: {err:?}")
                },
            }
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
                return writeln!(f, "{:}: {:}", name, what)
            };
        };
        f.write_str("Could not show exception, unicode error")
    }
}

#[macro_export]
macro_rules! database_call {
    ($function:expr) => {{
        $crate::exception::CException::handle(
            "unknown database action",
            core::panic::AssertUnwindSafe(|| unsafe { $function }),
        )
    }};
    ($action:expr, $function:expr) => {{
        // tracing::trace!("{} at line {}", stringify!($function), line!());
        tracing::trace!(target: $crate::LOG_TARGET_DATABASE, "{}", $action);
        $crate::exception::CException::handle(
            $action,
            core::panic::AssertUnwindSafe(|| unsafe { $function }),
        )
    }};
}
