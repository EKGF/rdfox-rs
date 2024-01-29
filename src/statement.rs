// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    core::fmt::{Display, Formatter},
    crate::{Cursor, DataStoreConnection, Namespaces, Parameters},
    ekg_namespace::consts::{DEFAULT_GRAPH_RDFOX, LOG_TARGET_SPARQL},
    indoc::formatdoc,
    std::{borrow::Cow, ffi::CString, ops::Deref, sync::Arc},
};

/// SPARQL Statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Statement {
    pub prefixes: Arc<Namespaces>,
    pub(crate) text: String,
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "SPARQL Statement:")?;
        for (number, line) in self.text.lines().enumerate() {
            writeln!(f, "{:0>4}: {line}", number + 1)?;
        }
        Ok(())
    }
}

impl Statement {
    pub fn new(prefixes: &Arc<Namespaces>, statement: Cow<str>) -> Result<Self, ekg_error::Error> {
        let s = Self {
            prefixes: prefixes.clone(),
            text: format!("{}\n{}", &prefixes.to_string(), statement.trim()),
        };
        tracing::trace!(target: LOG_TARGET_SPARQL, "{:}", s);
        Ok(s)
    }

    pub fn cursor(
        &self,
        connection: &Arc<DataStoreConnection>,
        parameters: &Parameters,
    ) -> Result<Cursor, ekg_error::Error> {
        Cursor::create(connection, parameters, self)
    }

    pub(crate) fn as_c_string(&self) -> Result<CString, ekg_error::Error> {
        Ok(CString::new(self.text.as_str())?)
    }

    pub fn as_str(&self) -> &str { self.text.as_str() }

    pub fn no_comments(&self) -> String { no_comments(self.text.as_str()) }

    /// Return a Statement that can be used to export all data in
    /// `application/nquads` format
    pub fn nquads_query(prefixes: &Arc<Namespaces>) -> Result<Statement, ekg_error::Error> {
        let default_graph = DEFAULT_GRAPH_RDFOX.deref().as_display_iri();
        let statement = Statement::new(
            prefixes,
            formatdoc!(
                r##"
                SELECT ?S ?P ?O ?G
                WHERE {{
                    {{
                        GRAPH ?G {{ ?S ?P ?O }}
                    }} UNION {{
                        ?S ?P ?P .
                        BIND({default_graph} AS ?G)
                    }}
                }}
            "##
            )
                .into(),
        )?;
        Ok(statement)
    }
}

pub fn no_comments(string: &str) -> String {
    use std::fmt::Write;

    let re = fancy_regex::Regex::new(r"(.*)(?!#>)#.*$").unwrap();

    let do_line = |line: &str| -> (bool, String) {
        let caps = re.captures(line);
        if let Ok(Some(caps)) = caps {
            let mat = caps.get(1).unwrap();
            (
                true,
                line[mat.start()..mat.end()].trim_end().to_string(),
            )
        } else {
            (false, line.trim_end().to_string())
        }
    };

    let mut output = String::new();
    for line in string.lines() {
        let mut line = line.to_string();
        loop {
            let (again, result) = do_line(line.as_str());
            if again {
                // Repeat the call to do_line again to make sure that all #-comments are removed
                // (there could be multiple on one line)
                line = result;
            } else {
                writeln!(&mut output, "{result}").unwrap();
                break;
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    #[test_log::test]
    fn test_no_comments() {
        let sparql = indoc::formatdoc! {r##"
            PREFIX abc: <https://whatever.org#> # focus on this and the next line
            PREFIX owl: <http://www.w3.org/2002/07/owl#>
            SELECT DISTINCT ?thing
            WHERE {{
                {{ # some comment
                    GRAPH ?graph {{ # more # and more
                        ?thing a Whatever#
                    }}
                }} UNION {{
                    ?thing a Whatever .# abc
                                       # def
                    BIND(graph:Graph AS ?graph)
                }}
            }}
            "##
        };
        let expected = indoc::formatdoc! {r##"
            PREFIX abc: <https://whatever.org#>
            PREFIX owl: <http://www.w3.org/2002/07/owl#>
            SELECT DISTINCT ?thing
            WHERE {{
                {{
                    GRAPH ?graph {{
                        ?thing a Whatever
                    }}
                }} UNION {{
                    ?thing a Whatever .

                    BIND(graph:Graph AS ?graph)
                }}
            }}
            "##
        };
        let actual = crate::statement::no_comments(sparql.as_str());
        assert_eq!(actual.as_str(), expected.as_str());
    }
}
