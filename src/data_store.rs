use {
    crate::{
        connectable_data_store::ConnectableDataStore,
        Parameters,
        server_connection::ServerConnection,
    },
    owo_colors::OwoColorize,
    r2d2::Pool
    ,
    std::{
        fmt::{Display, Formatter},
        sync::Arc,
    },
};

/// A `DataStore` encapsulates a unit of logically related information.
///
/// See <https://docs.oxfordsemantic.tech/data-stores.html>
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct DataStore {
    pub name: String,
    pub parameters: Parameters,
}

impl Display for DataStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "datastore [{}]", self.name.green())
    }
}

impl DataStore {
    pub fn declare_with_parameters(
        name: &str,
        parameters: Parameters,
    ) -> Result<Arc<Self>, ekg_error::Error> {
        Ok(Arc::new(Self {
            name: name.to_string(),
            parameters,
        }))
    }

    pub fn create(self, server_connection: &Arc<ServerConnection>) -> Result<(), ekg_error::Error> {
        server_connection.create_data_store(&self).map(|_| ())
    }

    pub fn pool_for(
        self: &Arc<DataStore>,
        server_connection: &Arc<ServerConnection>,
        create: bool,
        release_on_return_to_pool: bool,
    ) -> Result<Pool<ConnectableDataStore>, ekg_error::Error> {
        if create {
            server_connection.create_data_store(self)?;
        }

        let cds = ConnectableDataStore::new(self, server_connection, release_on_return_to_pool);
        let pool = cds.build_pool()?;
        Ok(pool)
    }
}
