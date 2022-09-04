// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{collections::HashMap, ffi::CString, ops::Deref, ptr};

use iref::{Iri, IriBuf};

use crate::{
    database_call,
    error::Error,
    namespace::{PREFIX_RDF, PREFIX_RDFS},
    root::{
        CPrefixes,
        CPrefixes_DeclareResult as PrefixDeclareResult,
        CPrefixes_declarePrefix,
        CPrefixes_newDefaultPrefixes,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct Prefixes {
    pub(crate) inner: *mut CPrefixes,
    map:              HashMap<String, Prefix>,
}

impl std::fmt::Display for Prefixes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _prefix in self.map.values() {
            writeln!(f, "PREFIX {_prefix}")?
        }
        Ok(())
    }
}

impl Prefixes {
    pub fn builder() -> PrefixesBuilder { PrefixesBuilder::default() }

    pub fn empty() -> Result<Self, Error> {
        let mut prefixes = Self {
            inner: ptr::null_mut(),
            map:   HashMap::new(),
        };
        database_call!(
            "allocating prefixes",
            CPrefixes_newDefaultPrefixes(&mut prefixes.inner)
        )?;
        Ok(prefixes)
    }

    /// Return the RDF and RDFS prefixes
    pub fn default() -> Result<Self, Error> {
        Self::empty()?
            .add_prefix(PREFIX_RDF.deref())?
            .add_prefix(PREFIX_RDFS.deref())
    }

    pub fn declare_prefix<'a>(&mut self, prefix: &Prefix) -> Result<PrefixDeclareResult, Error> {
        log::trace!("Register prefix {prefix}");
        if let Some(_already_registered) = self.map.insert(prefix.name.clone(), prefix.clone()) {
            return Ok(PrefixDeclareResult::PREFIXES_NO_CHANGE)
        }
        let c_name = CString::new(prefix.name.as_str()).unwrap();
        let c_iri = CString::new(prefix.iri.as_str()).unwrap();
        let mut result = PrefixDeclareResult::PREFIXES_NO_CHANGE;
        database_call!(
            "declaring a prefix",
            CPrefixes_declarePrefix(self.inner, c_name.as_ptr(), c_iri.as_ptr(), &mut result)
        )?;
        match result {
            PrefixDeclareResult::PREFIXES_INVALID_PREFIX_NAME => {
                log::error!(
                    "Invalid prefix name \"{}\" while registering namespace <{}>",
                    prefix.name.as_str(),
                    prefix.iri.as_str()
                );
                Err(Error::InvalidPrefixName)
            },
            PrefixDeclareResult::PREFIXES_DECLARED_NEW => Ok(result),
            PrefixDeclareResult::PREFIXES_NO_CHANGE => {
                log::debug!("Registered {prefix} twice");
                Ok(result)
            },
            _ => {
                log::error!("Result of registering prefix {prefix} is {:?}", result);
                Ok(result)
            },
        }
    }

    pub fn declare<'a, Base: Into<Iri<'a>>>(
        &mut self,
        name: &str,
        iri: Base,
    ) -> Result<PrefixDeclareResult, Error> {
        self.declare_prefix(&Prefix::declare(name, iri))
    }

    pub fn add_prefix(mut self, prefix: &Prefix) -> Result<Self, Error> {
        self.declare_prefix(prefix).map(|_result| self)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

    pub fn declare_with_name_and_iri<Base: Into<Iri<'a>>>(mut self, name: &str, iri: Base) -> Self {
        self.prefixes.push(Prefix::declare(name, iri));
        self
    }

    pub fn declare(mut self, prefix: Prefix) -> Self {
        self.prefixes.push(prefix);
        self
    }

    pub fn build(self) -> Result<Prefixes, Error> {
        let mut to_build = Prefixes::empty()?;
        for prefix in self.prefixes {
            to_build.declare_prefix(&prefix)?;
        }
        Ok(to_build)
    }
}

#[cfg(test)]
mod tests {
    use iref::Iri;

    use crate::Prefix;

    #[test]
    fn test_a_prefix() -> Result<(), iref::Error> {
        let prefix = Prefix::declare("test:", Iri::new("http://whatever.kom/test#").unwrap());
        let x = prefix.with_local_name("abc")?;

        assert_eq!(x.as_str(), "http://whatever.kom/test#abc");
        Ok(())
    }
}
