// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

extern crate alloc;

use {
    alloc::ffi::CString,
    crate::{
        database_call,
        rdfox_api::{
            CParameters,
            CParameters_destroy,
            CParameters_getString,
            CParameters_newEmptyParameters,
            CParameters_setString,
        },
    }
    ,
    std::{
        ffi::CStr,
        fmt::{Display, Formatter},
        os::raw::c_char,
        path::Path,
        ptr,
        sync::Arc,
    },
};

pub enum FactDomain {
    ASSERTED,
    INFERRED,
    ALL,
}

pub enum PersistenceMode {
    File,
    FileSequence,
    Off,
}

impl Display for PersistenceMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceMode::File => write!(f, "file"),
            PersistenceMode::FileSequence => write!(f, "file-sequence"),
            PersistenceMode::Off => write!(f, "off"),
        }
    }
}

pub enum DataStoreType {
    ParallelNN,
    ParallelNW,
    ParallelWW,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parameters {
    pub(crate) inner: Arc<*mut CParameters>,
}

unsafe impl Sync for Parameters {}

unsafe impl Send for Parameters {}

impl Display for Parameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parameters[]") // TODO: show keys and values (currently not
        // possible)
    }
}

impl Drop for Parameters {
    fn drop(&mut self) {
        assert!(
            !self.inner.is_null(),
            "Parameters-object was already dropped"
        );
        unsafe {
            CParameters_destroy(self.inner.cast());
            // tracing::trace!(target: LOG_TARGET_DATABASE, "Dropped Params");
        }
    }
}

const SENSITIVE_PARAMETERS: [&str; 1] = ["license-content"];

impl Parameters {
    pub fn empty() -> Result<Self, ekg_error::Error> {
        let mut parameters: *mut CParameters = ptr::null_mut();
        database_call!(
            "Allocating parameters",
            CParameters_newEmptyParameters(&mut parameters)
        )?;
        Ok(Parameters { inner: Arc::new(parameters) })
    }

    pub fn set_string(&self, key: &str, value: &str) -> Result<(), ekg_error::Error> {
        let c_key = CString::new(key).unwrap();
        let c_value = CString::new(value).unwrap();
        let msg = if SENSITIVE_PARAMETERS.contains(&c_key.to_str().unwrap()) {
            format!(
                "Setting parameter {}=[***]",
                c_key.to_str().unwrap(),
            )
        } else {
            format!(
                "Setting parameter {}=[{}]",
                c_key.to_str().unwrap(),
                c_value.to_str().unwrap()
            )
        };
        database_call!(
            msg.as_str(),
            CParameters_setString(*self.inner, c_key.as_ptr(), c_value.as_ptr())
        )
    }

    pub fn get_string(&self, key: &str, default: &str) -> Result<String, ekg_error::Error> {
        let c_key = CString::new(key).unwrap();
        let c_default = CString::new(default).unwrap();
        let mut c_value: *const c_char = ptr::null();
        let msg = format!(
            "Getting parameter {} with default value {}",
            c_key.to_str().unwrap(),
            c_default.to_str().unwrap()
        );
        database_call!(
            msg.as_str(),
            CParameters_getString(
                *self.inner,
                c_key.as_ptr(),
                c_default.as_ptr(),
                &mut c_value as *mut *const c_char
            )
        )?;
        let c_version = unsafe { CStr::from_ptr(c_value) };
        Ok(c_version.to_str().unwrap().to_owned())
    }

    pub fn fact_domain(self, fact_domain: FactDomain) -> Result<Self, ekg_error::Error> {
        match fact_domain {
            FactDomain::ASSERTED => self.set_string("fact-domain", "explicit")?,
            FactDomain::INFERRED => self.set_string("fact-domain", "derived")?,
            FactDomain::ALL => self.set_string("fact-domain", "all")?,
        };
        Ok(self)
    }

    pub fn switch_off_file_access_sandboxing(self) -> Result<Self, ekg_error::Error> {
        self.set_string("sandbox-directory", "")?;
        Ok(self)
    }

    pub fn persist_datastore(self, mode: PersistenceMode) -> Result<Self, ekg_error::Error> {
        #[cfg(feature = "rdfox-7-0")]
        match mode {
            PersistenceMode::File => self.set_string("persistence", "file")?,
            PersistenceMode::FileSequence => self.set_string("persistence", "file-sequence")?,
            PersistenceMode::Off => self.set_string("persistence", "off")?,
        };
        #[cfg(not(feature = "rdfox-7-0"))]
        match mode {
            PersistenceMode::File => self.set_string("persist-ds", "file")?,
            PersistenceMode::FileSequence => self.set_string("persist-ds", "file-sequence")?,
            PersistenceMode::Off => self.set_string("persist-ds", "off")?,
        };
        Ok(self)
    }

    #[cfg(not(feature = "rdfox-7-0"))]
    pub fn persist_roles(self, mode: PersistenceMode) -> Result<Self, ekg_error::Error> {
        self.set_string("persist-roles", &mode.to_string())?;
        Ok(self)
    }

    pub fn server_directory(self, dir: &Path) -> Result<Self, ekg_error::Error> {
        if dir.is_dir() {
            self.set_string("server-directory", dir.to_str().unwrap())?;
            Ok(self)
        } else {
            panic!("{dir:?} is not a directory")
        }
    }

    pub fn license_file(self, file: &Path) -> Result<Self, ekg_error::Error> {
        if file.is_file() {
            self.set_string("license-file", file.to_str().unwrap())?;
            Ok(self)
        } else {
            panic!("{file:?} does not exist")
        }
    }

    pub fn license_content(self, content: &str) -> Result<Self, ekg_error::Error> {
        // Content that comes in via an environment variable can have literal `\\n`
        // strings in them that should be replaced by actual line-feeds
        let content = content.replace("\r\n", "\n").replace("\\n", "\n");
        // Add a line feed at the end of the content just to make sure, if it's
        // missing the last field in the license key will not be recognized
        self.set_string("license-content", format!("{content}\n").as_str())?;
        Ok(self)
    }

    pub fn set_license(self, database_dir: Option<&Path>) -> Result<Self, ekg_error::Error> {
        match super::license::find_license(database_dir)? {
            (Some(license_file_name), None) => {
                return self.license_file(license_file_name.as_path());
            }
            (None, Some(license_content)) => return self.license_content(license_content.as_str()),
            _ => {}
        };
        Ok(self)
    }

    pub fn import_rename_user_blank_nodes(self, setting: bool) -> Result<Self, ekg_error::Error> {
        self.set_string(
            "import.rename-user-blank-nodes",
            format!("{setting:?}").as_str(),
        )?;
        Ok(self)
    }

    /// If true, all API calls are recorded in a script that
    /// the shell can replay later. later.
    /// The default value is false.
    pub fn api_log(self, on: bool) -> Result<Self, ekg_error::Error> {
        if on {
            self.set_string("api-log", "on")?;
        } else {
            self.set_string("api-log", "off")?;
        }
        Ok(self)
    }

    /// Specifies the directory into which API logs will be written.
    /// Default is directory api-log within the configured server directory.
    pub fn api_log_directory(self, dir: &Path) -> Result<Self, ekg_error::Error> {
        if dir.exists() {
            let x = self.api_log(true)?;
            x.set_string("api-log.directory", dir.to_str().unwrap())?;
            Ok(x)
        } else {
            tracing::error!(
                "Could not enable logging since directory does not exist: {}",
                dir.to_str().unwrap()
            );
            Ok(self)
        }
    }

    pub fn data_store_type(self, data_store_type: DataStoreType) -> Result<Self, ekg_error::Error> {
        match data_store_type {
            DataStoreType::ParallelNN => self.set_string("type", "parallel-nn")?,
            DataStoreType::ParallelNW => self.set_string("type", "parallel-nw")?,
            DataStoreType::ParallelWW => self.set_string("type", "parallel-ww")?,
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    #[test_log::test]
    fn test_set_param() {
        let params = crate::Parameters::empty().unwrap();
        params.set_string("key1", "value1").unwrap();
        let value = params.get_string("key1", "whatever").unwrap();
        assert_eq!(value, "value1");
    }
}
