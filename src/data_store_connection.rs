// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::ptr;

use crate::{
    root::{
        CDataStoreConnection, CDataStoreConnection_getID, CDataStoreConnection_getUniqueID,
        CDataStoreConnection_importDataFromFile, CException, CUpdateType,
    },
    Error, Graph, Parameters, Prefixes, Statement, Transaction, TEXT_TURTLE,
};

pub struct DataStoreConnection {
    pub(crate) inner: *mut CDataStoreConnection,
}

impl Display for DataStoreConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("data store connection").unwrap();
        match self.get_id() {
            Ok(id) => write!(f, " id={}", id),
            Err(error) => write!(f, " id=({:?})", error),
        }
        .unwrap();
        match self.get_unique_id() {
            Ok(id) => write!(f, " unique-id={}", id),
            Err(_error) => write!(f, " unique-id=(error could not get id)"),
        }
    }
}

impl DataStoreConnection {
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

    pub fn import_data_from_file<P>(&mut self, file: P, graph: &Graph) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        log::debug!(
            "Importing file {} into graph {:} of {:}",
            file.as_ref().display(),
            graph,
            self
        );

        let c_graph_name = graph.as_c_string();
        let prefixes = Prefixes::default()?;
        let file_name = CString::new(file.as_ref().file_name().unwrap().to_str().unwrap()).unwrap();
        let format_name = CString::new(TEXT_TURTLE.as_ref()).unwrap();

        assert!(!self.inner.is_null());

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
        // getTriplesCount(dataStoreConnection, "all", emptyPrefixes)
        // static size_t getTriplesCount(CDataStoreConnection* dataStoreConnection, const char* queryDomain, CPrefixes* prefixes) {
        //     CParameters* parameters = NULL;
        //     CParameters_newEmptyParameters(&parameters);
        //     CParameters_setString(parameters, "fact-domain", queryDomain);
        //
        //     CCursor* cursor = NULL;
        //     CDataStoreConnection_createCursor(dataStoreConnection, NULL, prefixes, "SELECT ?X ?Y ?Z WHERE { ?X ?Y ?Z }", 34, parameters, &cursor);
        //     CParameters_destroy(parameters);
        //     CDataStoreConnection_beginTransaction(dataStoreConnection, TRANSACTION_TYPE_READ_ONLY);
        //     size_t result = 0;
        //     size_t multiplicity;
        //     for (CCursor_open(cursor, &multiplicity); multiplicity != 0; CCursor_advance(cursor, &multiplicity))
        //     result += multiplicity;
        //     CCursor_destroy(cursor);
        //     CDataStoreConnection_rollbackTransaction(dataStoreConnection);
        //     return result;
        // }
        let parameters = Parameters::empty()?;
        parameters.fact_domain_all()?;

        let prefixes = Prefixes::default()?;

        let cursor = Statement::query(
            &prefixes,
            "SELECT ?G ?X ?Y ?Z WHERE { GRAPH ?G { ?X ?Y ?Z }}",
        )?
        .cursor(&self, &parameters)?;

        Transaction::begin_read_only(self)?.execute_and_rollback(|| {
            let mut result = 0 as std::os::raw::c_ulong;
            let mut multiplicity = cursor.open()?;
            while multiplicity > 0 {
                multiplicity = cursor.advance()?;
                result += multiplicity;
            }
            Ok(result)
        })
    }
}
