// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{
        database_call,
        rdfox_api::{
            CPrefixes,
            CPrefixes_DeclareResult as NamespaceDeclareResult,
            CPrefixes_declarePrefix,
            CPrefixes_destroy,
            CPrefixes_newDefaultPrefixes,
        },
    },
    iref::Iri,
    rdf_store_rs::{
        consts::{LOG_TARGET_DATABASE, PREFIX_OWL, PREFIX_RDF, PREFIX_RDFS, PREFIX_XSD},
        Class,
        Namespace,
        Predicate,
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
pub struct Namespaces {
    inner: *mut CPrefixes,
    map:   Mutex<HashMap<String, Namespace>>,
}

impl PartialEq for Namespaces {
    fn eq(&self, other: &Self) -> bool { self.c_ptr() == other.c_ptr() }
}

impl Eq for Namespaces {}

unsafe impl Send for Namespaces {}

unsafe impl Sync for Namespaces {}

impl Drop for Namespaces {
    fn drop(&mut self) {
        assert!(!self.inner.is_null());
        unsafe {
            CPrefixes_destroy(self.inner);
        }
        self.inner = ptr::null_mut();
        tracing::trace!(target: LOG_TARGET_DATABASE, "Dropped Namespaces");
    }
}

/// Show the namespaces in SPARQL format
impl std::fmt::Display for Namespaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _prefix in self.map.lock().unwrap().values() {
            writeln!(f, "PREFIX {_prefix}")?
        }
        Ok(())
    }
}

impl Namespaces {
    pub fn builder() -> NamespacesBuilder { NamespacesBuilder::default() }

    pub fn empty() -> Result<Arc<Self>, RDFStoreError> {
        let mut prefixes = Self {
            inner: ptr::null_mut(),
            map:   Mutex::new(HashMap::new()),
        };
        database_call!(
            "allocating namespaces",
            CPrefixes_newDefaultPrefixes(&mut prefixes.inner)
        )?;
        Ok(Arc::new(prefixes))
    }

    /// Return the default namespaces: `RDF`, `RDFS`, `OWL` and `XSD`
    pub fn default_namespaces() -> Result<Arc<Self>, RDFStoreError> {
        Self::empty()?
            .add_namespace(PREFIX_RDF.deref())?
            .add_namespace(PREFIX_RDFS.deref())?
            .add_namespace(PREFIX_OWL.deref())?
            .add_namespace(PREFIX_XSD.deref())
    }

    pub fn declare_namespace(
        self: &Arc<Self>,
        namespace: &Namespace,
    ) -> Result<NamespaceDeclareResult, RDFStoreError> {
        tracing::trace!("Register namespace {namespace}");
        if let Some(_already_registered) = self
            .map
            .lock()
            .unwrap()
            .insert(namespace.name.clone(), namespace.clone())
        {
            return Ok(NamespaceDeclareResult::PREFIXES_NO_CHANGE)
        }
        let c_name = CString::new(namespace.name.as_str()).unwrap();
        let c_iri = CString::new(namespace.iri.as_str()).unwrap();
        let mut result = NamespaceDeclareResult::PREFIXES_NO_CHANGE;
        database_call!(
            format!(
                "Declaring prefix {} for namespace {}",
                namespace.name.as_str(),
                namespace.iri.as_str()
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
            NamespaceDeclareResult::PREFIXES_INVALID_PREFIX_NAME => {
                tracing::error!(
                    target: LOG_TARGET_DATABASE,
                    "Invalid prefix name \"{}\" while registering namespace <{}>",
                    namespace.name.as_str(),
                    namespace.iri.as_str()
                );
                Err(RDFStoreError::InvalidPrefixName)
            },
            NamespaceDeclareResult::PREFIXES_DECLARED_NEW => Ok(result),
            NamespaceDeclareResult::PREFIXES_NO_CHANGE => {
                tracing::trace!(
                    target: LOG_TARGET_DATABASE,
                    "Registered {namespace} twice"
                );
                Ok(result)
            },
            _ => {
                tracing::error!(
                    target: LOG_TARGET_DATABASE,
                    "Result of registering prefix {namespace} is {:?}",
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
    ) -> Result<NamespaceDeclareResult, RDFStoreError> {
        self.declare_namespace(&Namespace::declare(name, iri))
    }

    pub fn add_namespace(
        self: &Arc<Self>,
        namespace: &Namespace,
    ) -> Result<Arc<Self>, RDFStoreError> {
        let _ = self.declare_namespace(namespace);
        Ok(self.clone())
    }

    pub fn add_class(self: &Arc<Self>, clazz: &Class) -> Result<Arc<Self>, RDFStoreError> {
        self.add_namespace(&clazz.namespace)
    }

    pub fn add_predicate(
        self: &Arc<Self>,
        predicate: &Predicate,
    ) -> Result<Arc<Self>, RDFStoreError> {
        self.add_namespace(predicate.namespace)
    }

    pub fn for_each_namespace_do<F: FnMut(&str, &Namespace) -> Result<(), E>, E>(
        &self,
        mut f: F,
    ) -> Result<(), E> {
        for (key, namespace) in self.map.lock().unwrap().iter() {
            f(key.as_str(), namespace)?;
        }
        Ok(())
    }

    pub fn c_ptr(&self) -> *const CPrefixes { self.inner }

    pub fn c_mut_ptr(&self) -> *mut CPrefixes { self.inner }
}

#[derive(Default)]
pub struct NamespacesBuilder {
    namespaces: Vec<Namespace>,
}

impl<'a> NamespacesBuilder {
    pub fn default_builder() -> Self { NamespacesBuilder { namespaces: Vec::new() } }

    pub fn declare_with_name_and_iri<Base: Into<Iri<'a>>>(mut self, name: &str, iri: Base) -> Self {
        self.namespaces.push(Namespace::declare(name, iri));
        self
    }

    pub fn declare(mut self, namespace: Namespace) -> Self {
        self.namespaces.push(namespace);
        self
    }

    pub fn build(self) -> Result<Arc<Namespaces>, RDFStoreError> {
        let to_build = Namespaces::empty()?;
        for namespace in self.namespaces {
            to_build.declare_namespace(&namespace)?;
        }
        Ok(to_build)
    }
}
