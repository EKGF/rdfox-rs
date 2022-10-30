// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{
    fmt::{Debug, Display, Formatter},
    mem::ManuallyDrop,
    str::FromStr,
};

use iref::{Iri, IriBuf};

use crate::{rdf::LexicalValueUnion, DataType, Error, Error::Unknown, Term};

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
            if self.data_type.is_iri() {
                self.value.iri == other.value.iri
            } else if self.data_type.is_string() {
                self.value.string == other.value.string
            } else if self.data_type.is_boolean() {
                self.value.boolean == other.value.boolean
            } else if self.data_type.is_signed_integer() {
                self.value.signed_integer == other.value.signed_integer
            } else if self.data_type.is_unsigned_integer() {
                self.value.unsigned_integer == other.value.unsigned_integer
            } else if self.data_type.is_blank_node() {
                self.value.blank_node == other.value.blank_node
            } else {
                panic!("Cannot compare, unimplemented datatype")
            }
        }
    }
}

impl Eq for LexicalValue {}

impl std::hash::Hash for LexicalValue {
    fn hash<H>(&self, state: &mut H)
    where H: std::hash::Hasher {
        self.data_type.hash(state);
        unsafe {
            if self.data_type.is_iri() {
                self.value.iri.hash(state)
            } else if self.data_type.is_string() {
                self.value.string.hash(state)
            } else if self.data_type.is_blank_node() {
                self.value.blank_node.hash(state)
            } else if self.data_type.is_boolean() {
                self.value.boolean.hash(state)
            } else if self.data_type.is_signed_integer() {
                self.value.signed_integer.hash(state)
            } else if self.data_type.is_unsigned_integer() {
                self.value.unsigned_integer.hash(state)
            } else {
                panic!("Cannot hash, unimplemented datatype")
            }
        }
    }
}

impl Debug for LexicalValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "LexicalValue({:?},", self.data_type)?;
        unsafe {
            if self.data_type.is_iri() {
                write!(f, "<{}>)", self.value.iri.as_str())?
            } else if self.data_type.is_string() {
                write!(f, "\"{}\"", self.value.string.as_str())?
            } else if self.data_type.is_blank_node() {
                write!(f, "_:{}", self.value.blank_node.as_str())?
            } else if self.data_type.is_boolean() {
                write!(f, "{}", self.value.boolean)?
            } else if self.data_type.is_signed_integer() {
                write!(f, "{}", self.value.signed_integer)?
            } else if self.data_type.is_unsigned_integer() {
                write!(f, "{}", self.value.unsigned_integer)?
            } else {
                panic!("Cannot compare, unimplemented datatype")
            }
        }
        write!(f, ")")
    }
}

// impl Debug for LexicalValue {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         // Just show the value in its most turtle-like form
//         unsafe {
//             match self {
//                 LexicalValue {
//                     data_type: DataType::AnyUri | DataType::IriReference,
//                     value: LexicalValueUnion {
//                         iri,
//                     },
//                 } => {
//                     write!(f, "<{}>", iri.deref())
//                 },
//                 LexicalValue {
//                     data_type: DataType::String | DataType::PlainLiteral,
//                     value: LexicalValueUnion {
//                         string,
//                     },
//                 } => {
//                     write!(f, "{:?}", string.deref())
//                 },
//                 LexicalValue {
//                     data_type: DataType::Boolean,
//                     value:
//                         LexicalValueUnion {
//                             boolean,
//                         },
//                 } => {
//                     write!(f, "{}", boolean)
//                 },
//                 LexicalValue {
//                     data_type: DataType::BlankNode,
//                     value:
//                         LexicalValueUnion {
//                             blank_node,
//                         },
//                 } => {
//                     write!(f, "_:{}", blank_node.as_str())
//                 },
//                 &_ => {
//                     write!(f, "unsupported type {:?}", self.data_type)
//                 },
//             }
//         }
//     }
// }

impl Display for LexicalValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.data_type.is_iri() {
            write!(f, "<{}>", self.as_iri().unwrap().as_str())
        } else if self.data_type.is_blank_node() {
            write!(f, "_:{}", self.as_string().unwrap().as_str())
        } else {
            write!(
                f,
                "{:?}={}",
                self.data_type,
                self.as_string().unwrap().as_str()
            )
        }
    }
}

impl Clone for LexicalValue {
    // noinspection RsUnreachableCode
    fn clone(&self) -> Self {
        if self.data_type.is_iri() {
            if let Some(ref iri) = self.as_iri() {
                LexicalValue {
                    data_type: self.data_type,
                    value:     LexicalValueUnion::new_iri(iri),
                }
            } else {
                todo!("the situation where the iri in a lexical value is empty")
            }
        } else if self.data_type.is_blank_node() {
            if let Some(blank_node) = self.as_str() {
                LexicalValue::new_blank_node_with_datatype(blank_node, self.data_type).unwrap()
            } else {
                todo!("the situation where the blank_node in a lexical value is empty")
            }
        } else if self.data_type.is_string() {
            if let Some(str) = self.as_str() {
                LexicalValue::new_string_with_datatype(str, self.data_type).unwrap()
            } else {
                todo!("the situation where the string in a lexical value is empty")
            }
        } else if self.data_type.is_boolean() {
            if let Some(boolean) = self.as_boolean() {
                LexicalValue::new_boolean_with_datatype(boolean, self.data_type).unwrap()
            } else {
                todo!("the situation where the boolean in a lexical value is not a boolean")
            }
        } else {
            todo!("dealing with other datatypes: {:?}", self.data_type)
        }
    }
}

impl LexicalValue {
    pub fn as_term(&self) -> Term {
        match self.data_type {
            DataType::IriReference | DataType::AnyUri => Term::Iri(self.clone()),
            DataType::BlankNode => Term::BlankNode(self.clone()),
            _ => Term::Literal(self.clone()),
        }
    }

    pub fn as_iri(&self) -> Option<Iri> {
        if self.data_type.is_iri() {
            Some(unsafe { self.value.iri.as_iri() })
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if self.data_type.is_iri() {
            unsafe { Some(self.value.iri.as_str()) }
        } else if self.data_type.is_string() {
            unsafe { Some(self.value.string.as_str()) }
        } else if self.data_type.is_signed_integer() {
            None
        } else if self.data_type.is_unsigned_integer() {
            None
        } else if self.data_type.is_blank_node() {
            unsafe { Some(self.value.blank_node.as_str()) }
        } else if self.data_type.is_boolean() {
            unsafe {
                if self.value.boolean {
                    Some("true")
                } else {
                    Some("false")
                }
            }
        } else {
            panic!("Data type {:?} not yet supported", self.data_type);
        }
    }

    pub fn as_string(&self) -> Option<String> { self.as_str().map(|v| v.to_owned()) }

    pub fn as_boolean(&self) -> Option<bool> {
        match self.data_type {
            DataType::Boolean => Some(unsafe { self.value.boolean }),
            _ => None,
        }
    }

    pub fn as_signed_long(&self) -> Option<i64> {
        if self.data_type.is_signed_integer() {
            Some(unsafe { self.value.signed_integer })
        } else {
            None
        }
    }

    pub fn as_unsigned_long(&self) -> Option<u64> {
        if self.data_type.is_unsigned_integer() {
            Some(unsafe { self.value.unsigned_integer })
        } else {
            None
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
                let iri = IriBuf::from_str(buffer)?;
                Ok(Some(LexicalValue::new_iri_with_datatype(
                    &iri.as_iri(),
                    data_type,
                )?))
            },
            DataType::BlankNode => {
                Ok(Some(LexicalValue::new_blank_node_with_datatype(
                    buffer, data_type,
                )?))
            },
            DataType::Boolean => {
                match buffer {
                    "true" | "false" => {
                        Ok(Some(LexicalValue::new_boolean_with_datatype(
                            buffer.starts_with("true"),
                            data_type,
                        )?))
                    },
                    _ => {
                        Err(Error::UnknownNTriplesValue {
                            value: buffer.to_string(),
                        })
                    },
                }
            },
            DataType::String | DataType::PlainLiteral => {
                Ok(Some(LexicalValue::new_string_with_datatype(
                    buffer, data_type,
                )?))
            },
            DataType::UnboundValue => Ok(None),
            _ => {
                log::warn!("Unsupported datatype: {data_type:?} value={buffer}");
                Err(Unknown)
            },
        }
    }

    pub fn from_iri(iri: &Iri) -> Result<Self, Error> {
        Ok(LexicalValue {
            data_type: DataType::IriReference,
            value:     LexicalValueUnion {
                iri: ManuallyDrop::new(IriBuf::from(iri)),
            },
        })
    }

    pub fn new_plain_literal_string(str: &str) -> Result<Self, Error> {
        Self::new_string_with_datatype(str, DataType::PlainLiteral)
    }

    pub fn new_plain_literal_boolean(boolean: bool) -> Result<Self, Error> {
        Self::new_string_with_datatype(boolean.to_string().as_str(), DataType::PlainLiteral)
    }

    pub fn new_string_with_datatype(str: &str, data_type: DataType) -> Result<Self, Error> {
        assert!(&data_type.is_string(), "{data_type:?} is not a string type");
        Ok(LexicalValue {
            data_type,
            value: LexicalValueUnion::new_string(str),
        })
    }

    pub fn new_iri_from_string_with_datatype(
        iri_string: &str,
        data_type: DataType,
    ) -> Result<Self, Error> {
        let iri = IriBuf::from_str(iri_string)?;
        Self::new_iri_with_datatype(&iri.as_iri(), data_type)
    }

    pub fn new_iri_with_datatype(iri: &Iri, data_type: DataType) -> Result<Self, Error> {
        assert!(&data_type.is_iri(), "{data_type:?} is not an IRI type");
        Ok(LexicalValue {
            data_type,
            value: LexicalValueUnion::new_iri(iri),
        })
    }

    pub fn new_blank_node_with_datatype(id: &str, data_type: DataType) -> Result<Self, Error> {
        assert!(
            &data_type.is_blank_node(),
            "{data_type:?} is not a blank node type"
        );
        Ok(LexicalValue {
            data_type,
            value: LexicalValueUnion::new_blank_node(id),
        })
    }

    pub fn new_boolean(boolean: bool) -> Result<Self, Error> {
        Self::new_boolean_with_datatype(boolean, DataType::Boolean)
    }

    pub fn new_boolean_from_string(boolean_string: &str) -> Result<Self, Error> {
        Self::new_boolean_from_string_with_datatype(boolean_string, DataType::Boolean)
    }

    pub fn new_boolean_from_string_with_datatype(
        boolean_string: &str,
        data_type: DataType,
    ) -> Result<Self, Error> {
        match boolean_string {
            "true" => Self::new_boolean_with_datatype(true, data_type),
            "false" => Self::new_boolean_with_datatype(false, data_type),
            &_ => {
                Err(Error::UnknownValueForDataType {
                    data_type,
                    value: boolean_string.to_string(),
                })
            },
        }
    }

    pub fn new_boolean_with_datatype(boolean: bool, data_type: DataType) -> Result<Self, Error> {
        assert!(
            &data_type.is_boolean(),
            "{data_type:?} is not a boolean type"
        );
        Ok(LexicalValue {
            data_type,
            value: LexicalValueUnion::new_boolean(boolean),
        })
    }

    pub fn new_signed_integer(signed_integer: i64) -> Result<Self, Error> {
        if signed_integer >= 0 {
            Self::new_unsigned_integer(signed_integer as u64)
        } else {
            Self::new_signed_integer_with_datatype(signed_integer, DataType::NegativeInteger)
        }
    }

    pub fn new_signed_integer_with_datatype(
        signed_integer: i64,
        data_type: DataType,
    ) -> Result<Self, Error> {
        assert!(
            &data_type.is_signed_integer(),
            "{data_type:?} is not an signed integer type"
        );
        Ok(LexicalValue {
            data_type,
            value: LexicalValueUnion::new_signed_integer(signed_integer),
        })
    }

    pub fn new_unsigned_integer(unsigned_integer: u64) -> Result<Self, Error> {
        Self::new_unsigned_integer_with_datatype(unsigned_integer, DataType::PositiveInteger)
    }

    pub fn new_unsigned_integer_with_datatype(
        unsigned_integer: u64,
        data_type: DataType,
    ) -> Result<Self, Error> {
        assert!(
            &data_type.is_unsigned_integer(),
            "{data_type:?} is not an unsigned integer type"
        );
        Ok(LexicalValue {
            data_type,
            value: LexicalValueUnion::new_unsigned_integer(unsigned_integer),
        })
    }

    pub fn display_turtle<'a, 'b>(&'a self) -> impl std::fmt::Display + 'a + 'b
    where 'a: 'b {
        struct TurtleLexVal<'b>(&'b LexicalValue);
        impl<'b> std::fmt::Display for TurtleLexVal<'b> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let data_type = self.0.data_type;
                unsafe {
                    if data_type.is_iri() {
                        write!(f, "<{}>", self.0.value.iri.as_str())?
                    } else if data_type.is_string() {
                        write!(f, "\"{}\"", self.0.value.string.as_str())?
                    } else if data_type.is_blank_node() {
                        write!(f, "_:{}", self.0.value.blank_node.as_str())?
                    } else if data_type.is_boolean() {
                        write!(f, "{}", self.0.value.boolean)?
                    } else if data_type.is_signed_integer() {
                        write!(f, "{}", self.0.value.signed_integer)?
                    } else if data_type.is_unsigned_integer() {
                        write!(f, "{}", self.0.value.unsigned_integer)?
                    } else {
                        panic!("Cannot compare, unimplemented datatype")
                    }
                }
                Ok(())
            }
        }
        TurtleLexVal(self)
    }
}

impl FromStr for LexicalValue {
    type Err = Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> { Self::new_plain_literal_string(str) }
}
