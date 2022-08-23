// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{
    ffi::{CStr, CString},
    fmt::{Debug, Display, Formatter},
    io::Write,
    ops::Deref,
    os::unix::ffi::OsStrExt,
    path::Path,
    ptr,
    ptr::null_mut,
    time::Instant,
};

use colored::Colorize;
use ignore::{types::TypesBuilder, WalkBuilder};
use indoc::formatdoc;
use mime::Mime;
use regex::Regex;

use crate::error::Error;
use crate::{
    database_call,
    root::{
        CDataStoreConnection,
        CDataStoreConnection_destroy,
        CDataStoreConnection_evaluateUpdate,
        // CDataStoreConnection_evaluateStatementToBuffer,
        CDataStoreConnection_getID,
        CDataStoreConnection_getUniqueID,
        CDataStoreConnection_importAxiomsFromTriples,
        CDataStoreConnection_importDataFromFile,
        CStatementResult,
        // COutputStream
        CUpdateType,
    },
    DataStore,
    FactDomain,
    Graph,
    Parameters,
    Prefix,
    Prefixes,
    ServerConnection,
    Statement,
    Streamer,
    Transaction,
    DEFAULT_GRAPH,
    TEXT_TURTLE,
};

#[derive(Debug, PartialEq)]
pub struct DataStoreConnection<'a> {
    pub data_store:        &'a DataStore,
    pub server_connection: &'a ServerConnection<'a>,
    pub(crate) inner:      *mut CDataStoreConnection,
    started_at:            Instant,
}

impl<'a> Display for DataStoreConnection<'a> {
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

impl<'a> Drop for DataStoreConnection<'a> {
    fn drop(&mut self) { self.destroy() }
}

impl<'a> DataStoreConnection<'a> {
    pub(crate) fn new(
        server_connection: &'a ServerConnection<'a>,
        data_store: &'a DataStore,
        inner: *mut CDataStoreConnection,
    ) -> Self {
        Self {
            data_store,
            server_connection,
            inner,
            started_at: Instant::now(),
        }
    }

    pub fn get_id(&self) -> Result<u32, Error> {
        assert!(!self.inner.is_null(), "invalid datastore connection");
        let mut id: u32 = 0;
        database_call!(
            "getting the id of a datastore connection",
            CDataStoreConnection_getID(self.inner, &mut id)
        )?;
        Ok(id)
    }

    pub fn get_unique_id(&self) -> Result<String, Error> {
        assert!(!self.inner.is_null(), "invalid datastore connection");
        let mut unique_id: *const std::os::raw::c_char = ptr::null();
        database_call!(
            "Getting the unique id of datastore connection",
            CDataStoreConnection_getUniqueID(self.inner, &mut unique_id)
        )?;
        let c_str = unsafe { CStr::from_ptr(unique_id) };
        Ok(c_str.to_str().unwrap().into())
    }

    pub fn import_data_from_file<P>(&self, file: P, graph: &Graph) -> Result<(), Error>
    where P: AsRef<Path> {
        assert!(!self.inner.is_null(), "invalid datastore connection");

        let rdf_file = file.as_ref().as_os_str().as_bytes();
        log::trace!(
            "Importing file {} into {:} of {:}",
            file.as_ref().display(),
            graph,
            self
        );

        let c_graph_name = graph.as_c_string()?;
        let prefixes = Prefixes::default()?;
        let file_name = CString::new(rdf_file).unwrap();
        let format_name = CString::new(TEXT_TURTLE.as_ref()).unwrap();

        database_call!(
            "importing data from a file",
            CDataStoreConnection_importDataFromFile(
                self.inner,
                c_graph_name.as_ptr() as *const std::os::raw::c_char,
                CUpdateType::UPDATE_TYPE_ADDITION,
                prefixes.inner,
                file_name.as_ptr() as *const std::os::raw::c_char,
                format_name.as_ptr() as *const std::os::raw::c_char,
            )
        )?;
        log::debug!("Imported file {} into {:}", file.as_ref().display(), graph);
        Ok(())
    }

    pub fn import_axioms_from_triples(
        &self,
        source_graph: &Graph,
        target_graph: &Graph,
    ) -> Result<(), Error> {
        assert!(!self.inner.is_null(), "invalid datastore connection");

        let c_source_graph_name = source_graph.as_c_string()?;
        let c_target_graph_name = target_graph.as_c_string()?;

        database_call!(
            "importing axioms",
            CDataStoreConnection_importAxiomsFromTriples(
                self.inner,
                c_source_graph_name.as_ptr() as *const std::os::raw::c_char,
                false,
                c_target_graph_name.as_ptr() as *const std::os::raw::c_char,
                CUpdateType::UPDATE_TYPE_ADDITION,
            )
        )?;
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
    /// Returns the number of loaded files.
    ///
    /// TODO: Support all the types that RDFox supports (and more)
    /// TODO: Support '*.gz' files
    /// TODO: Parallelize appropriately in sync with number of threads that
    /// RDFox uses
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
                        continue
                    }
                    let rdf_file = dir_entry.path();
                    // log::debug!("entry {:?}", dir_entry);
                    self.import_data_from_file(rdf_file, &graph)?;
                    count += 1;
                },
                Err(error) => {
                    log::error!("error {:?}", error);
                    return Err(Error::WalkError(error))
                },
            }
        }
        Ok(count)
    }

    pub fn evaluate_update<'b>(
        &self,
        statement: &'b Statement<'b>,
        parameters: &Parameters,
    ) -> Result<(), Error> {
        assert!(!self.inner.is_null(), "invalid datastore connection");
        let base_iri = ptr::null_mut();
        let statement_text = statement.as_c_string()?;
        let statement_text_len: u64 = statement_text.as_bytes().len() as u64;
        let mut statement_result: CStatementResult = Default::default();
        database_call!(
            "evaluating an update statement",
            CDataStoreConnection_evaluateUpdate(
                self.inner,
                base_iri,
                statement.prefixes.inner,
                statement_text.as_ptr(),
                statement_text_len,
                parameters.inner,
                statement_result.as_mut_ptr(),
            )
        )?;
        log::debug!("evaluated update statement: {:?}", statement_result);
        Ok(())
    }

    pub fn evaluate_to_stream<W>(
        &'a self,
        writer: W,
        statement: &'a Statement<'a>,
        mime_type: &'static Mime,
    ) -> Result<Streamer<'a, W>, Error>
    where
        W: 'a + Write + Debug,
    {
        Streamer::run(
            self,
            writer,
            statement,
            mime_type,
            Prefix::declare_from_str("base", "https://whatever.kg"),
        )
    }

    pub fn get_triples_count(
        &self,
        tx: &Transaction,
        fact_domain: FactDomain,
    ) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        Statement::new(
            &Prefixes::default()?,
            formatdoc!(
                r##"
                SELECT ?graph ?s ?p ?o
                WHERE {{
                    {{
                        GRAPH ?graph {{ ?s ?p ?o }}
                    }} UNION {{
                        ?s ?p ?o .
                        BIND({default_graph} AS ?graph)
                    }}
                }}
            "##
            )
            .as_str(),
        )?
        .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
        .count_in_transaction(tx)
    }

    pub fn get_subjects_count(&self, fact_domain: FactDomain) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        Statement::new(
            &Prefixes::default()?,
            formatdoc!(
                r##"
                SELECT DISTINCT ?subject
                WHERE {{
                    {{
                        GRAPH ?graph {{
                            ?subject ?p ?o
                        }}
                    }} UNION {{
                        ?subject ?p ?o .
                        BIND({default_graph} AS ?graph)
                    }}
                }}
            "##
            )
            .as_str(),
        )?
        .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
        .count()
    }

    pub fn get_predicates_count(&self, fact_domain: FactDomain) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        Statement::new(
            &Prefixes::default()?,
            formatdoc!(
                r##"
                SELECT DISTINCT ?predicate
                WHERE {{
                    {{
                        GRAPH ?graph {{
                            ?s ?predicate ?o
                        }}
                    }} UNION {{
                        ?s ?predicate ?o .
                        BIND({default_graph} AS ?graph)
                    }}
                }}
            "##
            )
            .as_str(),
        )?
        .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
        .count()
    }

    pub fn get_ontologies_count(&self, fact_domain: FactDomain) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        Statement::new(
            &Prefixes::default()?,
            formatdoc!(
                r##"
                SELECT DISTINCT ?ontology
                WHERE {{
                    {{
                        GRAPH ?graph {{
                            ?ontology a <http://www.w3.org/2002/07/owl#Ontology>
                        }}
                    }} UNION {{
                        ?ontology a <http://www.w3.org/2002/07/owl#Ontology>
                        BIND({default_graph} AS ?graph)
                    }}
                }}
                "##
            )
            .as_str(),
        )?
        .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
        .count()
    }

    fn destroy(&mut self) {
        let duration = self.started_at.elapsed();

        assert!(!self.inner.is_null(), "invalid datastore connection");

        let self_msg = format!("{self}");

        unsafe {
            CDataStoreConnection_destroy(self.inner);
        }
        self.inner = null_mut();
        log::debug!("Destroyed {self_msg} after {:?}", duration);
    }
}
