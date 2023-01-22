// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

const RDFOX_DEFAULT_ROLE_USERID: &str = "admin";
const RDFOX_DEFAULT_ROLE_PASSWD: &str = "admin";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoleCreds {
    pub(crate) role_name: String,
    pub(crate) password:  String,
}

impl RoleCreds {
    #[allow(dead_code)]
    pub fn new(role_name: &str, password: &str) -> Self {
        Self {
            role_name: role_name.to_string(),
            password:  password.to_string(),
        }
    }

    pub fn default() -> Self {
        Self {
            role_name: RDFOX_DEFAULT_ROLE_USERID.to_string(),
            password:  RDFOX_DEFAULT_ROLE_PASSWD.to_string(),
        }
    }
}
