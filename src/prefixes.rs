// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{
        database_call,
        root::{
            CPrefixes,
            CPrefixes_DeclareResult as PrefixDeclareResult,
            CPrefixes_declarePrefix,
            CPrefixes_destroy,
            CPrefixes_newDefaultPrefixes,
        },
        Predicate,
    },
    iref::Iri,
    rdf_store_rs::{
        consts::{LOG_TARGET_DATABASE, PREFIX_OWL, PREFIX_RDF, PREFIX_RDFS, PREFIX_XSD},
        Class,
        Prefix,
        RDFStoreError,
    },
    std::{
        collections::HashMap,
        ffi::CString,
        ops::Deref,
        ptr,
        sync::{Arc, Mutex},
    },
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
    fn drop(&mut self) {
        assert!(!self.inner.is_null());
        unsafe {
            CPrefixes_destroy(self.inner);
        }
        self.inner = ptr::null_mut();
        tracing::trace!(target: LOG_TARGET_DATABASE, "Dropped Prefixes");
    }
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

    pub fn empty() -> Result<Arc<Self>, RDFStoreError> {
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
    pub fn default() -> Result<Arc<Self>, RDFStoreError> {
        Self::empty()?
            .add_prefix(PREFIX_RDF.deref())?
            .add_prefix(PREFIX_RDFS.deref())?
            .add_prefix(PREFIX_OWL.deref())?
            .add_prefix(PREFIX_XSD.deref())
    }

    pub fn declare_prefix(
        self: &Arc<Self>,
        prefix: &Prefix,
    ) -> Result<PrefixDeclareResult, RDFStoreError> {
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
            format!(
                "Declaring prefix {} for namespace {}",
                prefix.name.as_str(),
                prefix.iri.as_str()
            )
            .as_str(),
            CPrefixes_declarePrefix(
                self.inner,
                c_name.as_ptr(),
                c_iri.as_ptr(),
                &mut result
            )
        )?;
        match result {
            PrefixDeclareResult::PREFIXES_INVALID_PREFIX_NAME => {
                tracing::error!(
                    target: LOG_TARGET_DATABASE,
                    "Invalid prefix name \"{}\" while registering namespace <{}>",
                    prefix.name.as_str(),
                    prefix.iri.as_str()
                );
                Err(RDFStoreError::InvalidPrefixName)
            },
            PrefixDeclareResult::PREFIXES_DECLARED_NEW => Ok(result),
            PrefixDeclareResult::PREFIXES_NO_CHANGE => {
                tracing::trace!(
                    target: LOG_TARGET_DATABASE,
                    "Registered {prefix} twice"
                );
                Ok(result)
            },
            _ => {
                tracing::error!(
                    target: LOG_TARGET_DATABASE,
                    "Result of registering prefix {prefix} is {:?}",
                    result
                );
                Ok(result)
            },
        }
    }

    pub fn declare<'a, Base: Into<Iri<'a>>>(
        self: &Arc<Self>,
        name: &str,
        iri: Base,
    ) -> Result<PrefixDeclareResult, RDFStoreError> {
        self.declare_prefix(&Prefix::declare(name, iri))
    }

    pub fn add_prefix(self: &Arc<Self>, prefix: &Prefix) -> Result<Arc<Self>, RDFStoreError> {
        let _ = self.declare_prefix(prefix);
        Ok(self.clone())
    }

    pub fn add_class(self: &Arc<Self>, clazz: &Class) -> Result<Arc<Self>, RDFStoreError> {
        self.add_prefix(&clazz.prefix)
    }

    pub fn add_predicate(
        self: &Arc<Self>,
        predicate: &Predicate,
    ) -> Result<Arc<Self>, RDFStoreError> {
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

#[derive(Default)]
pub struct PrefixesBuilder {
    prefixes: Vec<Prefix>,
}

impl<'a> PrefixesBuilder {
    pub fn default() -> Self { PrefixesBuilder { prefixes: Vec::new() } }

    pub fn declare_with_name_and_iri<Base: Into<Iri<'a>>>(mut self, name: &str, iri: Base) -> Self {
        self.prefixes.push(Prefix::declare(name, iri));
        self
    }

    pub fn declare(mut self, prefix: Prefix) -> Self {
        self.prefixes.push(prefix);
        self
    }

    pub fn build(self) -> Result<Arc<Prefixes>, RDFStoreError> {
        let to_build = Prefixes::empty()?;
        for prefix in self.prefixes {
            to_build.declare_prefix(&prefix)?;
        }
        Ok(to_build)
    }
}
