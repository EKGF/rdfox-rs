use crate::{Error, ServerConnection};
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct DataStore {
    pub(crate) name: String,
}

impl Display for DataStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "data store [{}]", self.name)
    }
}

impl DataStore {
    pub fn define(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn create(self, server_connection: &ServerConnection) -> Result<Self, Error> {
        server_connection.create_data_store(self)
    }
}
