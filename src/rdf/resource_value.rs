// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
use super::DataType;
use crate::{cursor, Error, Error::Unknown};

#[derive(Debug)]
pub struct ResourceValue {
    pub data_type: DataType,
    pub namespace: Option<&'static str>,
    pub value:     &'static str,
}

impl ResourceValue {
    pub fn from(
        data_type: DataType,
        namespace: *const u8,
        namespace_len: usize,
        data: *const u8,
        data_len: usize,
    ) -> Result<Self, Error> {
        let ns = if namespace_len == 0 {
            None
        } else {
            Some(
                cursor::ptr_to_cstr(namespace, namespace_len + 1)?
                    .to_str()
                    .map_err(|err| {
                        tracing::error!("Couldn't convert namespace due to UTF-8 error: {err:?}");
                        Unknown
                    })?,
            )
        };
        Ok(Self {
            data_type,
            namespace: ns,
            value: cursor::ptr_to_cstr(data, data_len)?.to_str().unwrap(),
        })
    }
}
