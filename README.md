# rdfox-rs

Embedded [Oxford Semantic Technologies RDFox](https://www.oxfordsemantic.tech/product) database for Rust programs.

- Downloads RDFox zip file (to your target directory)
- Generates bindings from `CRDFox.h` using bindgen (which requires llvm to be installed)
- Links to dynamic link library `libRDFox.dylib`
- Requires an RDFox license (see https://www.oxfordsemantic.tech/product)
  - Copy license to `~/.RDFox/RDFox.lic`
- Provides a higher level rust-friendly interface over the C-API

## Status

- Works with the dynamic link library of RDFox, does not work yet with the static library

## Plans

- Make high-level interface more abstract so that it can also be used for remote endpoints using REST calls
  and potentially any other triple store product.

## Version

The major/minor version numbers of this crate are used to determine which version of RDFox
needs to be downloaded and used.

## How to run the tests

```shell
RUST_LOG=info cargo test 
```

Or, if you want to see all output:

```shell
RUST_LOG=trace cargo test load_rdfox  -- --nocapture
```

If you want to run the tests with the dynamic link library of RDFox, then run this:

```shell
RUST_LOG=trace cargo test --features rdfox-dylib load_rdfox  -- --nocapture
```
