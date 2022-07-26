// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
#![feature(rustc_private)]

extern crate core;

use core::str::FromStr;

use lazy_static::lazy_static;
pub use mime::Mime;

pub use cursor::Cursor;
pub use data_store::DataStore;
pub use data_store_connection::DataStoreConnection;
pub use error::Error;
pub use graph::Graph;
pub use graph_connection::GraphConnection;
pub use parameters::{FactDomain, Parameters};
pub use prefixes::{Class, Prefixes, Prefix, PrefixesBuilder};
pub use role_creds::RoleCreds;
pub use server::Server;
pub use server_connection::ServerConnection;
pub use statement::Statement;
pub use transaction::Transaction;

lazy_static! {
    pub static ref TEXT_TURTLE: Mime = Mime::from_str("text/turtle").unwrap();
}

mod cursor;
mod data_store;
mod data_store_connection;
mod error;
mod exception;
mod graph;
mod parameters;
mod prefixes;
mod role_creds;
mod server;
mod server_connection;
mod statement;
mod transaction;
mod graph_connection;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
