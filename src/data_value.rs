// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::fmt::{Debug, Display, Formatter};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::str::FromStr;
use iref::IriBuf;
use crate::{DataType, Error};
use crate::Error::UNKNOWN;

union DataValueUnion {
    pub iri: ManuallyDrop<IriBuf>,
    #[allow(dead_code)]
    pub string: ManuallyDrop<String>,
    pub blank_node: ManuallyDrop<String>
}

pub struct DataValue {
    pub data_type: DataType,
    value: DataValueUnion,
}

impl Debug for DataValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Just show the value in its most turtle-like form
        unsafe {
            match self {
                DataValue {
                    data_type: DataType::AnyUri | DataType::IriReference,
                    value: DataValueUnion{ iri }
                } => {
                    let iri_buf = iri.deref();
                    write!(f, "<{}>", iri_buf)
                }
                DataValue {
                    data_type: DataType::String | DataType::PlainLiteral,
                    value: DataValueUnion{
                        string
                    }
                } => {
                    write!(f, "{:?}", string)
                }
                DataValue {
                    data_type: DataType::BlankNode,
                    value: DataValueUnion{
                        blank_node
                    }
                } => {
                    write!(f, "_:{}", blank_node.as_str())
                }
                &_ => {
                    write!(f, "unsupported type")
                }
            }
        }
    }
}

impl Display for DataValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.data_type {
            DataType::IriReference | DataType::AnyUri => write!(f, "<{}>", self.as_iri().unwrap().as_str()),
            _ => write!(f, "{:?}={}", self.data_type, self.as_string().unwrap().as_str())
        }
    }
}

impl DataValue {
    pub fn as_iri(&self) -> Option<IriBuf> {
        match self.data_type {
            DataType::IriReference | DataType::AnyUri => {
                unsafe {
                    let DataValue { value: DataValueUnion { iri }, .. } = self;
                    Some(iri.deref().clone())
                }
            }
            _ => None
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self.data_type {
            DataType::IriReference | DataType::AnyUri =>
                self.as_iri().map(|iri| String::from(iri.as_str())),
            DataType::String | DataType::PlainLiteral =>
                unsafe {
                    let DataValue { value: DataValueUnion { string }, .. } = self;
                    Some(string.deref().clone())
                },
            _ => {
                panic!("Data type {:?} not yet supported", self.data_type);
            }
        }
    }

    pub fn from_type_and_c_buffer(data_type: DataType, buffer: &[u8]) -> Result<DataValue, Error> {
        let str_buffer = std::ffi::CStr::from_bytes_until_nul(buffer)
            .map_err(|err| {
                log::error!("Cannot read buffer: {err:?}");
                UNKNOWN // TODO
            })?
            .to_str()
            .map_err(|err| {
                log::error!("Cannot convert buffer to string: {err:?}");
                UNKNOWN // TODO
            })?;
        Self::from_type_and_buffer(data_type, str_buffer)
    }

    pub fn from_type_and_buffer(data_type: DataType, buffer: &str) -> Result<DataValue, Error> {
        match data_type {
            DataType::AnyUri | DataType::IriReference => {
                Ok(DataValue {
                    data_type,
                    value: DataValueUnion {
                        iri: ManuallyDrop::new(IriBuf::from_str(buffer)?),
                    },
                })
            }
            DataType::BlankNode => {
                Ok(DataValue {
                    data_type,
                    value: DataValueUnion {
                        blank_node: ManuallyDrop::new(buffer.to_string())
                    }
                })
            }
            _ => {
                log::warn!("Unsupported datatype: {data_type:?} value={buffer}");
                Err(UNKNOWN)
            }
        }
    }
}