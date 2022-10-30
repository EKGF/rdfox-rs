// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
use num_enum::TryFromPrimitive;

use crate::{Error, Error::UnknownDataType};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum DataType {
    UnboundValue       = 0,
    BlankNode          = 1,
    IriReference       = 2,
    Literal            = 3,
    AnyUri             = 4,
    String             = 5,
    PlainLiteral       = 6,
    Boolean            = 7,
    DateTime           = 8,
    DateTimeStamp      = 9,
    Time               = 10,
    Date               = 11,
    YearMonth          = 12,
    Year               = 13,
    MonthDay           = 14,
    Day                = 15,
    Month              = 16,
    Duration           = 17,
    YearMonthDuration  = 18,
    DayTimeDuration    = 19,
    Double             = 20,
    Float              = 21,
    Decimal            = 22,
    Integer            = 23,
    NonNegativeInteger = 24,
    NonPositiveInteger = 25,
    NegativeInteger    = 26,
    PositiveInteger    = 27,
    Long               = 28,
    Int                = 29,
    Short              = 30,
    Byte               = 31,
    UnsignedLong       = 32,
    UnsignedInt        = 33,
    UnsignedShort      = 34,
    UnsignedByte       = 35,
}

impl DataType {
    pub fn from_datatype_id(data_type_id: u8) -> Result<DataType, Error> {
        DataType::try_from(data_type_id).map_err(|_err| {
            UnknownDataType {
                data_type_id,
            }
        })
    }

    pub fn from_xsd_iri(iri: &str) -> Result<Self, Error> {
        match iri {
            "http://www.w3.org/2001/XMLSchema#boolean" => Ok(DataType::Boolean),
            "http://www.w3.org/2001/XMLSchema#string" => Ok(DataType::String),
            _ => {
                Err(Error::UnknownXsdDataType {
                    data_type_iri: iri.to_string(),
                })
            },
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        // STRING_TYPES
        match self {
            DataType::String | DataType::PlainLiteral => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_iri(&self) -> bool {
        // IRI_TYPES
        match self {
            DataType::AnyUri | DataType::IriReference => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        // IRI_TYPES
        match self {
            DataType::Boolean => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_signed_integer(&self) -> bool {
        // IRI_TYPES
        match self {
            DataType::Int |
            DataType::Integer |
            DataType::NegativeInteger |
            DataType::NonPositiveInteger |
            DataType::Long |
            DataType::Short => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_unsigned_integer(&self) -> bool {
        // IRI_TYPES
        match self {
            DataType::PositiveInteger |
            DataType::NonNegativeInteger |
            DataType::UnsignedByte |
            DataType::UnsignedInt |
            DataType::UnsignedShort |
            DataType::UnsignedLong => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_blank_node(&self) -> bool {
        // BLANK_NODE_TYPES
        match self {
            DataType::BlankNode => true,
            _ => false,
        }
    }
}
