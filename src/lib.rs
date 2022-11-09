// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
#![feature(rustc_private)]
#![feature(cstr_from_bytes_until_nul)]
#![feature(ptr_metadata)]

extern crate core;

use core::str::FromStr;

pub use class::Class;
pub use cursor::{Cursor, CursorRow, OpenedCursor};
pub use data_store::DataStore;
pub use data_store_connection::DataStoreConnection;
pub use error::Error;
pub use exception::CException;
pub use graph::{Graph, DEFAULT_GRAPH, NS_RDFOX};
pub use graph_connection::GraphConnection;
use lazy_static::lazy_static;
pub use license::{find_license, RDFOX_DEFAULT_LICENSE_FILE_NAME, RDFOX_HOME};
pub use mime::Mime;
pub use namespace::*;
pub use parameters::{FactDomain, Parameters, PersistenceMode};
pub use predicate::Predicate;
pub use prefixes::{Prefix, Prefixes, PrefixesBuilder};
pub use rdf::{DataType, LexicalValue, ResourceValue, Term};
pub use role_creds::RoleCreds;
pub use server::Server;
pub use server_connection::ServerConnection;
pub use statement::Statement;
pub use streamer::Streamer;
pub use transaction::Transaction;

pub const LOG_TARGET_CONFIG: &str = "config";
pub const LOG_TARGET_SPARQL: &str = "sparql";
pub const LOG_TARGET_FILES: &str = "files";
pub const LOG_TARGET_DATABASE: &str = "database";

// All supported MIME types
lazy_static! {
    // As documented here: https://docs.oxfordsemantic.tech/5.6/programmatic-access-APIs.html#formats-encoding-sparql-query-results
    pub static ref TEXT_TSV: Mime = Mime::from_str("text/tab-separated-values").unwrap();
    pub static ref TEXT_CSV: Mime = Mime::from_str("text/csv").unwrap();
    pub static ref TEXT_X_CSV_ABBREV: Mime = Mime::from_str("text/x.csv-abbrev").unwrap();
    pub static ref TEXT_TURTLE: Mime = Mime::from_str("text/turtle").unwrap();
    pub static ref TEXT_OWL_FUNCTIONAL: Mime = Mime::from_str("text/owl-functional").unwrap();
    pub static ref TEXT_X_TAB_SEPARATED_VALUES_ABBREV: Mime =
        Mime::from_str("text/x.tab-separated-values-abbrev").unwrap();
    pub static ref APPLICATION_TRIG: Mime = Mime::from_str("application/trig").unwrap();
    pub static ref APPLICATION_N_QUADS: Mime = Mime::from_str("application/n-quads").unwrap();
    pub static ref APPLICATION_N_TRIPLES: Mime = Mime::from_str("application/n-triples").unwrap();
    pub static ref APPLICATION_X_DATALOG: Mime = Mime::from_str("application/x.datalog").unwrap();
    pub static ref APPLICATION_SPARQL_RESULTS_XML: Mime =
        Mime::from_str("application/sparql-results+xml").unwrap();
    pub static ref APPLICATION_SPARQL_RESULTS_JSON: Mime =
        Mime::from_str("application/sparql-results+json").unwrap();
    pub static ref APPLICATION_SPARQL_RESULTS_TURTLE: Mime =
        Mime::from_str("application/sparql-results+turtle").unwrap();
    pub static ref APPLICATION_X_SPARQL_RESULTS_XML_ABBREV: Mime =
        Mime::from_str("application/x.sparql-results+xml-abbrev").unwrap();
    pub static ref APPLICATION_X_SPARQL_RESULTS_JSON_ABBREV: Mime =
        Mime::from_str("application/x.sparql-results+json-abbrev").unwrap();
    pub static ref APPLICATION_X_SPARQL_RESULTS_TURTLE_ABBREV: Mime =
        Mime::from_str("application/x.sparql-results+turtle-abbrev").unwrap();
    pub static ref APPLICATION_X_SPARQL_RESULTS_RESOURCEID: Mime =
        Mime::from_str("application/x.sparql-results+resourceid").unwrap();
    pub static ref APPLICATION_X_SPARQL_RESULTS_NULL: Mime =
        Mime::from_str("application/x.sparql-results+null").unwrap();
}

mod class;
mod cursor;
mod data_store;
mod data_store_connection;
mod error;
mod exception;
mod graph;
mod graph_connection;
mod license;
mod namespace;
mod parameters;
mod predicate;
mod prefixes;
mod rdf;
mod role_creds;
mod server;
mod server_connection;
mod statement;
mod streamer;
mod transaction;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
