// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CString;

use crate::Prefix;

#[derive(Debug, Clone)]
pub struct Graph {
    pub namespace:  Prefix,
    pub local_name: String,
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{:}{}>",
            self.namespace.iri.as_str(),
            self.local_name.as_str()
        )
    }
}

impl Graph {
    pub fn declare(namespace: Prefix, local_name: &str) -> Self {
        // TODO: Find a class for URI/IRIs that has separate base + local name and use
        // that as param instead
        Self {
            namespace,
            local_name: local_name.to_string(),
        }
    }

    pub fn dataset_from_path(namespace: Prefix, path: &std::path::Path) -> Self {
        Self::declare(namespace, path.file_name().unwrap().to_str().unwrap())
    }

    pub fn test_dataset_from_path(namespace: Prefix, path: &std::path::Path) -> Self {
        Self::declare(
            namespace,
            format!("test-{}", path.file_name().unwrap().to_str().unwrap()).as_str(),
        )
    }

    pub fn as_iri(&self) -> Result<iref::IriBuf, crate::Error> {
        self.namespace
            .with_local_name(self.local_name.as_str())
            .map_err(crate::Error::from)
    }

    pub fn as_c_string(&self) -> Result<CString, crate::Error> {
        CString::new(self.as_iri()?.as_str()).map_err(crate::Error::from)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_graph_ns() {
        let ns = iref::Iri::new("https://whatever.kom/graph/").unwrap();
        let graph_prefix = crate::Prefix::declare("kggraph:", ns);

        let graph = crate::Graph::declare(graph_prefix, "somedataset");
        let c_string = graph.as_c_string().unwrap().into_string().unwrap();

        assert_eq!(c_string, "https://whatever.kom/graph/somedataset");
    }
}
