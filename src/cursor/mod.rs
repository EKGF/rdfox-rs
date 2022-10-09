// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CStr;

pub use cursor::Cursor;
pub use cursor_row::CursorRow;
pub use opened_cursor::OpenedCursor;

use crate::Error;

#[allow(clippy::module_inception)]
mod cursor;
mod cursor_row;
mod opened_cursor;

pub fn ptr_to_cstr<'b>(data: *const u8, len: usize) -> Result<&'b CStr, Error> {
    unsafe {
        let slice = std::slice::from_raw_parts(data, len as usize);
        Ok(CStr::from_bytes_with_nul_unchecked(slice))
    }
}
