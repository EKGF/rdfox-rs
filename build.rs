// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
//! build.rs
#![feature(absolute_path)]

extern crate core;

use {
    bindgen::{CodegenConfig, RustTarget},
    lazy_static::lazy_static,
    std::{
        env,
        fs::File,
        io::{BufReader, Write},
        option_env,
        path::PathBuf,
        process::Command,
    },
};

const ARCH: &str = env::consts::ARCH;
const FAMILY: &str = env::consts::FAMILY;
const OS: &str = env::consts::OS;

fn rdfox_os_name() -> &'static str {
    match env::var("TARGET").ok().as_deref() {
        Some("x86_64-unknown-linux-gnu") => return "linux",
        Some("x86_64-apple-darwin") => return "macOS",
        Some("aarch64-apple-darwin") => return "macOS",
        Some("x86_64-pc-windows-msvc") => return "windows",
        _ => (),
    }
    if FAMILY == "windows" {
        FAMILY
    } else if OS == "macos" {
        "macOS"
    } else if OS == "linux" {
        "linux"
    } else {
        panic!("Unknown OS: {}", OS);
    }
}

const BLOCK_LIST_ITEMS: &[&str] = &[
    "std::integral_constant_value_type",
    "std::remove_const_type",
    "std::remove_volatile_type",
    "std::.*",
    "^std::value$",
    "^__.*",
    "^size_t.*",
    "^user_.*",
    "^uint_.*",
    "^int_.*",
    "^fpos_t.*",
    "^FILE.*",
    "^off_t.*",
    "^ssize_t.*",
    "^syscall.*",
    "^fget.*",
    "^getline.*",
    "^tempnam.*",
    "^fopen.*",
    "^fput.*",
    "^clearerr.*",
    "^fclose.*",
    "^feof.*",
    "^fread.*",
    "^ferror.*",
    "^fflush.*",
    "^freopen.*",
    "^fscanf.*",
    "^fseek.*",
    "^fsetpos.*",
    "^ftell.*",
    "^fwrite.*",
    "^getc.*",
    "^putc.*",
    "^rewind.*",
    "^setbuf.*",
    "^setvbuf.*",
    "^tmpfile.*",
    "^ungetc.*",
    "^fdopen.*",
    "^fileno.*",
    "^pclose.*",
    "^popen.*",
    "^flockfile.*",
    "^ftrylockfile.*",
    "^funlockfile.*",
    "^getc_unlocked.*",
    "^putc_unlocked.*",
    "^getw.*",
    "^putw.*",
    "^fseeko.*",
    "^ftello.*",
    "^getdelim.*",
    "^fmemopen.*",
    "^open_memstream.*",
    "^fpurge.*",
    "^funopen.*",
    "^setlinebuf.*",
    "^setbuffer.*",
    "^fgetln.*",
    "__va_list_tag",
    "__builtin_va_list",
    "__darwin_va_list",
    "^va_list",
    ".*printf",
    "^vs.*",
    "^vf.*",
    "^sn.*",
    "^u_.*",
    "^__darwin_.*",
    "__darwin_pthread_handler_rec",
    "__std.*",
    "_opaque.*",
    "^_Tp$",
];
const ALLOW_LIST_ITEMS: &[&str] = &["^RDFOX_.*", "^C.*"];
// const ALLOW_LIST_ITEMS: &[&str] = &[".*"];

const RUSTFMT_CONFIG: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.rustfmt.toml");

lazy_static! {
    static ref RDFOX_DOWNLOAD_HOST: &'static str = option_env!("RDFOX_DOWNLOAD_HOST")
        .unwrap_or("https://rdfox-distribution.s3.eu-west-2.amazonaws.com/release");
}
#[cfg(feature = "rdfox-6-2")]
lazy_static! {
    static ref RDFOX_VERSION_EXPECTED: &'static str =
        option_env!("RDFOX_VERSION_EXPECTED").unwrap_or("6.2");
}
#[cfg(feature = "rdfox-6-3a")]
lazy_static! {
    static ref RDFOX_VERSION_EXPECTED: &'static str =
        option_env!("RDFOX_VERSION_EXPECTED").unwrap_or("6.3a");
}
#[cfg(feature = "rdfox-6-3b")]
lazy_static! {
    static ref RDFOX_VERSION_EXPECTED: &'static str =
        option_env!("RDFOX_VERSION_EXPECTED").unwrap_or("6.3b");
}
#[cfg(feature = "rdfox-7-0")]
lazy_static! {
    static ref RDFOX_VERSION_EXPECTED: &'static str =
        option_env!("RDFOX_VERSION_EXPECTED").unwrap_or("7.0");
}
#[cfg(not(any(feature = "rdfox-6-2", feature = "rdfox-6-3a", feature = "rdfox-6-3b", feature = "rdfox-7-0")))]
compile_error!("You have to at least specify one of the rdfox-X-Y version number features");

fn rdfox_download_url() -> String {
    let host = *RDFOX_DOWNLOAD_HOST;
    let version = *RDFOX_VERSION_EXPECTED;
    let os = rdfox_os_name();
    let arch = if ARCH == "aarch64" { "arm64" } else { ARCH };

    format!("{host}/v{version}/RDFox-{os}-{arch}-{version}.zip")
}

// noinspection RsExternalLinter
fn rdfox_archive_name() -> String {
    let version = *RDFOX_VERSION_EXPECTED;
    let os = rdfox_os_name();
    let arch = if ARCH == "aarch64" { "arm64" } else { ARCH };

    format!("RDFox-{os}-{arch}-{version}")
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

    // println!(
    //     "cargo:warning=\"TARGET: {}\"",
    //     env::var("TARGET").ok().unwrap_or("not set".to_string())
    // );

    let mut curl = curl::easy::Easy::new();
    let url = rdfox_download_url();

    let file_name = rdfox_download_file();

    if file_name.try_exists().unwrap_or_else(|_| {
        panic!(
            "cargo:warning=Can't check existence of file {}",
            file_name.to_str().unwrap()
        )
    }) {
        // println!(
        //     "cargo:warning=\"RDFox has already been downloaded: {}\"",
        //     file_name.to_str().unwrap()
        // );
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

fn set_clang_path<S: Into<String>>(path: Option<S>) -> Option<(PathBuf, PathBuf)> {
    if path.is_none() {
        return None;
    }
    path.as_ref()?;
    let path = PathBuf::from(path.unwrap().into());
    if !path.exists() {
        return None;
    }
    let path = std::fs::canonicalize(path).unwrap();

    let mut clang_bin = path.join("bin/clang");
    if !clang_bin.exists() {
        clang_bin = path.join("clang");
    }
    if clang_bin.exists() {
        println!(
            "cargo:warning=using {}",
            clang_bin.display()
        );
    }
    Some((path, clang_bin))
}

fn set_llvm_config_path<S: Into<String>>(path: Option<S>) -> Option<(PathBuf, PathBuf)> {
    if path.is_none() {
        return None;
    }
    path.as_ref()?;
    let path = PathBuf::from(path.unwrap().into());
    if !path.exists() {
        return None;
    }
    let path = std::fs::canonicalize(path).unwrap();

    let mut llvm_config_bin = path.join("bin/llvm-config");
    if !llvm_config_bin.exists() {
        llvm_config_bin = path.join("llvm-config");
    }
    if llvm_config_bin.exists() {
        println!(
            "cargo:warning=using {}",
            llvm_config_bin.display()
        );
    }
    Some((path, llvm_config_bin))
}

fn add_clang_path() {
    let (clang_path, _clang_bin) = set_clang_path(option_env!("LIBCLANG_PATH"))
        .or_else(|| set_clang_path(option_env!("HOMEBREW_PREFIX")))
        .or_else(|| set_clang_path(Some("/opt/homebrew")))
        .unwrap_or_else(|| panic!("Could not find the clang path"));

    let clang_path = clang_path.to_str().unwrap();

    println!("cargo:rustc-env=LIBCLANG_PATH={:}", clang_path);
    println!("cargo:rustc-link-search=native={:}/lib", clang_path);
    println!(
        "cargo:rustc-link-search=native={:}/lib/c++",
        clang_path
    );
}

fn add_llvm_path() {
    let (_, llvm_config_bin) = set_llvm_config_path(option_env!("LLVM_CONFIG_PATH"))
        .or_else(|| set_llvm_config_path(option_env!("LLVM_PATH")))
        .or_else(|| set_llvm_config_path(check_llvm_via_brew()))
        .or_else(|| set_llvm_config_path(Some("/usr/local/opt/llvm")))
        .or_else(|| set_llvm_config_path(Some("/usr/bin")))
        .unwrap_or_else(|| panic!("Could not find the LLVM path"));

    // Now get the REAL llvm path from the llvm-config utility itself
    let llvm_config_path = Command::new(llvm_config_bin)
        // .env(
        //     "PATH",
        //     format!(
        //         "{}:~/llvm/build/bin:{}/bin",
        //         env!("PATH"),
        //         llvm_config_path.display()
        //     ),
        // )
        .args(["--prefix"])
        .output()
        .expect("`llvm-config` must be in PATH")
        .stdout;
    let llvm_config_path =
        String::from_utf8(llvm_config_path).expect("`llvm-config --prefix` output must be UTF-8");
    let llvm_config_path = llvm_config_path.trim();

    println!("cargo:rustc-env=LLVM_CONFIG_PATH={llvm_config_path}");
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn check_homebrew_prefix() -> Option<PathBuf> {
    if let Some(path) = option_env!("HOMEBREW_PREFIX") {
        if let Ok(path) = PathBuf::from(path).canonicalize() {
            return Some(path);
        }
    }
    if let Ok(path) = PathBuf::from("/opt/homebrew").canonicalize() {
        return Some(path);
    }
    if let Ok(output) = Command::new("brew").args(["--prefix"]).output() {
        let homebrew_prefix =
            String::from_utf8(output.stdout).expect("`brew --prefix` output must be UTF-8");
        if let Ok(path) = PathBuf::try_from(homebrew_prefix) {
            return Some(path);
        }
    }
    None
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn check_llvm_via_brew() -> Option<String> {
    if let Some(homebrew_prefix) = check_homebrew_prefix() {
        if homebrew_prefix.join("opt/llvm").exists() {
            return Some(homebrew_prefix.join("opt/llvm").to_str().unwrap().to_owned());
        }
    }
    if let Ok(output) = Command::new("brew").args(["--prefix", "llvm"]).output() {
        let llvm_path =
            String::from_utf8(output.stdout).expect("`brew --prefix llvm` output must be UTF-8");
        Some(llvm_path)
    } else {
        None
    }
}

#[cfg(not(feature = "rdfox-dylib"))]
fn check_lib_iconv() -> Option<String> {
    if let Some(homebrew_path) = check_homebrew_prefix() {
        if let Ok(path) = homebrew_path.join("opt/libiconv/lib/").canonicalize() {
            if let Some(path_str) = path.to_str() {
                println!(
                    "cargo:rustc-link-lib=static:+whole-archive,-bundle,+verbatim={path_str}/libiconv.a"
                );
                println!("cargo:rustc-link-search=native={path_str}");
                return Some(path_str.to_owned());
            }
            println!("cargo:warning=libiconv path is not valid: {:?}", path);
        }
    }
    println!("cargo:warning=libiconv not found");
    None
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn check_llvm_via_brew() -> Option<String> { None }

#[cfg(not(feature = "rdfox-dylib"))]
fn check_cpp() {
    if let Some(homebrew_path) = check_homebrew_prefix() {
        if let Ok(path) = homebrew_path.join("opt/llvm/c++/libc++.a").canonicalize() {
            if let Some(path_str) = path.to_str() {
                println!(
                    "cargo:rustc-link-lib=static:+whole-archive,-bundle,+verbatim={path_str}"
                );
            } else {
                println!("cargo:warning=libc++ path is not valid: {:?}", path);
            }
        }
        if let Ok(path) = homebrew_path.join("opt/llvm/c++/libc++abi.a").canonicalize() {
            if let Some(path_str) = path.to_str() {
                println!(
                    "cargo:rustc-link-lib=static:+whole-archive,-bundle,+verbatim={path_str}"
                );
            } else {
                println!("cargo:warning=libc++abi path is not valid: {:?}", path);
            }
        }
    } else {
        println!("cargo:warning=homebrew not found");
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn check_llvm_via_brew() -> Option<String> { None }

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
    add_clang_path();

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
    {
        println!("cargo:rustc-link-lib=static:+whole-archive,-bundle=RDFox-static");
        check_cpp();
        check_lib_iconv();
    }

    if let Ok(_fmt_config) = PathBuf::from(RUSTFMT_CONFIG).canonicalize() {
        // println!("cargo:warning=Found rustfmt configuration file: {}", _fmt_config.to_str().unwrap());
    } else {
        println!("cargo:warning=Could not find rustfmt configuration file: {}", RUSTFMT_CONFIG);
    }

    let mut codegen_config = CodegenConfig::all();
    codegen_config.remove(CodegenConfig::CONSTRUCTORS);
    codegen_config.remove(CodegenConfig::DESTRUCTORS);
    codegen_config.remove(CodegenConfig::METHODS);

    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!(
            "{}/{}/include/CRDFox/CRDFox.h",
            out_path.display(),
            rdfox_archive_name()
        ))
        .rust_target(RustTarget::Nightly)
        .generate_comments(true)
        .opaque_type("void")
        .opaque_type("std::.*")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .translate_enum_integer_types(true)
        .clang_arg(r"-xc++")
        .clang_arg(r"-std=c++14")
        .clang_arg(r"-static-libstdc++")
        .clang_arg(format!("-I{}", rdfox_header_dir().to_str().unwrap()))
        .clang_arg("-v")
        // .clang_arg(r"-Wl,--whole-archive RDFox-static -Wl,--no-whole-archive")
        // .emit_builtins()
        .layout_tests(true)
        .enable_function_attribute_detection()
        .derive_default(false)
        .with_codegen_config(codegen_config)
        .no_copy(".*CCursor.*")
        .no_copy(".*COutputStream.*")
        .no_copy(".*CException.*")
        .no_copy(".*CInputStream.*")
        .no_copy(".*CParameters.*")
        .no_copy(".*CPrefixes.*")
        .no_copy(".*CServerConnection.*")
        .no_copy(".*CDataStoreConnection.*")
        // .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .rustfmt_configuration_file(Some(PathBuf::from(RUSTFMT_CONFIG)))
        .array_pointers_in_arguments(true)
        .detect_include_paths(true)
        .prepend_enum_name(false)
        .size_t_is_usize(true)
        .translate_enum_integer_types(true)
        .sort_semantically(true)
        .respect_cxx_access_specs(true)
        .generate_inline_functions(true)
        .vtable_generation(false)
        .merge_extern_blocks(true)
        .wrap_unsafe_ops(false);

    for item in BLOCK_LIST_ITEMS {
        builder = builder.blocklist_type(item);
        builder = builder.blocklist_item(item);
        builder = builder.blocklist_function(item);
    }
    for item in ALLOW_LIST_ITEMS {
        builder = builder.allowlist_type(item);
        builder = builder.allowlist_var(item);
        builder = builder.allowlist_function(item);
    }

    // let command_line_flags = builder.command_line_flags();
    // for flag in &command_line_flags {
    //     println!("cargo:warning={flag}");
    // }

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
