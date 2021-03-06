// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CString;
use std::panic::AssertUnwindSafe;
use std::ptr;
use indoc::formatdoc;

use iref::IriRef;

use crate::error::Error;
use crate::root::{
    CException,
    CPrefixes,
    CPrefixes_declarePrefix,
    CPrefixes_DeclareResult as PrefixDeclareResult,
    CPrefixes_newDefaultPrefixes,
};
use crate::{DataStoreConnection, FactDomain, Parameters, Statement};

pub struct Prefixes {
    pub(crate) inner: *mut CPrefixes,
}

impl Prefixes {
    pub fn builder<'a>() -> PrefixesBuilder<'a> {
        PrefixesBuilder::default()
    }

    pub fn default() -> Result<Self, Error> {
        let mut prefixes = Self {
            inner: ptr::null_mut(),
        };
        CException::handle(AssertUnwindSafe(|| unsafe {
            CPrefixes_newDefaultPrefixes(&mut prefixes.inner)
        }))?;
        Ok(prefixes)
    }

    pub fn declare_prefix<'a>(&self, prefix: &Prefix<'a>) -> Result<PrefixDeclareResult, Error> {
        log::trace!("Register prefix {prefix}");
        let c_name = CString::new(prefix.name.as_str()).unwrap();
        let c_iri = CString::new(prefix.iri.as_str()).unwrap();
        let mut result = PrefixDeclareResult::PREFIXES_NO_CHANGE;
        CException::handle(AssertUnwindSafe(|| unsafe {
            CPrefixes_declarePrefix(
                self.inner,
                c_name.as_ptr(),
                c_iri.as_ptr(),
                &mut result,
            )
        }))?;
        match result {
            PrefixDeclareResult::PREFIXES_INVALID_PREFIX_NAME => {
                log::error!("Invalid prefix name \"{}\" while registering namespace <{}>", prefix.name.as_str(), prefix.iri.as_str());
                Err(Error::InvalidPrefixName)
            },
            PrefixDeclareResult::PREFIXES_DECLARED_NEW => Ok(result),
            PrefixDeclareResult::PREFIXES_NO_CHANGE => {
                log::warn!("Registered {prefix} twice");
                Ok(result)
            },
            _ => {
                log::error!("Result of registering prefix {prefix} is {:?}", result);
                Ok(result)
            }
        }
    }

    pub fn declare<'a>(&self, name: &str, iri: &IriRef<'a>) -> Result<PrefixDeclareResult, Error> {
        self.declare_prefix(&Prefix::declare(name, iri))
    }
}

#[derive(Debug, Clone)]
pub struct Prefix<'a> {
    name: String,       // assumed to end with ':'
    iri: IriRef<'a>,    // assumed to end with either '/' or '#'
}

impl<'a> std::fmt::Display for Prefix<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: <{}>", self.name.as_str(), self.iri.as_str())
    }
}

impl<'a> Prefix<'a> {
    pub fn declare(name: &str, iri: &IriRef<'a>) -> Self {
        Self {
            name: name.to_string(),
            iri: *iri,
        }
    }
}

#[derive(Default)]
pub struct PrefixesBuilder<'a> {
    prefixes: Vec<Prefix<'a>>,
}

impl<'a> PrefixesBuilder<'a> {
    pub fn default() -> Self {
        PrefixesBuilder { prefixes: Vec::new() }
    }

    pub fn declare_with_name_and_iri<'b: 'a>(mut self, name: &str, iri: &IriRef<'b>) -> Self {
        self.prefixes.push(Prefix::declare(name, iri));
        self
    }
    pub fn declare(mut self, prefix: Prefix<'a>) -> Self {
        self.prefixes.push(prefix);
        self
    }

    pub fn build(self) -> Result<Prefixes, Error> {
        let to_build = Prefixes::default()?;
        for prefix in self.prefixes {
            to_build.declare_prefix(&prefix)?;
        }
        Ok(to_build)
    }
}


#[derive(Debug, Clone)]
pub struct Class<'a> {
    pub prefix: Prefix<'a>,
    pub local_name: String
}

impl<'a> std::fmt::Display for Class<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.prefix.name.as_str(), self.local_name.as_str())
    }
}

impl<'a> Class<'a> {
    pub fn declare(prefix: Prefix<'a>, local_name: &str) -> Self {
        Self {
            prefix,
            local_name: local_name.to_string()
        }
    }

    pub fn number_of_individuals(&self, ds_connection: &DataStoreConnection) -> Result<u64, Error> {
        let prefixes = Prefixes::builder()
            .declare(self.prefix.clone())
            .build()?;
        let count_result = Statement::query(
            &prefixes,
            (formatdoc! {r##"
                SELECT DISTINCT ?thing
                WHERE {{
                    {{
                        GRAPH ?graph {{
                            ?thing a {class}
                        }}
                    }} UNION {{
                            ?thing a {class}
                        BIND("default" AS ?graph)
                    }}
                }}
                "##,
                class = self
            }).as_str(),
        )?
            .cursor(
                ds_connection,
                &Parameters::empty()?.fact_domain(FactDomain::ALL)?,
            )?
            .count();
        #[allow(clippy::let_and_return)]
        count_result
    }
}



#[cfg(test)]
mod tests {
    use iref::Iri;
    use crate::{Class, Prefix};

    #[test]
    fn test_a_class() {
        let prefix = Prefix::declare("test:", &Iri::new("http://whatever.com/test#").unwrap());
        let class = Class::declare(prefix, "SomeClass");
        let s = format!("{:}", class);
        assert_eq!(s, "test:SomeClass")
    }
}

