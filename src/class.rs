// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ops::Deref;
use indoc::formatdoc;
use crate::{DataStoreConnection, DEFAULT_GRAPH, Error, FactDomain, Parameters, Prefix, Prefixes, Statement};

#[derive(Debug, Clone)]
pub struct Class {
    pub prefix: Prefix,
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

    pub fn number_of_individuals(
        &self,
        ds_connection: &DataStoreConnection,
    ) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        let prefixes =
            Prefixes::builder().declare(self.prefix.clone()).build()?;
        let count_result = Statement::query(
            &prefixes,
            (formatdoc! {r##"
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
            })
                .as_str(),
        )?
            .cursor(
                ds_connection,
                &Parameters::empty()?.fact_domain(FactDomain::ALL)?,
            )?
            .count();
        #[allow(clippy::let_and_return)]
        count_result
    }
}

#[cfg(test)]
mod tests {
    use iref::Iri;

    use crate::Prefix;
    use crate::class::Class;

    #[test]
    fn test_a_class() {
        let prefix = Prefix::declare(
            "test:",
            Iri::new("http://whatever.com/test#").unwrap(),
        );
        let class = Class::declare(prefix, "SomeClass");
        let s = format!("{:}", class);
        assert_eq!(s, "test:SomeClass")
    }
}

