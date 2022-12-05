// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{
    ffi::{CStr, CString},
    fmt::{Debug, Display, Formatter},
    io::Write,
    ops::Deref,
    os::unix::ffi::OsStrExt,
    path::Path,
    ptr::{self, null_mut},
    sync::Arc,
    time::Instant,
};

use colored::Colorize;
use ignore::{types::TypesBuilder, WalkBuilder};
use indoc::formatdoc;
use iref::Iri;
use mime::Mime;
use regex::Regex;

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
    Statement,
    Streamer,
    Transaction,
    DEFAULT_BASE_IRI,
    DEFAULT_GRAPH,
    LOG_TARGET_DATABASE,
    TEXT_TURTLE,
};
use crate::{error::Error, ServerConnection};

#[derive(Debug)]
pub struct DataStoreConnection {
    pub data_store:        Arc<DataStore>,
    pub server_connection: Arc<ServerConnection>,
    pub(crate) inner:      *mut CDataStoreConnection,
    started_at:            Instant,
    pub number:            usize,
}

unsafe impl Sync for DataStoreConnection {}

unsafe impl Send for DataStoreConnection {}

impl Display for DataStoreConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "connection #{} to {}", self.number, self.data_store)
    }
}

impl Drop for DataStoreConnection {
    fn drop(&mut self) { self.destroy() }
}

impl DataStoreConnection {
    pub(crate) fn new(
        server_connection: &Arc<ServerConnection>,
        data_store: &Arc<DataStore>,
        inner: *mut CDataStoreConnection,
    ) -> Self {
        Self {
            data_store: data_store.clone(),
            server_connection: server_connection.clone(),
            inner,
            started_at: Instant::now(),
            number: Self::get_number(),
        }
    }

    pub fn same(self: &Arc<Self>, other: &Arc<Self>) -> bool { self.number == other.number }

    fn get_number() -> usize {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
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
        tracing::trace!(
            "Importing file {} into {:} of {:}",
            file.as_ref().display(),
            graph,
            self
        );

        let c_graph_name = graph.as_c_string()?;
        let prefixes = Prefixes::empty()?;
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
        tracing::debug!("Imported file {} into {:}", file.as_ref().display(), graph);
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
        tracing::debug!(
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

        tracing::debug!(
            "Read all RDF files from directory {}",
            format!("{:?}", &root).green()
        );
        tracing::debug!("WalkBuilder::new({:?}), searching for {:?}", root, regex);

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
                    // tracing::debug!("entry {:?}", dir_entry);
                    self.import_data_from_file(rdf_file, graph)?;
                    count += 1;
                },
                Err(error) => {
                    tracing::error!("error {:?}", error);
                    return Err(Error::WalkError(error))
                },
            }
        }
        Ok(count)
    }

    // noinspection DuplicatedCode
    pub fn evaluate_update<'b>(
        &self,
        statement: &'b Statement,
        parameters: &Parameters,
        base_iri: Option<Iri>,
    ) -> Result<(), Error> {
        assert!(!self.inner.is_null(), "invalid datastore connection");
        let c_base_iri = if let Some(base_iri) = base_iri {
            CString::new(base_iri.as_str()).unwrap()
        } else {
            CString::new(DEFAULT_BASE_IRI).unwrap()
        };
        let statement_text = statement.as_c_string()?;
        let statement_text_len = statement_text.as_bytes().len();
        let mut statement_result: CStatementResult = Default::default();
        database_call!(
            "evaluating an update statement",
            CDataStoreConnection_evaluateUpdate(
                self.inner,
                c_base_iri.as_ptr(),
                statement.prefixes.inner,
                statement_text.as_ptr(),
                statement_text_len,
                parameters.inner,
                statement_result.as_mut_ptr(),
            )
        )?;
        tracing::error!("evaluated update statement: {:?}", statement_result);
        Ok(())
    }

    pub fn evaluate_to_stream<'a, W>(
        self: &Arc<Self>,
        writer: W,
        statement: &'a Statement,
        mime_type: &'static Mime,
        base_iri: Option<Iri>,
    ) -> Result<Streamer<'a, W>, Error>
    where
        W: 'a + Write,
    {
        Streamer::run(
            self,
            writer,
            statement,
            mime_type,
            Prefix::declare_from_str(
                "base",
                base_iri
                    .as_ref()
                    .map(|iri| iri.as_str())
                    .unwrap_or_else(|| DEFAULT_BASE_IRI.deref()),
            ),
        )
    }

    pub fn get_triples_count(
        self: &Arc<Self>,
        tx: &Arc<Transaction>,
        fact_domain: FactDomain,
    ) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        Statement::new(
            Prefixes::empty()?,
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
            .into(),
        )?
        .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?, None)?
        .count(tx)
    }

    pub fn get_subjects_count(
        self: &Arc<Self>,
        tx: &Arc<Transaction>,
        fact_domain: FactDomain,
    ) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        Statement::new(
            Prefixes::empty()?,
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
            .into(),
        )?
        .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?, None)?
        .count(tx)
    }

    pub fn get_predicates_count(
        self: &Arc<Self>,
        tx: &Arc<Transaction>,
        fact_domain: FactDomain,
    ) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        Statement::new(
            Prefixes::empty()?,
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
            .into(),
        )?
        .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?, None)?
        .count(tx)
    }

    pub fn get_ontologies_count(
        self: &Arc<Self>,
        tx: &Arc<Transaction>,
        fact_domain: FactDomain,
    ) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        Statement::new(
            Prefixes::empty()?,
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
            .into(),
        )?
        .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?, None)?
        .count(tx)
    }

    // noinspection RsUnreachableCode
    fn destroy(&mut self) {
        assert!(!self.inner.is_null(), "invalid datastore connection");

        let duration = self.started_at.elapsed();

        let self_msg = format!("{self}");
        unsafe {
            CDataStoreConnection_destroy(self.inner);
        }
        self.inner = null_mut();
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "Destroyed {self_msg} after {:?}",
            duration
        );
    }
}
