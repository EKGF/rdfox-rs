// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

// extern crate libc;

use std::{
    ffi::CStr,
    fmt::{Display, Formatter},
    panic::catch_unwind,
    str::Utf8Error,
};

use crate::{
    root::{CException, CException_getExceptionName, CException_what},
    Error,
    Error::Unknown,
};

impl CException {
    pub fn handle<F>(action: &'static str, f: F) -> Result<(), Error>
    where F: FnOnce() -> *const CException + std::panic::UnwindSafe {
        unsafe {
            let result = catch_unwind(|| {
                let exception = f();
                if exception.is_null() {
                    Ok(())
                } else {
                    log::error!("While {action}: {:}", *exception);
                    Err(Unknown) // TODO: Map to proper errors
                }
            });
            match result {
                Ok(res) => {
                    match res {
                        Ok(..) => Ok(()),
                        Err(err) => {
                            panic!("RDFox panicked while performing {action}: {err:?}")
                        },
                    }
                },
                Err(err) => {
                    panic!("RDFox panicked while performing {action}: {err:?}")
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
    ($action:expr, $function:expr) => {{
        // log::trace!("{} at line {}", stringify!($function), line!());
        log::trace!("{}", $action);
        crate::root::CException::handle(
            $action,
            core::panic::AssertUnwindSafe(|| unsafe { $function }),
        )
    }};
}
