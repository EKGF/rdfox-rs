// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
use num_enum::TryFromPrimitive;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum DataType {
    UnboundValue = 0,
    BlankNode = 1,
    IriReference = 2,
    Literal = 3,
    AnyUri = 4,
    String = 5,
    PlainLiteral = 6,
    Boolean = 7,
    DateTime = 8,
    DateTimeStamp = 9,
    Time = 10,
    Date = 11,
    YearMonth = 12,
    Year = 13,
    MonthDay = 14,
    Day = 15,
    Month = 16,
    Duration = 17,
    YearMonthDuration = 18,
    DayTimeDuration = 19,
    Double = 20,
    Float = 21,
    Decimal = 22,
    Integer = 23,
    NonNegativeInteger = 24,
    NonPositiveInteger = 25,
    NegativeInteger = 26,
    PositiveInteger = 27,
    Long = 28,
    Int = 29,
    Short = 30,
    Byte = 31,
    UnsignedLong = 32,
    UnsignedInt = 33,
    UnsignedShort = 34,
    UnsignedByte = 35,
}
