// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{
    fmt::{Debug, Display, Formatter},
    mem::ManuallyDrop,
    ops::Deref,
    str::FromStr,
};

use iref::IriBuf;

use crate::{DataType, Error, Error::Unknown};

union LexicalValueUnion {
    pub iri:        ManuallyDrop<IriBuf>,
    #[allow(dead_code)]
    pub string:     ManuallyDrop<String>,
    pub blank_node: ManuallyDrop<String>,
    pub boolean:    bool,
}

pub struct LexicalValue {
    pub data_type: DataType,
    value:         LexicalValueUnion,
}

impl PartialEq for LexicalValue {
    fn eq(&self, other: &Self) -> bool {
        if self.data_type != other.data_type {
            return false
        }
        unsafe {
            match self.data_type {
                DataType::AnyUri | DataType::IriReference => self.value.iri == other.value.iri,
                DataType::String | DataType::PlainLiteral => {
                    self.value.string == other.value.string
                },
                DataType::BlankNode => self.value.blank_node == other.value.blank_node,
                DataType::Boolean => self.value.boolean == other.value.boolean,
                _ => panic!("Cannot compare, unimplemented datatype"),
            }
        }
    }
}

impl std::cmp::Eq for LexicalValue {}

impl std::hash::Hash for LexicalValue {
    fn hash<H>(&self, state: &mut H)
    where H: std::hash::Hasher {
        self.data_type.hash(state);
        unsafe {
            match self.data_type {
                DataType::AnyUri | DataType::IriReference => self.value.iri.hash(state),
                DataType::String | DataType::PlainLiteral => self.value.string.hash(state),
                DataType::BlankNode => self.value.blank_node.hash(state),
                DataType::Boolean => self.value.boolean.hash(state),
                _ => panic!("Cannot hash, unimplemented datatype"),
            }
        }
    }
}

impl Debug for LexicalValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Just show the value in its most turtle-like form
        unsafe {
            match self {
                LexicalValue {
                    data_type: DataType::AnyUri | DataType::IriReference,
                    value: LexicalValueUnion {
                        iri,
                    },
                } => {
                    write!(f, "<{}>", iri.deref())
                },
                LexicalValue {
                    data_type: DataType::String | DataType::PlainLiteral,
                    value: LexicalValueUnion {
                        string,
                    },
                } => {
                    write!(f, "{:?}", string.deref())
                },
                LexicalValue {
                    data_type: DataType::Boolean,
                    value:
                        LexicalValueUnion {
                            boolean,
                        },
                } => {
                    write!(f, "{}", boolean)
                },
                LexicalValue {
                    data_type: DataType::BlankNode,
                    value:
                        LexicalValueUnion {
                            blank_node,
                        },
                } => {
                    write!(f, "_:{}", blank_node.as_str())
                },
                &_ => {
                    write!(f, "unsupported type {:?}", self.data_type)
                },
            }
        }
    }
}

impl Display for LexicalValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.data_type {
            DataType::IriReference | DataType::AnyUri => {
                write!(f, "<{}>", self.as_iri().unwrap().as_str())
            },
            DataType::BlankNode => {
                write!(f, "_:{}", self.as_string().unwrap().as_str())
            },
            _ => {
                write!(
                    f,
                    "{:?}={}",
                    self.data_type,
                    self.as_string().unwrap().as_str()
                )
            },
        }
    }
}

impl Clone for LexicalValue {
    fn clone(&self) -> Self {
        match self.data_type {
            DataType::IriReference | DataType::AnyUri => {
                if let Some(iri) = self.as_iri() {
                    LexicalValue {
                        data_type: self.data_type,
                        value:     LexicalValueUnion {
                            iri: ManuallyDrop::new(iri),
                        },
                    }
                } else {
                    todo!("the situation where the iri in a lexical value is empty")
                }
            },
            _ => {
                todo!("dealing with other datatypes")
            },
        }
    }
}

impl LexicalValue {
    pub fn as_iri(&self) -> Option<IriBuf> {
        match self.data_type {
            DataType::IriReference | DataType::AnyUri => unsafe {
                let LexicalValue {
                    value: LexicalValueUnion {
                        iri,
                    },
                    ..
                } = self;
                Some(iri.deref().clone())
            },
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self.data_type {
            DataType::IriReference | DataType::AnyUri => {
                self.as_iri().map(|iri| String::from(iri.as_str()))
            },
            DataType::String | DataType::PlainLiteral => unsafe {
                let LexicalValue {
                    value: LexicalValueUnion {
                        string,
                    },
                    ..
                } = self;
                Some(string.deref().clone())
            },
            DataType::BlankNode => unsafe {
                let LexicalValue {
                    value:
                        LexicalValueUnion {
                            blank_node,
                        },
                    ..
                } = self;
                Some(blank_node.deref().clone())
            },
            _ => {
                panic!("Data type {:?} not yet supported", self.data_type);
            },
        }
    }

    pub fn from_type_and_c_buffer(
        data_type: DataType,
        buffer: &[u8],
    ) -> Result<Option<LexicalValue>, Error> {
        let str_buffer = std::ffi::CStr::from_bytes_until_nul(buffer)
            .map_err(|err| {
                log::error!("Cannot read buffer: {err:?}");
                Unknown // TODO
            })?
            .to_str()
            .map_err(|err| {
                log::error!("Cannot convert buffer to string: {err:?}");
                Unknown // TODO
            })?;
        Self::from_type_and_buffer(data_type, str_buffer)
    }

    pub fn from_type_and_buffer(
        data_type: DataType,
        buffer: &str,
    ) -> Result<Option<LexicalValue>, Error> {
        match data_type {
            DataType::AnyUri | DataType::IriReference => {
                Ok(Some(LexicalValue {
                    data_type,
                    value: LexicalValueUnion {
                        iri: ManuallyDrop::new(IriBuf::from_str(buffer)?),
                    },
                }))
            },
            DataType::BlankNode => {
                Ok(Some(LexicalValue {
                    data_type,
                    value: LexicalValueUnion {
                        blank_node: ManuallyDrop::new(buffer.to_string()),
                    },
                }))
            },
            DataType::Boolean => {
                Ok(Some(LexicalValue {
                    data_type,
                    value: LexicalValueUnion {
                        boolean: buffer.starts_with("true"),
                    },
                }))
            },
            DataType::String | DataType::PlainLiteral => {
                Ok(Some(LexicalValue {
                    data_type,
                    value: LexicalValueUnion {
                        string: ManuallyDrop::new(buffer.to_string()),
                    },
                }))
            },
            DataType::UnboundValue => Ok(None),
            _ => {
                log::warn!("Unsupported datatype: {data_type:?} value={buffer}");
                Err(Unknown)
            },
        }
    }
}
