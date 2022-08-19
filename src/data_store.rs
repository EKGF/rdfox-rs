use std::fmt::{Display, Formatter};

use crate::{error::Error, Parameters, ServerConnection};

#[derive(Debug, PartialEq, Clone)]
pub struct DataStore<'a> {
    pub name:       String,
    pub parameters: &'a Parameters,
}

impl<'a> Display for DataStore<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "data store [{}]", self.name)
    }
}

impl<'a> DataStore<'a> {
    pub fn declare_with_parameters(name: &str, parameters: &'a Parameters) -> Result<Self, Error> {
        Ok(Self {
            name: name.to_string(),
            parameters,
        })
    }

    pub fn create(self, server_connection: &'a ServerConnection) -> Result<(), Error> {
        server_connection.create_data_store(&self).map(|_| ())
    }
}
