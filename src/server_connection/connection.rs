use {
    crate::{
        database_call,
        rdfox_api::{
            CServerConnection,
            CServerConnection_createDataStore,
            CServerConnection_deleteDataStore,
            CServerConnection_destroy,
            CServerConnection_getMemoryUse,
            CServerConnection_getNumberOfThreads,
            CServerConnection_getVersion,
            CServerConnection_newDataStoreConnection,
            CServerConnection_setNumberOfThreads,
        },
        DataStore,
        DataStoreConnection,
        RDFStoreError,
        RoleCreds,
        Server,
    },
    rdf_store_rs::consts::LOG_TARGET_DATABASE,
    std::{
        ffi::{CStr, CString},
        ptr,
        sync::Arc,
    },
};

/// A connection to a given [`Server`].
#[derive(Debug)]
pub struct ServerConnection {
    #[allow(dead_code)]
    role_creds: RoleCreds,
    server:     Arc<Server>,
    inner:      *mut CServerConnection,
}

unsafe impl Sync for ServerConnection {}

unsafe impl Send for ServerConnection {}

impl Drop for ServerConnection {
    fn drop(&mut self) {
        assert!(
            !self.inner.is_null(),
            "Could not drop ServerConnection"
        );
        unsafe {
            CServerConnection_destroy(self.inner);
        }
        self.inner = ptr::null_mut();
        tracing::debug!(target: LOG_TARGET_DATABASE, "Dropped {self:}");
    }
}

impl std::fmt::Display for ServerConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "connection to {:}", self.server)
    }
}

impl ServerConnection {
    pub(crate) fn new(
        role_creds: RoleCreds,
        server: Arc<Server>,
        server_connection_ptr: *mut CServerConnection,
    ) -> Self {
        assert!(!server_connection_ptr.is_null());
        assert!(
            server.is_running(),
            "cannot connect to an RDFox server that is not running"
        );
        let connection = Self { role_creds, server, inner: server_connection_ptr };
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "Established {connection:}"
        );
        connection
    }

    /// Return the version number of the underlying database engine
    ///
    /// CRDFOX const CException*
    /// CServerConnection_getVersion(
    ///     CServerConnection* serverConnection,
    ///     const char** version
    /// );
    pub fn get_version(&self) -> Result<String, RDFStoreError> {
        let mut c_buf: *const std::os::raw::c_char = ptr::null();
        database_call!(
            "Getting the version",
            CServerConnection_getVersion(self.inner, &mut c_buf)
        )?;
        let c_version = unsafe { CStr::from_ptr(c_buf) };
        Ok(c_version.to_str().unwrap().to_owned())
    }

    pub fn get_number_of_threads(&self) -> Result<u32, RDFStoreError> {
        let mut number_of_threads = 0_u64;
        database_call!(
            format!(
                "Getting the number of server-threads via {}",
                self
            )
            .as_str(),
            CServerConnection_getNumberOfThreads(self.inner, &mut number_of_threads)
        )?;
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "Number of threads is {}",
            number_of_threads
        );
        Ok(number_of_threads as u32)
    }

    pub fn set_number_of_threads(&self, number_of_threads: u64) -> Result<(), RDFStoreError> {
        assert!(!self.inner.is_null());
        let msg = format!(
            "Setting the number of threads to {}",
            number_of_threads
        );
        database_call!(
            msg.as_str(),
            CServerConnection_setNumberOfThreads(self.inner, number_of_threads)
        )
    }

    pub fn get_memory_use(&self) -> Result<(u64, u64), RDFStoreError> {
        let mut max_used_bytes = 0_u64;
        let mut available_bytes = 0_u64;
        database_call!(CServerConnection_getMemoryUse(
            self.inner,
            &mut max_used_bytes,
            &mut available_bytes
        ))?;
        Ok((max_used_bytes, available_bytes))
    }

    pub fn delete_data_store(&self, data_store: &DataStore) -> Result<(), RDFStoreError> {
        assert!(!self.inner.is_null());
        let msg = format!("Deleting {data_store}");
        let c_name = CString::new(data_store.name.as_str()).unwrap();
        database_call!(
            msg.as_str(),
            CServerConnection_deleteDataStore(self.inner, c_name.as_ptr())
        )
    }

    pub fn create_data_store(&self, data_store: &DataStore) -> Result<(), RDFStoreError> {
        tracing::trace!(
            target: LOG_TARGET_DATABASE,
            "Creating {data_store:}"
        );
        assert!(!self.inner.is_null());
        let c_name = CString::new(data_store.name.as_str()).unwrap();
        database_call!(
            "creating a datastore",
            CServerConnection_createDataStore(
                self.inner,
                c_name.as_ptr(),
                data_store.parameters.inner.cast_const(),
            )
        )?;
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "Created {data_store:}"
        );
        Ok(())
    }

    pub fn connect_to_data_store(
        self: &Arc<Self>,
        data_store: &Arc<DataStore>,
    ) -> Result<Arc<DataStoreConnection>, RDFStoreError> {
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            "Connecting to {}",
            data_store
        );
        assert!(!self.inner.is_null());
        let mut ds_connection = DataStoreConnection::new(self, data_store, ptr::null_mut());
        let c_name = CString::new(data_store.name.as_str()).unwrap();
        tracing::debug!(
            target: LOG_TARGET_DATABASE,
            conn = ds_connection.number,
            "Creating datastore connection #{}",
            ds_connection.number
        );
        database_call!(
            "Creating datastore connection",
            CServerConnection_newDataStoreConnection(
                self.inner,
                c_name.as_ptr(),
                &mut ds_connection.inner,
            )
        )?;
        tracing::info!(
            target: LOG_TARGET_DATABASE,
            "Connected to {}",
            data_store
        );
        Ok(Arc::new(ds_connection))
    }
}
