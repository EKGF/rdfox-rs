// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

extern crate alloc;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Unknown Error")]
    Unknown,
    #[error("Unknown data type {datatype_id}")]
    UnknownDatatype {
        datatype_id: u8
    },
    #[allow(dead_code)]
    #[error("Unknown resource")]
    UnknownResourceException,
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
    CApiError(#[from] std::ffi::NulError)
}
