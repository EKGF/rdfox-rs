// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
//! build.rs
#![feature(absolute_path)]

extern crate core;

use {
    lazy_static::lazy_static,
    std::{
        env,
        fs::File,
        io::{BufReader, Write},
        option_env,
        path::{Path, PathBuf},
        process::Command,
    },
};

#[cfg(target_os = "macos")]
const RDFOX_OS_NAME: &str = "macOS";
#[cfg(target_os = "linux")]
const RDFOX_OS_NAME: &str = "linux";
#[cfg(target_os = "windows")]
const RDFOX_OS_NAME: &str = "win64";

const ARCH: &str = env::consts::ARCH;

const BLOCKLIST_ITEMS: &[&str] = &[
    "std::integral_constant_value_type",
    "std::remove_const_type",
    "std::remove_volatile_type",
    "^std::value$",
    "^_Tp$",
];
const RUSTFMT_CONFIG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.rustfmt.toml");

lazy_static! {
    static ref RDFOX_DOWNLOAD_HOST: &'static str = option_env!("RDFOX_DOWNLOAD_HOST")
        .unwrap_or("https://rdfox-distribution.s3.eu-west-2.amazonaws.com/release");
    static ref RDFOX_VERSION_EXPECTED: &'static str =
        option_env!("RDFOX_VERSION_EXPECTED").unwrap_or("6.0");
}

fn rdfox_download_url() -> String {
    let _host = *RDFOX_DOWNLOAD_HOST;
    let _version = *RDFOX_VERSION_EXPECTED;
    let _os = RDFOX_OS_NAME;

    format!("{_host}/v{_version}/RDFox-{_os}-{ARCH}-{_version}.zip")
}

fn rdfox_archive_name() -> String {
    let version = *RDFOX_VERSION_EXPECTED;
    format!("RDFox-{RDFOX_OS_NAME}-{ARCH}-{version}")
}

fn rdfox_download_file() -> PathBuf {
    let dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    dir.parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!("{}.zip", rdfox_archive_name()))
}

fn rdfox_lib_dir() -> PathBuf {
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

    if file_name.try_exists().unwrap_or_else(|_| {
        panic!(
            "cargo:warning=Can't check existence of file {}",
            file_name.to_str().unwrap()
        )
    }) {
        println!(
            "cargo:warning=\"RDFox has already been downloaded: {}\"",
            file_name.to_str().unwrap()
        );
        return Ok(file_name)
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

fn set_llvm_config_path<P>(llvm_config_path: P) -> PathBuf
where P: AsRef<Path> {
    let path = llvm_config_path.as_ref();
    assert!(llvm_config_path.as_ref().exists());
    let path = std::fs::canonicalize(path).unwrap();
    println!(
        "cargo:warning=llvm config path is {}",
        path.display()
    );
    println!(
        "cargo:rustc-env=LLVM_CONFIG_PATH={:}",
        path.display()
    );
    llvm_config_path.as_ref().into()
}

fn add_llvm_path() {
    let path = env!("PATH");
    let opt_llvm_dir = PathBuf::from("/usr/local/opt/llvm");

    let llvm_path = if let Some(llvm_config_path) = option_env!("LLVM_CONFIG_PATH") {
        set_llvm_config_path(PathBuf::from(llvm_config_path.trim()).as_path())
    } else if let Some(llvm_path) = option_env!("LLVM_PATH") {
        set_llvm_config_path(PathBuf::from(llvm_path).as_path())
    } else if opt_llvm_dir.exists() {
        set_llvm_config_path(opt_llvm_dir.as_path())
    } else if let Some(brew_llvm_dir) = check_llvm_via_brew() {
        set_llvm_config_path(brew_llvm_dir)
    } else {
        panic!("Could not find the LLVM path")
    };

    let llvm_config_path = Command::new("llvm-config")
        .env(
            "PATH",
            format!("{}:{}/bin", path, llvm_path.display()),
        )
        .args(["--prefix"])
        .output()
        .expect("`llvm-config` must be in PATH")
        .stdout;
    let llvm_config_path =
        String::from_utf8(llvm_config_path).expect("`llvm-config --prefix` output must be UTF-8");
    let llvm_config_path = llvm_config_path.trim();
    println!("cargo:rustc-env=LLVM_CONFIG_PATH={llvm_config_path}");
    println!("cargo:rustc-link-search={llvm_config_path}/lib/c++");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn check_llvm_via_brew() -> Option<PathBuf> {
    if let Ok(output) = Command::new("brew").args(["--prefix", "llvm"]).output() {
        let llvm_path =
            String::from_utf8(output.stdout).expect("`brew --prefix llvm` output must be UTF-8");
        Some(PathBuf::from(llvm_path))
    } else {
        None
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn check_llvm_via_brew() -> Option<PathBuf> { None }

// The CRDFox.h file misses the `#include <cstddef>` statement which is
// needed to define the symbol `nullptr_t`. This is only an issue on Linux,
// things compile fine on Darwin.
//
// fn write_workaround_header<P: AsRef<Path>>(workaround_h: P) -> io::Result<()>
// {     fn create_file<P: AsRef<Path>>(path: P) -> io::Result<File> {
//         let file = OpenOptions::new()
//             .write(true)
//             .truncate(true)
//             .create(true)
//             .open(path.as_ref())?;
//         Ok(file)
//     }
//
//     let mut file = create_file(workaround_h)?;
//
//     writeln!(
//         file,
//         "namespace std {{ typedef decltype(nullptr) nullptr_t; }}"
//     )?;
//     writeln!(file, "typedef decltype(nullptr) nullptr_t;")?;
//
//     Ok(())
// }

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    add_llvm_path();

    let file_name = download_rdfox().expect("cargo:warning=Could not download RDFox");
    unzip_rdfox(file_name, rdfox_archive_name());

    // Tell cargo to look for shared libraries in the specified directory
    println!(
        "cargo:rustc-link-search={}",
        rdfox_lib_dir().to_str().unwrap()
    );

    // Tell cargo to tell rustc to link the libRDFox.dylib shared library.
    #[cfg(feature = "rdfox-dylib")]
    println!("cargo:rustc-link-lib=dylib=RDFox");

    // Tell cargo to tell rustc to link the libRDFox.a static library.
    #[cfg(not(feature = "rdfox-dylib"))]
    println!("cargo:rustc-link-lib=static=RDFox-static");
    #[cfg(not(feature = "rdfox-dylib"))]
    println!("cargo:rustc-link-lib=static=c++");
    #[cfg(not(feature = "rdfox-dylib"))]
    println!("cargo:rustc-link-lib=static=c++abi");

    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!(
            "{}/{}/include/CRDFox/CRDFox.h",
            out_path.display(),
            rdfox_archive_name()
        ))
        .generate_comments(true)
        .opaque_type("void")
        .clang_arg(r"-xc++")
        .clang_arg(r"-std=c++17")
        .clang_arg(format!("-I{}", rdfox_header_dir().to_str().unwrap()))
        .clang_arg("-v")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        // .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_configuration_file(Some(PathBuf::from(RUSTFMT_CONFIG)))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        .enable_cxx_namespaces()
        .size_t_is_usize(true)
        .respect_cxx_access_specs(true)
        .vtable_generation(true)
        .enable_function_attribute_detection()
        .generate_inline_functions(true);

    for item in BLOCKLIST_ITEMS {
        builder = builder.blocklist_type(item);
        builder = builder.blocklist_item(item);
        builder = builder.blocklist_function(item);
    }

    // Finish the builder and generate the bindings.
    let bindings = builder
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
