// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
#![feature(rustc_private)]
#![feature(cstr_from_bytes_until_nul)]
#![feature(ptr_metadata)]

extern crate core;

pub use {
    class_report::ClassReport,
    connectable_data_store::ConnectableDataStore,
    cursor::{Cursor, CursorRow, OpenedCursor},
    data_store::DataStore,
    data_store_connection::DataStoreConnection,
    exception::CException,
    graph_connection::GraphConnection,
    license::{find_license, RDFOX_DEFAULT_LICENSE_FILE_NAME, RDFOX_HOME},
    mime::Mime,
    parameters::{FactDomain, Parameters, PersistenceMode},
    prefixes::{Prefixes, PrefixesBuilder},
    rdf_store_rs::{DataType, LexicalValue, Predicate, Prefix, RDFStoreError, ResourceValue, Term},
    role_creds::RoleCreds,
    server::Server,
    server_connection::ServerConnection,
    statement::Statement,
    streamer::Streamer,
    transaction::Transaction,
};

mod class_report;
mod connectable_data_store;
mod cursor;
mod data_store;
mod data_store_connection;
mod exception;
mod graph_connection;
mod license;
mod parameters;
mod prefixes;
mod role_creds;
mod server;
mod server_connection;
mod statement;
mod streamer;
mod transaction;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
