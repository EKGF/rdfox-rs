// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    ::r2d2::{ManageConnection, Pool},
    crate::{DataStore, DataStoreConnection, ServerConnection},
    std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    }
    ,
};

/// A pool-able connectable [`DataStore`]
pub struct ConnectableDataStore {
    data_store: Arc<DataStore>,
    server_connection: Arc<ServerConnection>,
    /// Indicates that we want to release all connections on return to the pool
    /// (used to shutdown gracefully)
    release_on_return_to_pool: AtomicBool,
}

impl ConnectableDataStore {
    /// release_on_return_to_pool: Mark connection as "destroy" when return back
    /// to pool
    pub fn new(
        data_store: &Arc<DataStore>,
        server_connection: &Arc<ServerConnection>,
        release_on_return_to_pool: bool,
    ) -> Self {
        Self {
            data_store: data_store.clone(),
            server_connection: server_connection.clone(),
            release_on_return_to_pool: AtomicBool::new(release_on_return_to_pool),
        }
    }

    /// Build an `r2d2::Pool` for the given `DataStore` and `ServerConnection`
    pub fn build_pool(self) -> Result<Pool<ConnectableDataStore>, ekg_error::Error> {
        let cds = Pool::builder()
            .max_size(self.server_connection.get_number_of_threads()?)
            .build(self)?;
        Ok(cds)
    }
}

impl ManageConnection for ConnectableDataStore {
    type Connection = Arc<DataStoreConnection>;
    type Error = ekg_error::Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        self.server_connection
            .connect_to_data_store(&self.data_store)
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> Result<(), Self::Error> { Ok(()) }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        self.release_on_return_to_pool.load(Ordering::Relaxed)
    }
}
