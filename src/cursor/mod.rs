// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CStr;
pub use cursor::Cursor;
pub use opened_cursor::OpenedCursor;
pub use cursor_row::CursorRow;
use crate::{DataType, Error};
use crate::Error::Unknown;

mod cursor;
mod opened_cursor;
mod cursor_row;

#[derive(Debug)]
pub struct ResourceValue {
    pub data_type: DataType,
    pub namespace: Option<&'static str>,
    pub value: &'static str,
}

impl ResourceValue {

    pub fn from(
        data_type: DataType,
        namespace: *const u8, namespace_len: usize,
        data: *const u8, data_len: usize
    ) -> Result<Self, Error> {
        let ns = if namespace_len == 0 {
            None
        } else {
            Some(ptr_to_cstr(namespace, namespace_len + 1)?.to_str().map_err(|err| {
                log::error!("Couldn't convert namespace due to UTF-8 error: {err:?}");
                Unknown
            })?)
        };
        Ok(Self {
            data_type,
            namespace: ns,
            value: ptr_to_cstr(data, data_len)?.to_str().unwrap(),
        })
    }
}

fn ptr_to_cstr<'b>(data: *const u8, len: usize) -> Result<&'b CStr, Error> {
    unsafe {
        let slice = std::slice::from_raw_parts(data, len as usize);
        Ok(CStr::from_bytes_with_nul_unchecked(slice))
    }
}
