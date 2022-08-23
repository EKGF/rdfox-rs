// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::path::{Path, PathBuf};

pub static RDFOX_HOME: &str = concat!(env!("HOME"), "/.RDFox");
pub const RDFOX_DEFAULT_LICENSE_FILE_NAME: &str = "RDFox.lic";

pub fn find_license(dir: &Path) -> Result<PathBuf, crate::Error> {
    if dir.exists() {
        let license = dir.join(RDFOX_DEFAULT_LICENSE_FILE_NAME);
        log::debug!("Checking license file {license:?}");
        if license.exists() {
            return Ok(license)
        }
    }
    // Now check home directory ~/.RDFox/RDFox.lic
    //
    let license = PathBuf::from(format!("{RDFOX_HOME}/{RDFOX_DEFAULT_LICENSE_FILE_NAME}"));
    log::debug!("Checking license file {license:?}");
    if license.exists() {
        return Ok(license)
    }

    Err(crate::Error::RDFoxLicenseFileNotFound)
}
