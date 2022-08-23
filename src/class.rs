// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ops::Deref;

use indoc::formatdoc;

use crate::{
    Error,
    FactDomain,
    GraphConnection,
    Parameters,
    Prefix,
    Prefixes,
    Statement,
    Transaction,
    DEFAULT_GRAPH,
};

#[derive(Debug, Clone)]
pub struct Class {
    pub prefix:     Prefix,
    pub local_name: String,
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.prefix.name.as_str(),
            self.local_name.as_str()
        )
    }
}

impl Class {
    pub fn declare(prefix: Prefix, local_name: &str) -> Self {
        Self {
            prefix,
            local_name: local_name.to_string(),
        }
    }

    pub fn display_turtle<'a>(&'a self) -> impl std::fmt::Display + 'a {
        struct TurtleClass<'a>(&'a Class);
        impl<'a> std::fmt::Display for TurtleClass<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}{}", self.0.prefix.name, self.0.local_name)
            }
        }
        TurtleClass(self)
    }

    pub fn number_of_individuals(&self, tx: &Transaction) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        let prefixes = Prefixes::builder().declare(self.prefix.clone()).build()?;
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
        log::debug!(target: "sparql", "\n{sparql}");
        let count_result = Statement::new(&prefixes, sparql.as_str())?
            .cursor(
                tx.connection,
                &Parameters::empty()?.fact_domain(FactDomain::ALL)?,
            )?
            .count(tx);
        #[allow(clippy::let_and_return)]
        count_result
    }

    pub fn number_of_individuals_in_graph(
        &self,
        tx: &Transaction,
        graph_connection: &GraphConnection,
    ) -> Result<u64, Error> {
        let graph = graph_connection.graph.as_display_iri();
        let prefixes = Prefixes::builder().declare(self.prefix.clone()).build()?;
        let sparql = formatdoc! {r##"
            SELECT DISTINCT ?thing
            WHERE {{
                GRAPH {graph} {{
                    ?thing a {self}
                }}
            }}
            "##
        };
        log::debug!(target: "sparql", "\n{sparql}");
        let count_result = Statement::new(&prefixes, sparql.as_str())?
            .cursor(
                graph_connection.data_store_connection,
                &Parameters::empty()?.fact_domain(FactDomain::ALL)?,
            )?
            .count(tx);
        #[allow(clippy::let_and_return)]
        count_result
    }
}

#[cfg(test)]
mod tests {
    use iref::Iri;

    use crate::{class::Class, Prefix};

    #[test]
    fn test_a_class() {
        let prefix = Prefix::declare("test:", Iri::new("http://whatever.com/test#").unwrap());
        let class = Class::declare(prefix, "SomeClass");
        let s = format!("{:}", class);
        assert_eq!(s, "test:SomeClass")
    }
}
