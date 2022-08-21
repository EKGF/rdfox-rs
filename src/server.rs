// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{ffi::CString, ptr};

use crate::{
    database_call,
    error::{Error, Error::CouldNotConnectToServer},
    root::{
        CServerConnection,
        CServerConnection_newServerConnection,
        CServer_createFirstLocalServerRole,
        CServer_startLocalServer,
        CServer_getNumberOfLocalServerRoles,
        CServer_stopLocalServer
    },
    Parameters,
    RoleCreds,
    ServerConnection,
};

pub struct Server {
    default_role_creds: RoleCreds,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop();
    }
}

impl std::fmt::Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "server {self:p})")
    }
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
        database_call!(
            "starting a local server",
            CServer_startLocalServer(params.inner)
        )?;
        let server = Server {
            default_role_creds: role_creds,
        };

        if server.get_number_of_local_server_roles()? == 0 {
            server.create_role(&server.default_role_creds)?;
        }

        log::debug!("Local RDFox server has been started");
        Ok(server)
    }

    pub fn create_role(&self, role_creds: &RoleCreds) -> Result<(), Error> {
        let c_role_name = CString::new(role_creds.role_name.as_str()).unwrap();
        let c_password = CString::new(role_creds.password.as_str()).unwrap();
        log::debug!("Creating server role named [{}]", role_creds.role_name);
        database_call!(
            "creating a local server role",
            CServer_createFirstLocalServerRole(c_role_name.as_ptr(), c_password.as_ptr())
        )?;
        log::debug!("Created server role named [{}]", role_creds.role_name);
        Ok(())
    }

    pub fn get_number_of_local_server_roles(&self) -> Result<u16, Error> {
        let mut number_of_roles = 0_u64;
        database_call!(
            "getting the number of local server roles",
            CServer_getNumberOfLocalServerRoles(&mut number_of_roles)
        )?;
        Ok(number_of_roles as u16)
    }

    pub fn connection_with_default_role(&self) -> Result<ServerConnection, Error> {

        self.connection(&self.default_role_creds)
    }

    pub fn connection(&self, role_creds: &RoleCreds) -> Result<ServerConnection, Error> {
        let c_role_name = CString::new(role_creds.role_name.as_str()).unwrap();
        let c_password = CString::new(role_creds.password.as_str()).unwrap();
        let mut server_connection_ptr: *mut CServerConnection = ptr::null_mut();
        database_call!(
            "creating a server connection",
            CServerConnection_newServerConnection(
                c_role_name.as_ptr(),
                c_password.as_ptr(),
                &mut server_connection_ptr,
            )
        )?;
        if server_connection_ptr.is_null() {
            log::error!("Could not establish connection to server");
            Err(CouldNotConnectToServer)
        } else {
            log::debug!("Established connection to server");
            Ok(ServerConnection::new(role_creds, self, server_connection_ptr))
        }
    }

    pub fn stop(&self) {
        unsafe {
            CServer_stopLocalServer();
        }
        log::trace!("Stopped local RDFox server");
    }
}
