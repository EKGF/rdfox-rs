// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

pub use cursor::Cursor;
pub use opened_cursor::OpenedCursor;
pub use cursor_row::CursorRow;

mod cursor;
mod opened_cursor;
mod cursor_row;

#[derive(Debug)]
pub struct ResourceValue {
    pub prefix: &'static str,
    pub value: &'static str,
}
