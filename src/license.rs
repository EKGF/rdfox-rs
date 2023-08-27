// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    rdf_store_rs::consts::LOG_TARGET_DATABASE,
    std::path::{Path, PathBuf},
};

pub static RDFOX_HOME: &str = concat!(env!("HOME"), "/.RDFox");
pub const RDFOX_DEFAULT_LICENSE_FILE_NAME: &str = "RDFox.lic";

/// Find the license file in the given directory or in the home directory or
/// check the environment variable RDFOX_LICENSE_CONTENT (which takes
/// precedence).
///
/// If the environment variable RDFOX_LICENSE_CONTENT is set, then the content
/// of the license file is returned as the second element of the tuple.
pub fn find_license(
    dir: Option<&Path>,
) -> Result<(Option<PathBuf>, Option<String>), rdf_store_rs::RDFStoreError> {
    if let Ok(license_content) = std::env::var("RDFOX_LICENSE_CONTENT") {
        tracing::info!(
            target: LOG_TARGET_DATABASE,
            "Using license content from environment variable RDFOX_LICENSE_CONTENT"
        );
        return Ok((None, Some(license_content)))
    }
    if let Some(dir) = dir {
        let license_file_name = dir.join(RDFOX_DEFAULT_LICENSE_FILE_NAME);
        tracing::info!(
            target: LOG_TARGET_DATABASE,
            "Checking license file {license_file_name:?}"
        );
        if license_file_name.exists() {
            return Ok((Some(license_file_name), None))
        }
        // Now check home directory ~/.RDFox/RDFox.lic
        //
        let license_file_name = PathBuf::from(format!(
            "{RDFOX_HOME}/{RDFOX_DEFAULT_LICENSE_FILE_NAME}"
        ));
        tracing::info!(
            target: LOG_TARGET_DATABASE,
            "Checking license file {license_file_name:?}"
        );
        if license_file_name.exists() {
            return Ok((Some(license_file_name), None))
        }
    }

    Err(rdf_store_rs::RDFStoreError::RDFoxLicenseFileNotFound)
}
