// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use core::fmt::{Display, Formatter};
use std::{ffi::CString, ops::Deref};

use indoc::formatdoc;

use crate::{error::Error, Cursor, DataStoreConnection, Parameters, Prefixes, DEFAULT_GRAPH};

/// SPARQL Statement
#[derive(Debug, PartialEq, Clone)]
pub struct Statement<'a> {
    pub prefixes:    &'a Prefixes,
    pub(crate) text: String,
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "SPARQL Statement:\n{}", self.text)
    }
}

impl<'a> Statement<'a> {
    pub fn new(prefixes: &'a Prefixes, statement: &str) -> Result<Self, Error> {
        let s = Self {
            prefixes,
            text: statement.trim().into(),
        };
        log::trace!("{:}", s);
        Ok(s)
    }

    pub fn cursor<'b>(
        self,
        connection: &'a DataStoreConnection,
        parameters: &Parameters,
    ) -> Result<Cursor<'a>, Error> {
        Cursor::create(connection, parameters, self)
    }

    pub(crate) fn as_c_string(&self) -> Result<CString, Error> {
        Ok(CString::new(self.text.as_str())?)
    }

    /// Return a Statement that can be used to export all data in
    /// `application/nquads` format
    pub fn nquads_query(prefixes: &'a Prefixes) -> Result<Statement<'a>, Error> {
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
