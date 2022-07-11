// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CString;
use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::{
    Connection,
    RoleCreds,
    root::{
        CException,
        CParameters_getEmptyParameters,
        CServer_createFirstLocalServerRole,
        CServer_startLocalServer,
        CServerConnection,
        CServerConnection_newServerConnection,
    },
};
use crate::exception::Error;

pub struct Server {}

impl Server {
    pub fn start(role_creds: &RoleCreds) -> Result<Self, Error> {
        log::debug!("Starting local RDFox server");
        CException::handle(|| unsafe {
            CServer_startLocalServer(CParameters_getEmptyParameters())
        })?;

        let server = Server {};

        server.create_role(role_creds)?;

        log::debug!("Local RDFox server has been started");
        Ok(server)
    }

    pub fn create_role(&self, role_creds: &RoleCreds) -> Result<(), Error> {
        let c_role_name = CString::new(role_creds.role_name.as_str()).unwrap();
        let c_password = CString::new(role_creds.password.as_str()).unwrap();
        log::debug!("Creating server role named [{}]", role_creds.role_name);
        CException::handle(|| unsafe {
            CServer_createFirstLocalServerRole(c_role_name.as_ptr(), c_password.as_ptr())
        })?;
        log::debug!("Created server role named [{}]", role_creds.role_name);
        Ok(())
    }

    pub fn connection(&self, role_creds: &RoleCreds) -> Result<Connection, Error> {
        let c_role_name = CString::new(role_creds.role_name.as_str()).unwrap();
        let c_password = CString::new(role_creds.password.as_str()).unwrap();
        let mut server_connection_ptr: *mut CServerConnection = ptr::null_mut();
        CException::handle(AssertUnwindSafe(|| unsafe {
            CServerConnection_newServerConnection(
                c_role_name.as_ptr(),
                c_password.as_ptr(),
                &mut server_connection_ptr,
            )
        }))?;
        log::debug!("Established connection");
        Ok(Connection::new(role_creds, server_connection_ptr))
    }
}
