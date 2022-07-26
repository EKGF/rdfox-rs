// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::fmt::{Display, Formatter};
use std::path::Path;
use std::time::Instant;

use indoc::formatdoc;

use crate::{
    DataStoreConnection, FactDomain, Graph, Parameters, Prefixes, Statement,
};
use crate::error::Error;

pub struct GraphConnection<'a> {
    pub data_store_connection: &'a DataStoreConnection,
    started_at: Instant,
    pub graph: Graph,
    pub ontology_graph: Option<Graph>
}

impl<'a> Display for GraphConnection<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "connection to {:}", self.graph)
    }
}

impl<'a> Drop for GraphConnection<'a> {
    fn drop(&mut self) {
        let duration = self.started_at.elapsed();
        log::info!("dropped {self} after {:?}", duration)
    }
}

impl<'a> GraphConnection<'a> {
    pub fn new(
        data_store_connection: &'a DataStoreConnection,
        graph: Graph,
        ontology_graph: Option<Graph>
    ) -> Self {
        Self {
            data_store_connection,
            started_at: Instant::now(),
            graph,
            ontology_graph
        }
    }

    pub fn import_data_from_file<P>(&self, file: P) -> Result<(), Error>
        where
            P: AsRef<Path>,
    {
        self.data_store_connection.import_data_from_file(file, &self.graph)
    }

    pub fn import_axioms(
        &self,
    ) -> Result<(), Error> {
        assert!(self.ontology_graph.is_some(),"no ontology graph specified");
        self.data_store_connection.import_axioms_from_triples(
            self.ontology_graph.as_ref().unwrap(),
            &self.graph
        )
    }

    /// Read all RDF files (currently it supports .ttl and .nt files) from
    /// the given directory, applying ignore files like `.gitignore`.
    ///
    /// TODO: Support all the types that RDFox supports (and more)
    /// TODO: Support '*.gz' files
    /// TODO: Parallelize appropriately in sync with number of threads that RDFox uses
    pub fn import_rdf_from_directory(&self, root: &Path) -> Result<u16, Error> {
        self.data_store_connection.import_rdf_from_directory(root, &self.graph)
    }

    pub fn get_triples_count(&self, fact_domain: FactDomain) -> Result<std::os::raw::c_ulong, Error> {
        Statement::query(
            &Prefixes::default()?,
            formatdoc! (r##"
                SELECT ?s ?p ?o
                FROM {:}
                WHERE {{
                    ?s ?p ?o .
                }}
            "##, self.graph
            ).as_str(),
        )?
            .cursor(&self.data_store_connection, &Parameters::empty()?.fact_domain(fact_domain)?)?
            .count()
    }

    // pub fn get_subjects_count(&self, fact_domain: FactDomain) -> Result<std::os::raw::c_ulong, Error> {
    //     Statement::query(
    //         &Prefixes::default()?,
    //         indoc! {r##"
    //             SELECT DISTINCT ?subject
    //             WHERE {
    //                 {
    //                     GRAPH ?graph {
    //                         ?subject ?p ?o
    //                     }
    //                 } UNION {
    //                     ?subject ?p ?o .
    //                     BIND("default" AS ?graph)
    //                 }
    //             }
    //         "##},
    //     )?
    //         .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
    //         .count()
    // }
    //
    // pub fn get_predicates_count(&self, fact_domain: FactDomain) -> Result<std::os::raw::c_ulong, Error> {
    //     Statement::query(
    //         &Prefixes::default()?,
    //         indoc! {r##"
    //             SELECT DISTINCT ?predicate
    //             WHERE {
    //                 {
    //                     GRAPH ?graph {
    //                         ?s ?predicate ?o
    //                     }
    //                 } UNION {
    //                     ?s ?predicate ?o .
    //                     BIND("default" AS ?graph)
    //                 }
    //             }
    //         "##},
    //     )?
    //         .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
    //         .count()
    // }
    //
    // pub fn get_ontologies_count(&self, fact_domain: FactDomain) -> Result<std::os::raw::c_ulong, Error> {
    //     Statement::query(
    //         &Prefixes::default()?,
    //         indoc! {r##"
    //             SELECT DISTINCT ?ontology
    //             WHERE {
    //                 {
    //                     GRAPH ?graph {
    //                         ?ontology a <http://www.w3.org/2002/07/owl#Ontology>
    //                     }
    //                 } UNION {
    //                         ?ontology a <http://www.w3.org/2002/07/owl#Ontology>
    //                     BIND("default" AS ?graph)
    //                 }
    //             }
    //         "##},
    //     )?
    //         .cursor(self, &Parameters::empty()?.fact_domain(fact_domain)?)?
    //         .count()
    // }
}
