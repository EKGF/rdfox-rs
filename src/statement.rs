// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use core::fmt::{Display, Formatter};
use crate::{Cursor, DataStoreConnection, Error, Parameters, Prefixes};

/// SPARQL Statement
pub struct Statement<'a> {
    pub prefixes: &'a Prefixes,
    pub(crate) text: String,
}

impl Display for Statement<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "SPARQL Statement: {}", self.text)
    }
}

impl<'a> Statement<'a> {

    pub fn query(
        prefixes: &'a Prefixes,
        statement: &str
    ) -> Result<Self, Error> {
        let s = Self { prefixes, text: statement.into() };
        log::debug!("{:}", s);
        Ok(s)
    }

    pub fn cursor(
        &self,
        connection: &DataStoreConnection,
        parameters: &Parameters
    ) -> Result<Cursor, Error> {
        Cursor::create(connection, &parameters, &self)
    }
}
