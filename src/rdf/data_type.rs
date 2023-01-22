// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
use {
    crate::{Error, Error::UnknownDataType},
    num_enum::TryFromPrimitive,
    phf::phf_map,
    serde::Serialize,
};

static XSD_DATA_TYPE_MAP: phf::Map<&'static str, DataType> = phf_map! {
    "http://www.w3.org/2001/XMLSchema#boolean" => DataType::Boolean,
    "http://www.w3.org/2001/XMLSchema#byte" => DataType::Byte,
    "http://www.w3.org/2001/XMLSchema#date" => DataType::Date,
    "http://www.w3.org/2001/XMLSchema#dateTime" => DataType::DateTime,
    "http://www.w3.org/2001/XMLSchema#dateTimeStamp" => DataType::DateTimeStamp,
    "http://www.w3.org/2001/XMLSchema#day" => DataType::Day,
    "http://www.w3.org/2001/XMLSchema#dayTimeDuration" => DataType::DayTimeDuration,
    "http://www.w3.org/2001/XMLSchema#decimal" => DataType::Decimal,
    "http://www.w3.org/2001/XMLSchema#double" => DataType::Double,
    "http://www.w3.org/2001/XMLSchema#duration" => DataType::Duration,
    "http://www.w3.org/2001/XMLSchema#float" => DataType::Float,
    "http://www.w3.org/2001/XMLSchema#int" => DataType::Int,
    "http://www.w3.org/2001/XMLSchema#integer" => DataType::Integer,
    "http://www.w3.org/2001/XMLSchema#long" => DataType::Long,
    "http://www.w3.org/2001/XMLSchema#month" => DataType::Month,
    "http://www.w3.org/2001/XMLSchema#monthDay" => DataType::MonthDay,
    "http://www.w3.org/2001/XMLSchema#negativeInteger" => DataType::NegativeInteger,
    "http://www.w3.org/2001/XMLSchema#nonNegativeInteger" => DataType::NonNegativeInteger,
    "http://www.w3.org/2001/XMLSchema#nonPositiveInteger" => DataType::NonPositiveInteger,
    "http://www.w3.org/2001/XMLSchema#short" => DataType::Short,
    "http://www.w3.org/2001/XMLSchema#string" => DataType::String,
    "http://www.w3.org/2001/XMLSchema#time" => DataType::Time,
    "http://www.w3.org/2001/XMLSchema#unsignedByte" => DataType::UnsignedByte,
    "http://www.w3.org/2001/XMLSchema#unsignedInt" => DataType::UnsignedInt,
    "http://www.w3.org/2001/XMLSchema#unsignedLong" => DataType::UnsignedLong,
    "http://www.w3.org/2001/XMLSchema#unsignedShort" => DataType::UnsignedShort,
    "http://www.w3.org/2001/XMLSchema#year" => DataType::Year,
    "http://www.w3.org/2001/XMLSchema#yearMonth" => DataType::YearMonth,
    "http://www.w3.org/2001/XMLSchema#yearMonthDuration" => DataType::YearMonthDuration,
};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, TryFromPrimitive, Serialize)]
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

impl Default for DataType {
    /// Choosing boolean here as the default type because the default
    /// for `LexicalValueUnion` is a boolean false.
    fn default() -> Self { DataType::Boolean }
}

impl DataType {
    pub fn from_datatype_id(data_type_id: u8) -> Result<DataType, Error> {
        DataType::try_from(data_type_id).map_err(|_err| UnknownDataType { data_type_id })
    }

    pub fn from_xsd_iri(iri: &str) -> Result<Self, Error> {
        if let Some(data_type) = XSD_DATA_TYPE_MAP.get(iri) {
            Ok(data_type.clone())
        } else {
            Err(Error::UnknownXsdDataType { data_type_iri: iri.to_string() })
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        match self {
            DataType::String | DataType::PlainLiteral => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_iri(&self) -> bool {
        match self {
            DataType::AnyUri | DataType::IriReference => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_boolean(&self) -> bool {
        match self {
            DataType::Boolean => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_date(&self) -> bool {
        match self {
            DataType::Date => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_date_time(&self) -> bool {
        match self {
            DataType::DateTime => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_decimal(&self) -> bool {
        match self {
            DataType::Decimal => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_date_time_stamp(&self) -> bool {
        match self {
            DataType::DateTimeStamp => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_duration(&self) -> bool {
        match self {
            DataType::Duration => true,
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
