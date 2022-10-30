// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

extern crate alloc;

use thiserror::Error;

use crate::DataType;

#[derive(Error, Debug)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Unknown Error")]
    Unknown,
    #[error("Unknown data type {data_type_id}")]
    UnknownDataType { data_type_id: u8 },
    #[error("Unknown value [{value}] for data type {data_type:?}")]
    UnknownValueForDataType {
        data_type: DataType,
        value:     String,
    },
    #[error("Unknown XSD data type {data_type_iri}")]
    UnknownXsdDataType { data_type_iri: String },
    #[error("Unknown literal value in N-Triples format: {value}")]
    UnknownNTriplesValue { value: String },
    #[error(
        "The multiplicity ({multiplicity}) of a cursor row exceeded the maximum number of rows \
         ({maxrow}) for query:\n{query}"
    )]
    MultiplicityExceededMaximumNumberOfRows {
        maxrow:       u64,
        multiplicity: u64,
        query:        String,
    },
    #[error("Maximum number of rows ({maxrow}) has been exceeded for query:\n{query}")]
    ExceededMaximumNumberOfRows { maxrow: u64, query: String },
    #[error("Could not find a license key")]
    RDFoxLicenseFileNotFound,
    #[allow(dead_code)]
    #[error("Unknown resource")]
    UnknownResourceException,
    #[error("Could not create RDFox server")]
    CouldNotCreateRDFoxServer,
    #[error("Could not connect to RDFox server")]
    CouldNotConnectToServer,
    #[error("Could not import RDF File")]
    CouldNotImportRDFFile,
    #[error("Invalid prefix name")]
    InvalidPrefixName,
    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    /// Represents all other cases of `ignore::Error`
    /// (see https://docs.rs/ignore/latest/ignore/enum.Error.html)
    #[error(transparent)]
    WalkError(#[from] ignore::Error),
    #[error(transparent)]
    IriParseError(#[from] iref::Error),
    #[error(transparent)]
    CApiError(#[from] std::ffi::NulError),
}

#[cfg(feature = "nom_support")]
impl<I: From<&'static str>> From<Error> for nom::Err<nom::error::Error<I>> {
    fn from(_: Error) -> Self {
        nom::Err::Error(nom::error::Error::new(
            "unknown rdfox error".into(),
            nom::error::ErrorKind::Fail,
        ))
    }
}
