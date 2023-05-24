// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
#![feature(rustc_private)]
#![feature(ptr_metadata)]
#![doc = include_str!("../README.md")]

extern crate core;

pub use {
    class_report::ClassReport,
    connectable_data_store::ConnectableDataStore,
    cursor::{Cursor, CursorRow, OpenedCursor},
    data_store::DataStore,
    data_store_connection::DataStoreConnection,
    graph_connection::GraphConnection,
    license::{find_license, RDFOX_DEFAULT_LICENSE_FILE_NAME, RDFOX_HOME},
    mime::Mime,
    namespaces::{Namespaces, NamespacesBuilder},
    parameters::{FactDomain, Parameters, PersistenceMode},
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
mod namespaces;
mod parameters;
mod role_creds;
mod server;
mod server_connection;
mod statement;
mod streamer;
mod transaction;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod rdfox_api {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
