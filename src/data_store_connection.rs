// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::{CStr, CString};
use std::fmt::{Display, Formatter};
use std::os::unix::ffi::OsStrExt;
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::ptr;
use std::time::Instant;

use colored::Colorize;
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use regex::Regex;
use indoc::indoc;

use crate::error::Error;
use crate::{
    root::{
        CDataStoreConnection, CDataStoreConnection_getID, CDataStoreConnection_getUniqueID,
        CDataStoreConnection_importAxiomsFromTriples,
        CDataStoreConnection_importDataFromFile, CException, CUpdateType,
    },
    DataStore, Graph, Parameters, FactDomain, Prefixes, Statement, TEXT_TURTLE,
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

        let c_graph_name = graph.as_c_string()?;
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
        log::debug!(
            "Imported file {} into graph {:}",
            file.as_ref().display(),
            graph
        );
        Ok(())
    }

    /// CRDFOX const CException* CDataStoreConnection_importAxiomsFromTriples (
    ///     CDataStoreConnection* dataStoreConnection,
    ///     const char* sourceGraphName,
    ///     bool translateAssertions,
    ///     const char* destinationGraphName,
    ///     CUpdateType updateType
    /// );
    pub fn import_axioms_from_triples(
        &self,
        source_graph: &Graph,
        target_graph: &Graph,
    ) -> Result<(), Error> {
        assert!(!self.inner.is_null(), "invalid datastore connection");

        let c_source_graph_name = source_graph.as_c_string()?;
        let c_target_graph_name = target_graph.as_c_string()?;

        CException::handle(|| unsafe {
            CDataStoreConnection_importAxiomsFromTriples(
                self.inner,
                c_source_graph_name.as_ptr() as *const std::os::raw::c_char,
                false,
                c_target_graph_name.as_ptr() as *const std::os::raw::c_char,
                CUpdateType::UPDATE_TYPE_ADDITION,
            )
        })?;
        log::debug!(
            "Imported axioms from {:} into graph {:}",
            source_graph,
            target_graph
        );
        Ok(())
    }

    /// Read all RDF files (currently it supports .ttl and .nt files) from
    /// the given directory, applying ignore files like `.gitignore`.
    ///
    /// TODO: Support all the types that RDFox supports (and more)
    /// TODO: Support '*.gz' files
    /// TODO: Parallelize appropriately in sync with number of threads that RDFox uses
    pub fn import_rdf_from_directory(&self, root: &Path, graph: &Graph) -> Result<u16, Error> {
        let mut count = 0u16;
        let regex = Regex::new(r"^.*.ttl$").unwrap();

        log::debug!(
            "Read all RDF files from directory {}",
            format!("{:?}", &root).green()
        );
        log::debug!("WalkBuilder::new({:?}), searching for {:?}", root, regex);

        let mut builder = TypesBuilder::new();
        builder.add("rdf", "*.nt").unwrap();
        builder.add("rdf", "*.ttl").unwrap();
        let file_types = builder.select("rdf").build().unwrap();

        let iter = WalkBuilder::new(root)
            .standard_filters(true)
            .ignore(false)
            .git_global(true)
            .git_ignore(true)
            .git_exclude(true)
            .follow_links(false)
            .parents(false)
            .threads(6)
            .types(file_types)
            .build();

        for rdf_file in iter {
            match rdf_file {
                Ok(dir_entry) => {
                    let file_type = dir_entry.file_type().unwrap();
                    if file_type.is_dir() {
                        continue;
                    }
                    let rdf_file = dir_entry.path();
                    // log::debug!("entry {:?}", dir_entry);
                    self.import_data_from_file(rdf_file, &graph)?;
                    count += 1;
                }
                Err(error) => {
                    log::error!("error {:?}", error);
                    return Err(Error::WalkError(error));
                }
            }
        }
        Ok(count)
    }

    pub fn get_triples_count(&self, fact_domain: FactDomain) -> Result<std::os::raw::c_ulong, Error> {
        Statement::query(
            &Prefixes::default()?,
            indoc! {r##"
                SELECT ?graph ?s ?p ?o
                WHERE {
                    {
                        GRAPH ?graph { ?s ?p ?o }
                    } UNION {
                        ?s ?p ?o .
                        BIND("default" AS ?graph)
                    }
                }
            "##},
        )?
            .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
            .count()
    }

    pub fn get_subjects_count(&self, fact_domain: FactDomain) -> Result<std::os::raw::c_ulong, Error> {
        Statement::query(
            &Prefixes::default()?,
            indoc! {r##"
                SELECT DISTINCT ?subject
                WHERE {
                    {
                        GRAPH ?graph {
                            ?subject ?p ?o
                        }
                    } UNION {
                        ?subject ?p ?o .
                        BIND("default" AS ?graph)
                    }
                }
            "##},
        )?
            .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
            .count()
    }

    pub fn get_predicates_count(&self, fact_domain: FactDomain) -> Result<std::os::raw::c_ulong, Error> {
        Statement::query(
            &Prefixes::default()?,
            indoc! {r##"
                SELECT DISTINCT ?predicate
                WHERE {
                    {
                        GRAPH ?graph {
                            ?s ?predicate ?o
                        }
                    } UNION {
                        ?s ?predicate ?o .
                        BIND("default" AS ?graph)
                    }
                }
            "##},
        )?
            .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
            .count()
    }

    pub fn get_ontologies_count(&self, fact_domain: FactDomain) -> Result<std::os::raw::c_ulong, Error> {
        Statement::query(
            &Prefixes::default()?,
            indoc! {r##"
                SELECT DISTINCT ?ontology
                WHERE {
                    {
                        GRAPH ?graph {
                            ?ontology a <http://www.w3.org/2002/07/owl#Ontology>
                        }
                    } UNION {
                            ?ontology a <http://www.w3.org/2002/07/owl#Ontology>
                        BIND("default" AS ?graph)
                    }
                }
            "##},
        )?
            .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
            .count()
    }
}
