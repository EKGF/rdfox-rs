// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::root::{CException, CPrefixes, CPrefixes_newDefaultPrefixes};
use crate::Error;

pub struct Prefixes {
    pub(crate) inner: *mut CPrefixes,
}

impl Prefixes {
    pub fn default() -> Result<Self, Error> {
        let mut prefixes = Self {
            inner: ptr::null_mut(),
        };
        CException::handle(AssertUnwindSafe(|| unsafe {
            CPrefixes_newDefaultPrefixes(&mut prefixes.inner)
        }))?;
        Ok(prefixes)
    }
}
