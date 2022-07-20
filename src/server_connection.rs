// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CString;
use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::error::Error;
use crate::{
    root::{
        CException, CParameters_getEmptyParameters, CServerConnection,
        CServerConnection_createDataStore, CServerConnection_destroy,
        CServerConnection_getNumberOfThreads, CServerConnection_newDataStoreConnection,
        CServerConnection_setNumberOfThreads,
    },
    DataStore, DataStoreConnection, RoleCreds,
};

pub struct ServerConnection {
    #[allow(dead_code)]
    role_creds: RoleCreds,
    inner: *mut CServerConnection,
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
            inner: server_connection_ptr,
        }
    }

    pub fn get_number_of_threads(&self) -> Result<std::os::raw::c_ulong, Error> {
        let mut number_of_threads = 0 as std::os::raw::c_ulong;
        CException::handle(AssertUnwindSafe(|| unsafe {
            CServerConnection_getNumberOfThreads(self.inner, &mut number_of_threads)
        }))?;
        log::debug!("Number of threads is {}", number_of_threads);
        Ok(number_of_threads)
    }

    pub fn set_number_of_threads(
        &self,
        number_of_threads: std::os::raw::c_ulong,
    ) -> Result<(), Error> {
        log::debug!("Setting the number of threads to {}", number_of_threads);
        CException::handle(|| unsafe {
            CServerConnection_setNumberOfThreads(self.inner, number_of_threads)
        })
    }

    pub fn create_data_store(&self, data_store: DataStore) -> Result<DataStore, Error> {
        log::debug!("Creating {data_store}");
        let c_name = CString::new(data_store.name.as_str()).unwrap();
        CException::handle(|| unsafe {
            CServerConnection_createDataStore(
                self.inner,
                c_name.as_ptr(),
                CParameters_getEmptyParameters(),
            )
        })?;
        log::info!("Created {data_store}");
        Ok(data_store)
    }

    pub fn connect_to_data_store(
        &self,
        data_store: DataStore,
    ) -> Result<DataStoreConnection, Error> {
        log::debug!("Connecting to {}", data_store);
        let mut ds_connection = DataStoreConnection::new(data_store.clone(), ptr::null_mut());
        let c_name = CString::new(data_store.name.as_str()).unwrap();
        CException::handle(AssertUnwindSafe(|| unsafe {
            CServerConnection_newDataStoreConnection(
                self.inner,
                c_name.as_ptr(),
                &mut ds_connection.inner,
            )
        }))?;
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
