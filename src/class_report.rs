// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{FactDomain, GraphConnection, Namespaces, Parameters, Statement, Transaction},
    indoc::formatdoc,
    rdf_store_rs::{consts::DEFAULT_GRAPH_RDFOX, Class},
    std::{ops::Deref, sync::Arc},
};

/// Some simple queries about a [`Class`](Class)
#[derive(Debug, Clone)]
pub struct ClassReport<'a>(pub &'a Class);

impl<'a> std::fmt::Display for ClassReport<'a> {
    /// TODO: Generate a decent looking set of class metrics
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
}

impl<'a> ClassReport<'a> {
    pub fn number_of_individuals(
        &self,
        tx: &Arc<Transaction>,
    ) -> Result<usize, rdf_store_rs::RDFStoreError> {
        let default_graph = DEFAULT_GRAPH_RDFOX.deref().as_display_iri();
        let prefixes = Namespaces::builder()
            .declare(self.0.namespace.clone())
            .build()?;
        let sparql = formatdoc! {r##"
            SELECT DISTINCT ?thing
            WHERE {{
                {{
                    GRAPH ?graph {{
                        ?thing a {self}
                    }}
                }} UNION {{
                        ?thing a {self}
                    BIND({default_graph} AS ?graph)
                }}
            }}
            "##
        };
        tracing::debug!(target: "sparql", "\n{sparql}");
        let count_result = Statement::new(&prefixes, sparql.into())?
            .cursor(
                &tx.connection,
                &Parameters::empty()?.fact_domain(FactDomain::ALL)?,
            )?
            .count(tx);
        #[allow(clippy::let_and_return)]
        count_result
    }

    pub fn number_of_individuals_in_graph(
        &self,
        tx: &Arc<Transaction>,
        graph_connection: &GraphConnection,
    ) -> Result<usize, rdf_store_rs::RDFStoreError> {
        let graph = graph_connection.graph.as_display_iri();
        let prefixes = Namespaces::builder()
            .declare(self.0.namespace.clone())
            .build()?;
        let sparql = formatdoc! {r##"
            SELECT DISTINCT ?thing
            WHERE {{
                GRAPH {graph} {{
                    ?thing a {self}
                }}
            }}
            "##
        };
        tracing::debug!(target: "sparql", "\n{sparql}");
        let count_result = Statement::new(&prefixes, sparql.into())?
            .cursor(
                &graph_connection.data_store_connection,
                &Parameters::empty()?.fact_domain(FactDomain::ALL)?,
            )?
            .count(tx);
        #[allow(clippy::let_and_return)]
        count_result
    }
}
