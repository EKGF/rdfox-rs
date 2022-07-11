// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CString;
use std::fmt::{Display, Formatter};

pub struct Graph {
    pub(crate) local_name: String,
}

impl Display for Graph {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.local_name.as_str())
    }
}

impl Graph {

    pub fn define(local_name: &str) -> Self { // TODO: Find a class for URI/IRIs that has separate base + local name and use that as param instead
        Self { local_name: local_name.to_string() }
    }

    pub fn as_c_string(&self) -> CString {
        CString::new(self.local_name.as_str()).expect("a string")
    }
}
