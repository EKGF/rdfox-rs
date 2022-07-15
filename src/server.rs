// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CString;
use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::exception::Error;
use crate::Error::CouldNotConnectToServer;
use crate::{
    root::{
        CException, CServerConnection, CServerConnection_newServerConnection,
        CServer_createFirstLocalServerRole, CServer_startLocalServer,
    },
    Parameters, RoleCreds, ServerConnection,
};

pub struct Server {
    default_role_creds: RoleCreds,
}

impl Server {
    pub fn start(role_creds: RoleCreds) -> Result<Self, Error> {
        Self::start_with_parameters(role_creds, &Parameters::empty()?)
    }

    pub fn start_with_parameters(
        role_creds: RoleCreds,
        params: &Parameters,
    ) -> Result<Self, Error> {
        log::debug!("Starting local RDFox server with {params}");
        CException::handle(|| unsafe { CServer_startLocalServer(params.inner) })?;

        let server = Server {
            default_role_creds: role_creds,
        };

        server.create_role(&server.default_role_creds)?;

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

    pub fn connection_with_default_role(&self) -> Result<ServerConnection, Error> {
        self.connection(&self.default_role_creds)
    }

    pub fn connection(&self, role_creds: &RoleCreds) -> Result<ServerConnection, Error> {
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
        if server_connection_ptr.is_null() {
            log::error!("Could not establish connection to server");
            Err(CouldNotConnectToServer)
        } else {
            log::debug!("Established connection to server");
            Ok(ServerConnection::new(role_creds, server_connection_ptr))
        }
    }
}
