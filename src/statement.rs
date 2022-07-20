// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use crate::error::Error;
use crate::{Cursor, DataStoreConnection, Parameters, Prefixes};
use core::fmt::{Display, Formatter};

/// SPARQL Statement
pub struct Statement<'a> {
    pub prefixes: &'a Prefixes,
    pub(crate) text: String,
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "SPARQL Statement:\n{}", self.text)
    }
}

impl<'a> Statement<'a> {
    pub fn query(prefixes: &'a Prefixes, statement: &str) -> Result<Self, Error> {
        let s = Self {
            prefixes,
            text: statement.trim().into(),
        };
        log::trace!("{:}", s);
        Ok(s)
    }

    pub fn cursor(
        self,
        connection: &'a DataStoreConnection,
        parameters: &Parameters,
    ) -> Result<Cursor<'a>, Error> {
        Cursor::create(connection, parameters, self)
    }
}
