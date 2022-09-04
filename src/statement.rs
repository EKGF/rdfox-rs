// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use core::fmt::{Display, Formatter};
use std::{ffi::CString, ops::Deref};

use indoc::formatdoc;
use iref::Iri;

use crate::{error::Error, Cursor, DataStoreConnection, Parameters, Prefixes, DEFAULT_GRAPH};

/// SPARQL Statement
#[derive(Debug, PartialEq, Clone)]
pub struct Statement {
    pub prefixes:    Prefixes,
    pub(crate) text: String,
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "SPARQL Statement:\n{}", self.text)
    }
}

impl Statement {
    pub fn new(prefixes: Prefixes, statement: &str) -> Result<Self, Error> {
        let text = format!("{}\n{}", &prefixes.to_string(), statement.trim());
        let s = Self {
            prefixes,
            text,
        };
        log::trace!(target: crate::LOG_TARGET_SPARQL, "{:}", s);
        Ok(s)
    }

    pub fn cursor<'a>(
        self,
        connection: &'a DataStoreConnection,
        parameters: &Parameters,
        base_iri: Option<Iri>,
    ) -> Result<Cursor<'a>, Error> {
        Cursor::create(connection, parameters, self, base_iri)
    }

    pub(crate) fn as_c_string(&self) -> Result<CString, Error> {
        Ok(CString::new(self.text.as_str())?)
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
            .as_str(),
        )?;
        Ok(statement)
    }
}
