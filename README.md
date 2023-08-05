# rdfox-rs

RDFox is a product of [Oxford Semantic Technologies RDFox](https://www.oxfordsemantic.tech/product).

RDFox is a high-performance, scalable and lightweight knowledge graph and semantic reasoning engine.
It supports the storage, querying and reasoning over large-scale ontologies represented in RDF triples.

This crate provides a Rust interface to the RDFox database allowing you to use RDFox as
database engine that is part of your program, no need to run a separate RDFox server (although that is also possible).

## How this crate works

- It downloads the RDFox distribution zip file during build, straight from the vendor's website to your target
  directory.
- It then generates bindings from `CRDFox.h` using bindgen (which requires llvm to be installed)
- It either links to the dynamic link library `libRDFox.dylib` (if you use feature `rdfox-dylib`)
- Or else it links to the static RDFox library `libRDFox-static.a` by default.
- It requires an RDFox license (see <https://www.oxfordsemantic.tech/product>)
  - Copy the license file to `~/.RDFox/RDFox.lic`
- It provides a higher level rust-friendly interface over the RDFox C-API

## Status

- All the basics work
- Proper documentation is still missing, check the [tests/load.rs](tests/load.rs) source code for an example
- Currently only supports RDFox 6.2 and RDFox 6.3
- RDFox itself is a C++ program with a C API that comes as a dynamic link library or a static library,
  both of which are supported by this Rust crate.
  - Use feature `rdfox-dylib` if you want to use the dynamic link library
  - At the moment, the static link library causes a `SIGSEGV` signal when running the tests.
    - This is being investigated.
    - The RDFox API logging does not work when linking with the static library (issue in progress)
  - Therefore, **this crate is not ready for production yet**.
- Has only been tested as an embedded database (meaning: running the whole RDFox database engine in your Rust process),
  however, in theory it should also be possible (with some tweaks that we have to add) to run it just as a client to
  a remote instance of RDFox.

## Plans

- Get the static link library to **not** cause a `SIGSEGV` signal
- Make high-level interface more abstract so that it can also be used for remote endpoints using REST calls
  and potentially any other triple store product.
  - Core components that are RDFox-independent have already been moved to
    the [rdf-store-rs crate](https://crates.io/crates/rdf-store-rs)

## Version

The major/minor version numbers of this crate are used to determine which version of RDFox
needs to be downloaded and used.

## How to run the tests

```shell
RUST_LOG=info cargo test 
```

Or, if you want to see all output:

```shell
RUST_LOG=trace cargo test --package rdfox-rs --test load load_rdfox -- --exact --nocapture
```

If you want to run the tests with the dynamic link library of RDFox, then run this:

```shell
RUST_LOG=trace cargo test --package rdfox-rs --features rdfox-dylib --test load load_rdfox -- --exact --nocapture
```

# Published where?

- Crate: <https://crates.io/crates/rdfox-rs>
- Documentation:
  - docs.rs: <https://docs.rs/rdfox-rs>
  - github: <https://ekgf.github.io/rdfox-rs/rdfox_rs/index.html>
