// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::CString;

use lazy_static::lazy_static;

use crate::{prefix::Prefix, LexicalValue};

lazy_static! {
    pub static ref NS_RDFOX: Prefix =
        Prefix::declare_from_str("rdfox", "http://oxfordsemantic.tech/RDFox#",);
    pub static ref DEFAULT_GRAPH: Graph =
        Graph::declare(NS_RDFOX.deref().clone(), "DefaultTriples");
}

#[derive(Debug, Clone)]
pub struct Graph {
    pub namespace:  Prefix,
    pub local_name: String,
}

/// Print IRI in prefix:localName format
impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.namespace.name.as_str(),
            self.local_name.as_str()
        )
    }
}

impl Graph {
    pub fn declare(namespace: Prefix, local_name: &str) -> Self {
        // TODO: Find a class for URI/IRIs that has separate base + local name
        // and use that as param instead
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

    pub fn as_iri_buf(&self) -> Result<iref::IriBuf, crate::Error> {
        self.namespace
            .with_local_name(self.local_name.as_str())
            .map_err(crate::Error::from)
    }

    pub fn as_display_iri(&self) -> GraphDisplayIRI {
        GraphDisplayIRI {
            graph: self
        }
    }

    pub fn as_c_string(&self) -> Result<CString, crate::Error> {
        CString::new(self.as_iri_buf()?.as_str()).map_err(crate::Error::from)
    }

    pub fn as_lexical_value(&self) -> Result<LexicalValue, crate::Error> {
        Ok(LexicalValue::from_iri(&self.as_iri_buf()?.as_iri())?)
    }
}

pub struct GraphDisplayIRI<'a> {
    graph: &'a Graph,
}

impl<'a> std::fmt::Display for GraphDisplayIRI<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}{}>",
            self.graph.namespace.iri.as_str(),
            self.graph.local_name.as_str()
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_display_iri() {
        let ns = iref::Iri::new("https://whatever.kom/graph/").unwrap();
        let graph_prefix = crate::Prefix::declare("graph:", ns);
        let graph = crate::Graph::declare(graph_prefix, "somedataset");

        assert_eq!(format!("{:}", graph).as_str(), "graph:somedataset");
        assert_eq!(
            format!("{:}", graph.as_display_iri()).as_str(),
            "<https://whatever.kom/graph/somedataset>"
        );
    }

    #[test]
    fn test_graph_ns() {
        let ns = iref::Iri::new("https://whatever.kom/graph/").unwrap();
        let graph_prefix = crate::Prefix::declare("kggraph:", ns);

        let graph = crate::Graph::declare(graph_prefix, "somedataset");
        let c_string = graph.as_c_string().unwrap().into_string().unwrap();

        assert_eq!(c_string, "https://whatever.kom/graph/somedataset");
    }
}
