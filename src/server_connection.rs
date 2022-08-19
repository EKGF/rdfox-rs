// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{ffi::CString, ptr};

use crate::{
    database_call,
    error::Error,
    root::{
        CServerConnection,
        CServerConnection_createDataStore,
        CServerConnection_deleteDataStore,
        CServerConnection_destroy,
        CServerConnection_getNumberOfThreads,
        CServerConnection_newDataStoreConnection,
        CServerConnection_setNumberOfThreads,
    },
    DataStore,
    DataStoreConnection,
    RoleCreds,
};

pub struct ServerConnection {
    #[allow(dead_code)]
    role_creds: RoleCreds,
    inner:      *mut CServerConnection,
}

impl Drop for ServerConnection {
    fn drop(&mut self) { self.destroy() }
}

impl ServerConnection {
    pub(crate) fn new(
        role_creds: &RoleCreds,
        server_connection_ptr: *mut CServerConnection,
    ) -> Self {
        assert!(!server_connection_ptr.is_null());
        Self {
            role_creds: role_creds.clone(),
            inner:      server_connection_ptr,
        }
    }

    pub fn get_number_of_threads(&self) -> Result<std::os::raw::c_ulong, Error> {
        let mut number_of_threads = 0 as std::os::raw::c_ulong;
        database_call!(
            "getting the number of threads",
            CServerConnection_getNumberOfThreads(self.inner, &mut number_of_threads)
        )?;
        log::debug!("Number of threads is {}", number_of_threads);
        Ok(number_of_threads)
    }

    pub fn set_number_of_threads(
        &self,
        number_of_threads: std::os::raw::c_ulong,
    ) -> Result<(), Error> {
        log::debug!("Setting the number of threads to {}", number_of_threads);
        database_call!(
            "setting the number of threads",
            CServerConnection_setNumberOfThreads(self.inner, number_of_threads)
        )
    }

    pub fn delete_data_store(&self, data_store: DataStore) -> Result<(), Error> {
        log::debug!("Creating {data_store}");
        let c_name = CString::new(data_store.name.as_str()).unwrap();
        database_call!(
            "deleting a datastore",
            CServerConnection_deleteDataStore(self.inner, c_name.as_ptr())
        )?;
        log::info!("Deleted {data_store}");
        Ok(())
    }

    pub fn create_data_store<'a>(&self, data_store: &'a DataStore<'a>) -> Result<(), Error> {
        log::debug!("Creating {data_store}");
        let c_name = CString::new(data_store.name.as_str()).unwrap();
        database_call!(
            "creating a datastore",
            CServerConnection_createDataStore(
                self.inner,
                c_name.as_ptr(),
                data_store.parameters.inner,
            )
        )?;
        log::info!("Created {data_store}");
        Ok(())
    }

    pub fn connect_to_data_store<'a>(
        &self,
        data_store: &'a DataStore<'a>,
    ) -> Result<DataStoreConnection<'a>, Error> {
        log::debug!("Connecting to {}", data_store);
        let mut ds_connection = DataStoreConnection::new(data_store, ptr::null_mut());
        let c_name = CString::new(data_store.name.as_str()).unwrap();
        database_call!(
            "creating a datastore connection",
            CServerConnection_newDataStoreConnection(
                self.inner,
                c_name.as_ptr(),
                &mut ds_connection.inner,
            )
        )?;
        log::debug!("Connected to {}", data_store);
        Ok(ds_connection)
    }

    fn destroy(&mut self) {
        unsafe {
            CServerConnection_destroy(self.inner);
        }
        self.inner = ptr::null_mut();
        log::debug!("Destroyed connection");
    }
}
