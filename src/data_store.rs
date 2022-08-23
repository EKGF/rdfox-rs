use std::fmt::{Display, Formatter};

use crate::{error::Error, Parameters, ServerConnection};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DataStore {
    pub name:       String,
    pub parameters: Parameters,
}

impl Display for DataStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "data store [{}]", self.name)
    }
}

impl DataStore {
    pub fn declare_with_parameters(name: &str, parameters: Parameters) -> Result<Self, Error> {
        Ok(Self {
            name: name.to_string(),
            parameters,
        })
    }

    pub fn create(self, server_connection: &ServerConnection) -> Result<(), Error> {
        server_connection.create_data_store(&self).map(|_| ())
    }
}
