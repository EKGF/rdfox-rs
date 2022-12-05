use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

use r2d2::Pool;

use crate::{
    connectable_data_store::ConnectableDataStore,
    error::Error,
    server_connection::ServerConnection,
    Parameters,
};

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
    pub fn declare_with_parameters(name: &str, parameters: Parameters) -> Result<Arc<Self>, Error> {
        Ok(Arc::new(Self {
            name: name.to_string(),
            parameters,
        }))
    }

    pub fn create(self, server_connection: &Arc<ServerConnection>) -> Result<(), Error> {
        server_connection.create_data_store(&self).map(|_| ())
    }

    pub fn pool_for(
        self: &Arc<DataStore>,
        server_connection: &Arc<ServerConnection>,
        create: bool,
        release_on_return_to_pool: bool,
    ) -> Result<Pool<ConnectableDataStore>, Error> {
        if create {
            server_connection.create_data_store(self)?;
        }

        let cds = ConnectableDataStore::new(self, server_connection, release_on_return_to_pool);
        let pool = cds.build_pool()?;
        Ok(pool)
    }
}
