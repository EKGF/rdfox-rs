// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{
        database_call,
        rdfox_api::{
            CDataStoreConnection,
            CDataStoreConnection_destroy,
            CDataStoreConnection_evaluateUpdate,
            CDataStoreConnection_getName,
            CDataStoreConnection_getUniqueID,
            CDataStoreConnection_importAxiomsFromTriples,
            CDataStoreConnection_importDataFromFile,
            CStatementResult,
            CUpdateType,
        },
        DataStore,
        FactDomain,
        Namespaces,
        Parameters,
        ServerConnection,
        Statement,
        Streamer,
        Transaction,
    },
    colored::Colorize,
    fancy_regex::Regex,
    ignore::{types::TypesBuilder, WalkBuilder},
    indoc::formatdoc,
    iref::Iri,
    mime::Mime,
    rdf_store_rs::{
        consts::{
            DEFAULT_BASE_IRI,
            DEFAULT_GRAPH_RDFOX,
            LOG_TARGET_DATABASE,
            LOG_TARGET_FILES,
            TEXT_TURTLE,
        },
        Graph,
        Namespace,
        RDFStoreError,
    },
    std::{
        ffi::{CStr, CString},
        fmt::{Debug, Display, Formatter},
        io::Write,
        mem::MaybeUninit,
        ops::Deref,
        os::unix::ffi::OsStrExt,
        path::Path,
        ptr::{self, null_mut},
        sync::Arc,
        time::Instant,
    },
};

/// A connection to a given [`DataStore`].
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
        write!(
            f,
            "connection #{} to {}",
            self.number, self.data_store
        )
    }
}

impl Drop for DataStoreConnection {
    fn drop(&mut self) {
        assert!(
            !self.inner.is_null(),
            "Could not drop datastore connection #{}",
            self.number
        );

        let duration = self.started_at.elapsed();

        let self_msg = format!("{self}");
        unsafe {
            CDataStoreConnection_destroy(self.inner.cast());
        }
        self.inner = null_mut();
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            duration = ?duration,
            "Dropped {self_msg}",
        );
    }
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

    pub fn get_id(&self) -> Result<String, RDFStoreError> {
        assert!(
            !self.inner.is_null(),
            "invalid datastore connection"
        );
        let mut name: *const std::os::raw::c_char = ptr::null();
        database_call!(
            "getting the name of a datastore connection",
            CDataStoreConnection_getName(self.inner, &mut name)
        )?;
        let c_str = unsafe { CStr::from_ptr(name) };
        Ok(c_str.to_str().unwrap().into())
    }

    pub fn get_unique_id(&self) -> Result<String, RDFStoreError> {
        assert!(
            !self.inner.is_null(),
            "invalid datastore connection"
        );
        let mut unique_id: *const std::os::raw::c_char = ptr::null();
        database_call!(
            "Getting the unique id of datastore connection",
            CDataStoreConnection_getUniqueID(self.inner, &mut unique_id)
        )?;
        let c_str = unsafe { CStr::from_ptr(unique_id) };
        Ok(c_str.to_str().unwrap().into())
    }

    /// Import RDF data from the given file into the given graph.
    ///
    /// NOTE: Only supports turtle files at the moment.
    pub fn import_data_from_file<P>(&self, file: P, graph: &Graph) -> Result<(), RDFStoreError>
    where P: AsRef<Path> {
        assert!(
            !self.inner.is_null(),
            "invalid datastore connection"
        );

        let rdf_file = file.as_ref().as_os_str().as_bytes();
        tracing::trace!(
            target: LOG_TARGET_DATABASE,
            conn = self.number,
            "Importing file {} into {:} of {:}",
            file.as_ref().display(),
            graph,
            self
        );

        let c_graph_name = graph.as_c_string()?;
        let file_name = CString::new(rdf_file).unwrap();
        let format_name = CString::new(TEXT_TURTLE.as_ref()).unwrap();

        database_call!(
            format!("Importing data from {file_name:?} (format={format_name:?})").as_str(),
            CDataStoreConnection_importDataFromFile(
                self.inner,
                c_graph_name.as_ptr() as *const std::os::raw::c_char,
                CUpdateType::UPDATE_TYPE_ADDITION,
                file_name.as_ptr() as *const std::os::raw::c_char,
                format_name.as_ptr() as *const std::os::raw::c_char,
            )
        )?;
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            conn = self.number,
            "Imported file {} into {:}",
            file.as_ref().display(),
            graph
        );
        Ok(())
    }

    pub fn import_axioms_from_triples(
        &self,
        source_graph: &Graph,
        target_graph: &Graph,
    ) -> Result<(), RDFStoreError> {
        assert!(
            !self.inner.is_null(),
            "invalid datastore connection"
        );

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
            target: LOG_TARGET_DATABASE,
            conn = self.number,
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
    pub fn import_rdf_from_directory(
        &self,
        root: &Path,
        graph: &Graph,
    ) -> Result<u16, RDFStoreError> {
        let mut count = 0u16;
        let regex = Regex::new(r"^.*.ttl$").unwrap();

        tracing::debug!(
            target: LOG_TARGET_FILES,
            "Read all RDF files from directory {}",
            format!("{:?}", &root).green()
        );
        tracing::debug!(
            target: LOG_TARGET_FILES,
            "WalkBuilder::new({:?}), searching for {:?}",
            root,
            regex
        );

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
                    tracing::error!(target: LOG_TARGET_FILES, "error {:?}", error);
                    return Err(RDFStoreError::WalkError(error))
                },
            }
        }
        Ok(count)
    }

    // noinspection DuplicatedCode
    pub fn evaluate_update(
        &self,
        statement: &Statement,
        parameters: &Parameters,
    ) -> Result<CStatementResult, RDFStoreError> {
        assert!(
            !self.inner.is_null(),
            "invalid datastore connection"
        );
        // let c_base_iri = if let Some(base_iri) = base_iri {
        //     CString::new(base_iri.as_str()).unwrap()
        // } else {
        //     CString::new(DEFAULT_BASE_IRI).unwrap()
        // };
        let statement_text = statement.as_c_string()?;
        let statement_text_len = statement_text.as_bytes().len();
        let mut statement_result = MaybeUninit::uninit();
        database_call!(
            "evaluating an update statement",
            CDataStoreConnection_evaluateUpdate(
                self.inner,
                statement_text.as_ptr(),
                statement_text_len,
                parameters.inner.as_ref().cast_const(),
                statement_result.as_mut_ptr(),
            )
        )?;
        let statement_result = unsafe { statement_result.assume_init() };
        tracing::trace!("Evaluated update statement: {statement_result:?}",);
        Ok(statement_result)
    }

    pub fn evaluate_to_stream<'a, W>(
        self: &Arc<Self>,
        writer: W,
        statement: &'a Statement,
        mime_type: &'static Mime,
        base_iri: Option<Iri>,
    ) -> Result<Streamer<'a, W>, RDFStoreError>
    where
        W: 'a + Write,
    {
        Streamer::run(
            self,
            writer,
            statement,
            mime_type,
            Namespace::declare_from_str(
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
    ) -> Result<usize, RDFStoreError> {
        let default_graph = DEFAULT_GRAPH_RDFOX.deref().as_display_iri();
        Statement::new(
            &Namespaces::empty()?,
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
        .cursor(
            self,
            &Parameters::empty()?.fact_domain(fact_domain)?,
        )?
        .count(tx)
    }

    pub fn get_subjects_count(
        self: &Arc<Self>,
        tx: &Arc<Transaction>,
        fact_domain: FactDomain,
    ) -> Result<usize, RDFStoreError> {
        let default_graph = DEFAULT_GRAPH_RDFOX.deref().as_display_iri();
        Statement::new(
            &Namespaces::empty()?,
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
        .cursor(
            self,
            &Parameters::empty()?.fact_domain(fact_domain)?,
        )?
        .count(tx)
    }

    pub fn get_predicates_count(
        self: &Arc<Self>,
        tx: &Arc<Transaction>,
        fact_domain: FactDomain,
    ) -> Result<usize, RDFStoreError> {
        let default_graph = DEFAULT_GRAPH_RDFOX.deref().as_display_iri();
        Statement::new(
            &Namespaces::empty()?,
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
        .cursor(
            self,
            &Parameters::empty()?.fact_domain(fact_domain)?,
        )?
        .count(tx)
    }

    pub fn get_ontologies_count(
        self: &Arc<Self>,
        tx: &Arc<Transaction>,
        fact_domain: FactDomain,
    ) -> Result<usize, RDFStoreError> {
        let default_graph = DEFAULT_GRAPH_RDFOX.deref().as_display_iri();
        Statement::new(
            &Namespaces::empty()?,
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
        .cursor(
            self,
            &Parameters::empty()?.fact_domain(fact_domain)?,
        )?
        .count(tx)
    }
}
