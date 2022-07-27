// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{ffi::CString, panic::AssertUnwindSafe, ptr};
use std::ops::Deref;

use indoc::formatdoc;
use iref::{Iri, IriBuf};

use crate::{
    error::Error,
    root::{
        CException,
        CPrefixes,
        CPrefixes_DeclareResult as PrefixDeclareResult,
        CPrefixes_declarePrefix,
        CPrefixes_newDefaultPrefixes,
    },
    DataStoreConnection,
    FactDomain,
    Parameters,
    Statement,
    graph::DEFAULT_GRAPH
};

pub struct Prefixes {
    pub(crate) inner: *mut CPrefixes,
}

impl Prefixes {
    pub fn builder() -> PrefixesBuilder { PrefixesBuilder::default() }

    pub fn default() -> Result<Self, Error> {
        let mut prefixes = Self {
            inner: ptr::null_mut(),
        };
        CException::handle(AssertUnwindSafe(|| unsafe {
            CPrefixes_newDefaultPrefixes(&mut prefixes.inner)
        }))?;
        Ok(prefixes)
    }

    pub fn declare_prefix<'a>(
        &self,
        prefix: &Prefix,
    ) -> Result<PrefixDeclareResult, Error> {
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
                log::error!(
                    "Invalid prefix name \"{}\" while registering namespace \
                     <{}>",
                    prefix.name.as_str(),
                    prefix.iri.as_str()
                );
                Err(Error::InvalidPrefixName)
            },
            PrefixDeclareResult::PREFIXES_DECLARED_NEW => Ok(result),
            PrefixDeclareResult::PREFIXES_NO_CHANGE => {
                log::warn!("Registered {prefix} twice");
                Ok(result)
            },
            _ => {
                log::error!(
                    "Result of registering prefix {prefix} is {:?}",
                    result
                );
                Ok(result)
            },
        }
    }

    pub fn declare<'a, Base: Into<Iri<'a>>>(
        &self,
        name: &str,
        iri: Base,
    ) -> Result<PrefixDeclareResult, Error> {
        self.declare_prefix(&Prefix::declare(name, iri))
    }
}

#[derive(Debug, Clone)]
pub struct Prefix {
    /// assumed to end with ':'
    pub name: String,
    /// assumed to end with either '/' or '#'
    pub iri:  IriBuf,
}

impl std::fmt::Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name.as_str(), self.iri.as_str())
    }
}

impl Prefix {
    pub fn declare<'a, Base: Into<Iri<'a>>>(name: &str, iri: Base) -> Self {
        let iri = iri.into();
        match iri.as_str().chars().last().unwrap() {
            '/' | '#' => {
                Self {
                    name: name.to_string(),
                    iri:  IriBuf::from(iri),
                }
            },
            _ => {
                Self {
                    name: name.to_string(),
                    iri:  IriBuf::from_string(format!("{}/", iri)).unwrap(),
                }
            },
        }
    }

    pub fn declare_from_str(name: &str, iri: &str) -> Self {
        Self::declare(name, Iri::from_str(iri).unwrap())
    }

    pub fn with_local_name(&self, name: &str) -> Result<IriBuf, iref::Error> {
        IriBuf::new(format!("{}{}", self.iri.as_str(), name).as_str())
    }
}

#[derive(Default)]
pub struct PrefixesBuilder {
    prefixes: Vec<Prefix>,
}

impl<'a> PrefixesBuilder {
    pub fn default() -> Self {
        PrefixesBuilder {
            prefixes: Vec::new(),
        }
    }

    pub fn declare_with_name_and_iri<Base: Into<Iri<'a>>>(
        mut self,
        name: &str,
        iri: Base,
    ) -> Self {
        self.prefixes.push(Prefix::declare(name, iri));
        self
    }

    pub fn declare(mut self, prefix: Prefix) -> Self {
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
pub struct Class {
    pub prefix:     Prefix,
    pub local_name: String,
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.prefix.name.as_str(),
            self.local_name.as_str()
        )
    }
}

impl Class {
    pub fn declare(prefix: Prefix, local_name: &str) -> Self {
        Self {
            prefix,
            local_name: local_name.to_string(),
        }
    }

    pub fn number_of_individuals(
        &self,
        ds_connection: &DataStoreConnection,
    ) -> Result<u64, Error> {
        let default_graph = DEFAULT_GRAPH.deref().as_display_iri();
        let prefixes =
            Prefixes::builder().declare(self.prefix.clone()).build()?;
        let count_result = Statement::query(
            &prefixes,
            (formatdoc! {r##"
                SELECT DISTINCT ?thing
                WHERE {{
                    {{
                        GRAPH ?graph {{
                            ?thing a {self}
                        }}
                    }} UNION {{
                            ?thing a {self}
                        BIND({default_graph} AS ?graph)
                    }}
                }}
                "##
            })
            .as_str(),
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
    fn test_a_prefix() -> Result<(), iref::Error> {
        let prefix = Prefix::declare(
            "test:",
            Iri::new("http://whatever.kom/test#").unwrap(),
        );
        let x = prefix.with_local_name("abc")?;

        assert_eq!(x.as_str(), "http://whatever.kom/test#abc");
        Ok(())
    }

    #[test]
    fn test_a_class() {
        let prefix = Prefix::declare(
            "test:",
            Iri::new("http://whatever.com/test#").unwrap(),
        );
        let class = Class::declare(prefix, "SomeClass");
        let s = format!("{:}", class);
        assert_eq!(s, "test:SomeClass")
    }
}
