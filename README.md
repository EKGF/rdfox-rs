# rdfox-rs

Embedded [Oxford Semantic Technologies RDFox](https://www.oxfordsemantic.tech/product) database for Rust programs.

- Downloads RDFox zip file (to your target directory)
- Generates bindings from `CRDFox.h` using bindgen (which requires llvm to be installed)
- Links to dynamic link library `libRDFox.dylib`
- Requires an RDFox license (see https://www.oxfordsemantic.tech/product)
   - Copy license to `~/.RDFox/RDFox.lic` 

## Version

The major/minor version numbers of this crate are used to determine which version of RDFox
needs to be downloaded and used.
