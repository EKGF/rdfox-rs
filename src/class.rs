use indoc::formatdoc;
use crate::{Class, DataStoreConnection, Error, FactDomain, Parameters, Prefix, Prefixes, Statement};

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
        let prefixes =
            Prefixes::builder().declare(self.prefix.clone()).build()?;
        let count_result = Statement::query(
            &prefixes,
            (formatdoc! {r##"
                SELECT DISTINCT ?thing
                WHERE {{
                    {{
                        GRAPH ?graph {{
                            ?thing a {class}
                        }}
                    }} UNION {{
                            ?thing a {class}
                        BIND("default" AS ?graph)
                    }}
                }}
                "##,
                class = self
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

    use crate::{Class, Prefix};

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

