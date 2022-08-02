// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
#![feature(rustc_private)]
#![feature(cstr_from_bytes_until_nul)]

extern crate core;

use core::str::FromStr;

use lazy_static::lazy_static;
pub use mime::Mime;

pub use class::Class;
pub use cursor::{Cursor,OpenedCursor,CursorRow,ResourceValue};
pub use data_store::DataStore;
pub use data_store_connection::DataStoreConnection;
pub use data_type::DataType;
pub use lexical_value::LexicalValue;
pub use error::Error;
pub use graph::{DEFAULT_GRAPH, Graph, NS_RDFOX};
pub use graph_connection::GraphConnection;
pub use parameters::{FactDomain, Parameters};
pub use prefixes::{Prefix, Prefixes, PrefixesBuilder};
pub use role_creds::RoleCreds;
pub use server::Server;
pub use server_connection::ServerConnection;
pub use statement::Statement;
pub use transaction::Transaction;

lazy_static! {
    pub static ref TEXT_TURTLE: Mime = Mime::from_str("text/turtle").unwrap();
    pub static ref TEXT_OWL_FUNCTIONAL: Mime = Mime::from_str("text/owl-functional").unwrap();
    pub static ref APPLICATION_N_TRIPLES: Mime = Mime::from_str("application/n-triples").unwrap();
    pub static ref APPLICATION_N_QUADS: Mime = Mime::from_str("application/n-quads").unwrap();
    pub static ref APPLICATION_TRIG: Mime = Mime::from_str("application/trig").unwrap();
    pub static ref APPLICATION_X_DATALOG: Mime = Mime::from_str("application/x.datalog").unwrap();
}

mod cursor;
mod data_store;
mod data_store_connection;
mod error;
mod exception;
mod graph;
mod graph_connection;
mod parameters;
mod prefixes;
mod role_creds;
mod server;
mod server_connection;
mod statement;
mod transaction;
mod class;
mod data_type;
mod lexical_value;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
