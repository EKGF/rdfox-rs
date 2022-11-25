// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use core::fmt::{Display, Formatter};
use std::{borrow::Cow, ffi::CString, ops::Deref, sync::Arc};

use indoc::formatdoc;
use iref::Iri;

use crate::{error::Error, Cursor, DataStoreConnection, Parameters, Prefixes, DEFAULT_GRAPH};

/// SPARQL Statement
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Statement {
    pub prefixes:    Prefixes,
    pub(crate) text: String,
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "SPARQL Statement:\n")?;
        for (number, line) in self.text.lines().enumerate() {
            writeln!(f, "{:0>4}: {line}", number + 1)?;
        }
        Ok(())
    }
}

impl Statement {
    pub fn new(prefixes: Prefixes, statement: Cow<str>) -> Result<Self, Error> {
        let text = format!("{}\n{}", &prefixes.to_string(), statement.trim());
        let s = Self {
            prefixes,
            text,
        };
        log::trace!(target: crate::LOG_TARGET_SPARQL, "{:}", s);
        Ok(s)
    }

    pub fn cursor<'a>(
        &self,
        connection: &Arc<DataStoreConnection>,
        parameters: &Parameters,
        base_iri: Option<Iri>,
    ) -> Result<Cursor, Error> {
        Cursor::create(connection, parameters, self, base_iri)
    }

    pub(crate) fn as_c_string(&self) -> Result<CString, Error> {
        Ok(CString::new(self.text.as_str())?)
    }

    pub fn as_str(&self) -> &str { self.text.as_str() }

    pub fn no_comments(&self) -> String {
        use std::fmt::Write;

        use regex::Regex;
        let re = Regex::new(r".*#[^>]").unwrap();
        let mut output = String::new();
        for line in self.as_str().lines() {
            if let Some(mat) = re.find(line) {
                write!(&mut output, "{}\n", &line[.. mat.start()]).unwrap();
            } else {
                write!(&mut output, "{}\n", line).unwrap();
            }
        }
        output
    }

    /// Return a Statement that can be used to export all data in
    /// `application/nquads` format
    pub fn nquads_query(prefixes: Prefixes) -> Result<Statement, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
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
