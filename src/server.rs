// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{
        database_call,
        root::{
            CServerConnection,
            CServerConnection_newServerConnection,
            CServer_createFirstLocalServerRole,
            CServer_getNumberOfLocalServerRoles,
            CServer_startLocalServer,
            CServer_stopLocalServer,
        },
        server_connection::ServerConnection,
        Parameters,
        RoleCreds,
    },
    rdf_store_rs::{
        consts::LOG_TARGET_DATABASE,
        Error::{self, CouldNotConnectToServer},
    },
    std::{
        ffi::CString,
        ptr,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    },
};

#[derive(Debug)]
pub struct Server {
    default_role_creds: RoleCreds,
    running:            AtomicBool,
}

impl Drop for Server {
    fn drop(&mut self) { self.stop(); }
}

impl std::fmt::Display for Server {
    // noinspection RsUnreachableCode
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "server {self:p}")
    }
}

impl Server {
    pub fn is_running(&self) -> bool { self.running.load(Ordering::Relaxed) }

    pub fn start(role_creds: RoleCreds) -> Result<Arc<Self>, Error> {
        Self::start_with_parameters(role_creds, &Parameters::empty()?)
    }

    pub fn start_with_parameters(
        role_creds: RoleCreds,
        params: &Parameters,
    ) -> Result<Arc<Self>, Error> {
        database_call!(
            "starting a local RDFFox server",
            CServer_startLocalServer(params.inner)
        )?;
        let server = Server {
            default_role_creds: role_creds,
            running:            AtomicBool::new(true),
        };

        if server.get_number_of_local_server_roles()? == 0 {
            server.create_role(&server.default_role_creds)?;
        }

        tracing::debug!("Local RDFox server has been started");
        Ok(Arc::new(server))
    }

    pub fn create_role(&self, role_creds: &RoleCreds) -> Result<(), Error> {
        let c_role_name = CString::new(role_creds.role_name.as_str()).unwrap();
        let c_password = CString::new(role_creds.password.as_str()).unwrap();
        let msg = format!(
            "Creating server role named [{}]",
            role_creds.role_name
        );
        database_call!(
            msg.as_str(),
            CServer_createFirstLocalServerRole(c_role_name.as_ptr(), c_password.as_ptr())
        )
    }

    pub fn get_number_of_local_server_roles(&self) -> Result<u16, Error> {
        let mut number_of_roles = 0_usize;
        database_call!(
            "Getting the number of local server roles",
            CServer_getNumberOfLocalServerRoles(&mut number_of_roles)
        )?;
        Ok(number_of_roles as u16)
    }

    pub fn connection_with_default_role(self: &Arc<Self>) -> Result<Arc<ServerConnection>, Error> {
        let role_creds = &self.default_role_creds;
        self.connection(role_creds.clone())
    }

    pub fn connection(
        self: &Arc<Self>,
        role_creds: RoleCreds,
    ) -> Result<Arc<ServerConnection>, Error> {
        let c_role_name = CString::new(role_creds.role_name.as_str()).unwrap();
        let c_password = CString::new(role_creds.password.as_str()).unwrap();
        let mut server_connection_ptr: *mut CServerConnection = ptr::null_mut();
        database_call!(
            "Creating a server connection",
            CServerConnection_newServerConnection(
                c_role_name.as_ptr(),
                c_password.as_ptr(),
                &mut server_connection_ptr,
            )
        )?;
        if server_connection_ptr.is_null() {
            tracing::error!("Could not establish connection to server");
            Err(CouldNotConnectToServer)
        } else {
            tracing::debug!("Established connection to server");
            Ok(Arc::new(ServerConnection::new(
                role_creds,
                self.clone(),
                server_connection_ptr,
            )))
        }
    }

    pub fn stop(&mut self) {
        *self.running.get_mut() = false;
        unsafe {
            CServer_stopLocalServer();
        }
        tracing::trace!(
            target: LOG_TARGET_DATABASE,
            "Stopped local RDFox server"
        );
    }
}
