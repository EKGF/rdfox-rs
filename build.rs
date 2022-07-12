// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

// build.rs

extern crate core;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, Write};
use std::option_env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::{env, io};

use lazy_static::lazy_static;

#[cfg(target_os = "macos")]
const RDFOX_OS_NAME: &str = "macOS";
#[cfg(target_os = "linux")]
const RDFOX_OS_NAME: &str = "linux";
#[cfg(target_os = "windows")]
const RDFOX_OS_NAME: &str = "win64";

const ARCH: &str = env::consts::ARCH;

lazy_static! {
    static ref RDFOX_DOWNLOAD_HOST: &'static str = option_env!("RDFOX_DOWNLOAD_HOST")
        .unwrap_or("https://rdfox-distribution.s3.eu-west-2.amazonaws.com/release");
    static ref RDFOX_VERSION_EXPECTED: &'static str =
        option_env!("RDFOX_VERSION_EXPECTED").unwrap_or("5.6");
}

fn rdfox_download_url() -> String {
    let host = *RDFOX_DOWNLOAD_HOST;
    let version = *RDFOX_VERSION_EXPECTED;
    let os = RDFOX_OS_NAME;

    format!("{host}/v{version}/RDFox-{os}-{ARCH}-{version}.zip")
}

fn rdfox_archive_name() -> String {
    let version = *RDFOX_VERSION_EXPECTED;
    format!("RDFox-{RDFOX_OS_NAME}-{ARCH}-{version}")
}

fn rdfox_download_file() -> PathBuf {
    format!(
        "{}/{}.zip",
        env::var("OUT_DIR").unwrap(),
        rdfox_archive_name()
    )
    .into()
}

fn rdfox_dylib_dir() -> PathBuf {
    format!(
        "{}/{}/lib",
        env::var("OUT_DIR").unwrap(),
        rdfox_archive_name()
    )
    .into()
}

fn rdfox_header_dir() -> PathBuf {
    format!(
        "{}/{}/include",
        env::var("OUT_DIR").unwrap(),
        rdfox_archive_name()
    )
    .into()
}

fn download_rdfox() -> Result<PathBuf, curl::Error> {
    println!("cargo:rerun-if-env-changed=RDFOX_DOWNLOAD_HOST");
    println!("cargo:rerun-if-env-changed=RDFOX_VERSION_EXPECTED");

    let mut curl = curl::easy::Easy::new();
    let url = rdfox_download_url();
    let file_name = rdfox_download_file();

    if file_name.exists() {
        println!(
            "cargo:warning=\"RDFox has already been downloaded: {}\"",
            file_name.to_str().unwrap()
        );
        return Ok(file_name);
    }

    curl.url(url.as_str())?;
    curl.verbose(false)?;
    curl.progress(false)?;
    let _redirect = curl.follow_location(true);

    let mut buffer = Vec::new();
    {
        let mut transfer = curl.transfer();
        transfer
            .write_function(|data| {
                buffer.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    {
        let mut file = File::create(file_name.to_str().unwrap()).unwrap_or_else(|_err| {
            panic!(
                "cargo:warning=\"Could not create {}\"",
                file_name.to_str().unwrap()
            )
        });
        file.write_all(buffer.as_slice()).unwrap_or_else(|_err| {
            panic!(
                "cargo:warning=\"Could not write to {}\"",
                file_name.to_str().unwrap()
            )
        });
        println!(
            "cargo:warning=\"Downloaded RDFox: {}\"",
            file_name.to_str().unwrap()
        );
    }
    Ok(file_name)
}

fn unzip_rdfox(zip_file: PathBuf, archive_name: String) -> PathBuf {
    let dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let file = File::open(zip_file.clone()).unwrap();
    let reader = BufReader::new(file);

    let mut zip = zip::ZipArchive::new(reader).unwrap_or_else(|_err| {
        panic!(
            "cargo:warning=\"Could not open zip archive: {}\"",
            zip_file.to_str().unwrap()
        )
    });

    zip.extract(dir.clone()).unwrap_or_else(|_err| {
        panic!(
            "cargo:warning=\"Could not unzip {}\"",
            zip_file.to_str().unwrap()
        )
    });

    let unpacked_dir = dir.join(archive_name);

    if !unpacked_dir.exists() {
        panic!(
            "cargo:warning=\"Unpacked directory does not exist: {}\"",
            unpacked_dir.to_str().unwrap()
        );
    }

    unpacked_dir
}

fn set_llvm_path(llvm_config_path: &Path) {
    env::set_var(
        "LLVM_CONFIG_PATH",
        format!("{:}", llvm_config_path.display()),
    );
    println!(
        "cargo:rustc-env=LLVM_CONFIG_PATH={:}",
        llvm_config_path.display()
    );
    if let Some(path) = option_env!("PATH") {
        env::set_var(
            "PATH",
            format!("{}:{}/bin", path, llvm_config_path.display()),
        );
    }
}

fn add_llvm_path() {
    if let Some(llvm_config_path) = option_env!("LLVM_CONFIG_PATH") {
        set_llvm_path(PathBuf::from(llvm_config_path.trim()).as_path());
        return;
    }

    let brew_llvm_dir = PathBuf::from("/usr/local/opt/llvm");
    if brew_llvm_dir.exists() {
        set_llvm_path(brew_llvm_dir.as_path());
    }

    let llvm_config_path = Command::new("llvm-config")
        .args(["--prefix"])
        .output()
        .expect("`llvm-config` must be in PATH")
        .stdout;
    let llvm_config_path =
        String::from_utf8(llvm_config_path).expect("`llvm-config --prefix` output must be UTF-8");
    env::set_var(
        "LLVM_CONFIG_PATH",
        format!("{}/bin/llvm-config", llvm_config_path.trim()),
    );
    println!("cargo:rustc-env=LLVM_CONFIG_PATH={}", llvm_config_path);
}

//
// The CRDFox.h file misses the `#include <cstddef>` statement which is
// needed to define the symbol `nullptr_t`. This is only an issue on Linux,
// things compile fine on Darwin.
//
fn write_workaround_header<P: AsRef<Path>>(workaround_h: P) -> io::Result<()> {
    fn create_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.as_ref())?;
        Ok(file)
    }

    let mut file = create_file(workaround_h)?;

    writeln!(
        file,
        "namespace std {{ typedef decltype(nullptr) nullptr_t; }}"
    )?;
    writeln!(
        file,
        "typedef decltype(nullptr) nullptr_t;"
    )?;

    Ok(())
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let workaround_h = PathBuf::from(format!("{}/workaround.h", out_path.display()));

    write_workaround_header(&workaround_h).expect("cargo:warning=Could not generate workaround.h");

    println!("cargo:rerun-if-changed=build.rs");

    let file_name = download_rdfox().expect("cargo:warning=Could not download RDFox");
    unzip_rdfox(file_name, rdfox_archive_name());

    // Tell cargo to look for shared libraries in the specified directory
    println!(
        "cargo:rustc-link-search={}",
        rdfox_dylib_dir().to_str().unwrap()
    );

    // Tell cargo to tell rustc to link the libRDFox.dylib shared library.
    println!("cargo:rustc-link-lib=dylib=RDFox");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    // println!("cargo:rerun-if-changed=wrapper.h");

    add_llvm_path();

    // "-x" and "c++" must be separate due to a bug
    let clang_args: Vec<&str> = vec!["-x", "c++", "-std=c++17"];

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(workaround_h.to_str().unwrap())
        .header(format!(
            "{}/{}/include/CRDFox.h",
            out_path.display(),
            rdfox_archive_name()
        ))
        .opaque_type("void")
        .clang_args(clang_args)
        .clang_arg(format!("-I/{}", rdfox_header_dir().to_str().unwrap()))
        .clang_arg("-v")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        // .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_bindings(true)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        .enable_cxx_namespaces()
        .respect_cxx_access_specs(true)
        .vtable_generation(true)
        .enable_function_attribute_detection()
        .generate_inline_functions(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
