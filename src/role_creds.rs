// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct RoleCreds {
    pub(crate) role_name: String,
    pub(crate) password: String,
}

impl RoleCreds {
    #[allow(dead_code)]
    pub fn new(role_name: &str, password: &str) -> Self {
        Self {
            role_name: role_name.to_string(),
            password: password.to_string(),
        }
    }

    pub fn default() -> Self {
        Self {
            role_name: "".to_string(),
            password: "".to_string(),
        }
    }
}
