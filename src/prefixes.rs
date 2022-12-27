// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{
    collections::HashMap,
    ffi::CString,
    ops::Deref,
    ptr,
    sync::{Arc, Mutex},
};

use iref::{Iri, IriBuf};

use crate::{
    database_call,
    error::Error,
    namespace::{PREFIX_OWL, PREFIX_RDF, PREFIX_RDFS, PREFIX_XSD},
    root::{
        CPrefixes,
        CPrefixes_DeclareResult as PrefixDeclareResult,
        CPrefixes_declarePrefix,
        CPrefixes_destroy,
        CPrefixes_newDefaultPrefixes,
    },
    Class,
    Predicate,
};

#[derive(Debug)]
pub struct Prefixes {
    inner: *mut CPrefixes,
    map:   Mutex<HashMap<String, Prefix>>,
}

impl PartialEq for Prefixes {
    fn eq(&self, other: &Self) -> bool { self.c_ptr() == other.c_ptr() }
}

impl Eq for Prefixes {}

unsafe impl Send for Prefixes {}

unsafe impl Sync for Prefixes {}

impl Drop for Prefixes {
    fn drop(&mut self) { self.destroy() }
}

impl std::fmt::Display for Prefixes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _prefix in self.map.lock().unwrap().values() {
            writeln!(f, "PREFIX {_prefix}")?
        }
        Ok(())
    }
}

impl Prefixes {
    pub fn builder() -> PrefixesBuilder { PrefixesBuilder::default() }

    pub fn empty() -> Result<Arc<Self>, Error> {
        let mut prefixes = Self {
            inner: ptr::null_mut(),
            map:   Mutex::new(HashMap::new()),
        };
        database_call!(
            "allocating prefixes",
            CPrefixes_newDefaultPrefixes(&mut prefixes.inner)
        )?;
        Ok(Arc::new(prefixes))
    }

    /// Return the RDF and RDFS prefixes
    pub fn default() -> Result<Arc<Self>, Error> {
        Self::empty()?
            .add_prefix(PREFIX_RDF.deref())?
            .add_prefix(PREFIX_RDFS.deref())?
            .add_prefix(PREFIX_OWL.deref())?
            .add_prefix(PREFIX_XSD.deref())
    }

    pub fn declare_prefix(self: &Arc<Self>, prefix: &Prefix) -> Result<PrefixDeclareResult, Error> {
        tracing::trace!("Register prefix {prefix}");
        if let Some(_already_registered) = self
            .map
            .lock()
            .unwrap()
            .insert(prefix.name.clone(), prefix.clone())
        {
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
                tracing::error!(
                    "Invalid prefix name \"{}\" while registering namespace <{}>",
                    prefix.name.as_str(),
                    prefix.iri.as_str()
                );
                Err(Error::InvalidPrefixName)
            },
            PrefixDeclareResult::PREFIXES_DECLARED_NEW => Ok(result),
            PrefixDeclareResult::PREFIXES_NO_CHANGE => {
                tracing::trace!("Registered {prefix} twice");
                Ok(result)
            },
            _ => {
                tracing::error!("Result of registering prefix {prefix} is {:?}", result);
                Ok(result)
            },
        }
    }

    fn destroy(&mut self) {
        assert!(!self.inner.is_null());
        unsafe {
            CPrefixes_destroy(self.inner);
        }
        self.inner = ptr::null_mut();
        tracing::trace!(target: crate::LOG_TARGET_DATABASE, "Destroyed Prefixes");
    }

    pub fn declare<'a, Base: Into<Iri<'a>>>(
        self: &Arc<Self>,
        name: &str,
        iri: Base,
    ) -> Result<PrefixDeclareResult, Error> {
        self.declare_prefix(&Prefix::declare(name, iri))
    }

    pub fn add_prefix(self: &Arc<Self>, prefix: &Prefix) -> Result<Arc<Self>, Error> {
        let _ = self.declare_prefix(prefix);
        Ok(self.clone())
    }

    pub fn add_class(self: &Arc<Self>, clazz: &Class) -> Result<Arc<Self>, Error> {
        self.add_prefix(&clazz.prefix)
    }

    pub fn add_predicate(self: &Arc<Self>, predicate: &Predicate) -> Result<Arc<Self>, Error> {
        self.add_prefix(predicate.namespace)
    }

    pub fn for_each_prefix_do<F: FnMut(&str, &Prefix) -> Result<(), E>, E>(
        &self,
        mut f: F,
    ) -> Result<(), E> {
        for (key, prefix) in self.map.lock().unwrap().iter() {
            f(key.as_str(), prefix)?;
        }
        Ok(())
    }

    pub fn c_ptr(&self) -> *const CPrefixes { self.inner }

    pub fn c_mut_ptr(&self) -> *mut CPrefixes { self.inner }
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
        match iri.as_str().chars().last() {
            Some('/') | Some('#') => {
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
        let binding = self.iri.as_iri_ref();
        match binding.path().as_str().chars().last() {
            Some(char) if char == '/' => {
                IriBuf::new(format!("{}{}", self.iri.as_str(), name).as_str())
            },
            Some(char) if char == '#' => {
                IriBuf::new(format!("{}{}", self.iri.as_str(), name).as_str())
            },
            _ => IriBuf::new(format!("{}/{}", self.iri.as_str(), name).as_str()),
        }
    }

    #[cfg(feature = "rdftk_support")]
    pub fn as_rdftk_iri_ref(&self) -> Result<rdftk_iri::IRIRef, rdftk_iri::error::Error> {
        Ok(rdftk_iri::IRIRef::new(self.as_rdftk_iri()?))
    }

    #[cfg(feature = "rdftk_support")]
    pub fn as_rdftk_iri(&self) -> Result<rdftk_iri::IRI, rdftk_iri::error::Error> {
        use std::str::FromStr;
        rdftk_iri::IRI::from_str(self.iri.as_str())
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

    pub fn build(self) -> Result<Arc<Prefixes>, Error> {
        let to_build = Prefixes::empty()?;
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
