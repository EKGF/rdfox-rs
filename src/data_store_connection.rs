// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::os::unix::ffi::OsStrExt;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::ptr;
use std::time::Instant;

use crate::{
    DataStore,
    Error, Graph, Parameters, Prefixes, root::{
        CDataStoreConnection, CDataStoreConnection_getID, CDataStoreConnection_getUniqueID,
        CDataStoreConnection_importDataFromFile, CException, CUpdateType,
    }, Statement, TEXT_TURTLE,
};

pub struct DataStoreConnection {
    pub data_store: DataStore,
    pub(crate) inner: *mut CDataStoreConnection,
    started_at: Instant,
}

impl Display for DataStoreConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("connection").unwrap();
        // match self.get_id() {
        //     Ok(id) => write!(f, " id={}", id),
        //     Err(error) => write!(f, " id=({:?})", error)
        // }.unwrap();
        match self.get_unique_id() {
            Ok(id) => write!(f, " {}", id),
            Err(_error) => write!(f, " (error could not get unique-id)"),
        }
            .unwrap();
        write!(f, " to {}", self.data_store)
    }
}

impl Drop for DataStoreConnection {
    fn drop(&mut self) {
        let duration = self.started_at.elapsed();
        log::info!("dropped {self} after {:?}", duration)
    }
}

impl DataStoreConnection {
    pub(crate) fn new(data_store: DataStore, inner: *mut CDataStoreConnection) -> Self {
        Self {
            data_store,
            inner,
            started_at: Instant::now(),
        }
    }

    pub fn get_id(&self) -> Result<u32, Error> {
        let mut id: u32 = 0;
        CException::handle(AssertUnwindSafe(|| unsafe {
            CDataStoreConnection_getID(self.inner, &mut id)
        }))?;
        Ok(id)
    }

    pub fn get_unique_id(&self) -> Result<String, Error> {
        let mut unique_id: *const std::os::raw::c_char = ptr::null();
        CException::handle(AssertUnwindSafe(|| unsafe {
            CDataStoreConnection_getUniqueID(self.inner, &mut unique_id)
        }))?;
        let c_str = unsafe { CStr::from_ptr(unique_id) };
        Ok(c_str.to_str().unwrap().into())
    }

    pub fn import_data_from_file<P>(&self, file: P, graph: &Graph) -> Result<(), Error>
        where
            P: AsRef<Path>,
    {
        assert!(!self.inner.is_null(), "invalid datastore connection");

        let rdf_file = file.as_ref().as_os_str().as_bytes();
        log::trace!(
            "Importing file {} into graph {:} of {:}",
            file.as_ref().display(),
            graph,
            self
        );

        let c_graph_name = graph.as_c_string();
        let prefixes = Prefixes::default()?;
        let file_name = CString::new(rdf_file).unwrap();
        let format_name = CString::new(TEXT_TURTLE.as_ref()).unwrap();

        CException::handle(|| unsafe {
            CDataStoreConnection_importDataFromFile(
                self.inner,
                c_graph_name.as_ptr() as *const std::os::raw::c_char,
                CUpdateType::UPDATE_TYPE_ADDITION,
                prefixes.inner,
                file_name.as_ptr() as *const std::os::raw::c_char,
                format_name.as_ptr() as *const std::os::raw::c_char,
            )
        })?;
        log::info!(
            "Imported file {} into graph {:}",
            file.as_ref().display(),
            graph
        );
        Ok(())
    }

    pub fn get_triples_count(&self) -> Result<std::os::raw::c_ulong, Error> {
        Statement::query(
            &Prefixes::default()?,
            "SELECT ?G ?X ?Y ?Z WHERE { GRAPH ?G { ?X ?Y ?Z }}",
        )?
            .cursor(self, &Parameters::empty()?.fact_domain_all()?)?
            .count()
    }

    pub fn get_subjects_count(&self) -> Result<std::os::raw::c_ulong, Error> {
        Statement::query(
            &Prefixes::default()?,
            "SELECT DISTINCT ?subject WHERE { GRAPH ?G { ?subject ?Y ?Z }}",
        )?
            .cursor(self, &Parameters::empty()?.fact_domain_all()?)?
            .count()
    }

    pub fn get_predicates_count(&self) -> Result<std::os::raw::c_ulong, Error> {
        Statement::query(
            &Prefixes::default()?,
            "SELECT DISTINCT ?predicate WHERE { GRAPH ?G { ?X ?predicate ?Z }}",
        )?
            .cursor(self, &Parameters::empty()?.fact_domain_all()?)?
            .count()
    }
}
