// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use iref::Iri;
use lazy_static::lazy_static;

use crate::Prefix;

type PrefixName<'a> = &'a str;

pub const DEFAULT_BASE_IRI: &str = "https://placeholder.kg";

const PREFIX_NAME_DCAT: PrefixName<'static> = "dcat:";
const PREFIX_NAME_OWL: PrefixName<'static> = "owl:";
const PREFIX_NAME_RDF: PrefixName<'static> = "rdf:";
const PREFIX_NAME_RDFS: PrefixName<'static> = "rdfs:";
const PREFIX_NAME_SKOS: PrefixName<'static> = "skos:";
const PREFIX_NAME_XSD: PrefixName<'static> = "xsd:";

const NS_IRI_DCAT: &str = "http://www.w3.org/ns/dcat#";
const NS_IRI_OWL: &str = "http://www.w3.org/2002/07/owl#";
const NS_IRI_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const NS_IRI_RDFS: &str = "http://www.w3.org/2000/01/rdf-schema#";
const NS_IRI_SKOS: &str = "http://www.w3.org/2004/02/skos/core#";
const NS_IRI_XSD: &str = "http://www.w3.org/2001/XMLSchema#";

lazy_static! {
    pub static ref NS_DCAT: Iri<'static> = Iri::new(NS_IRI_DCAT).unwrap();
    pub static ref NS_OWL: Iri<'static> = Iri::new(NS_IRI_OWL).unwrap();
    pub static ref NS_RDF: Iri<'static> = Iri::new(NS_IRI_RDF).unwrap();
    pub static ref NS_RDFS: Iri<'static> = Iri::new(NS_IRI_RDFS).unwrap();
    pub static ref NS_SKOS: Iri<'static> = Iri::new(NS_IRI_SKOS).unwrap();
    pub static ref NS_XSD: Iri<'static> = Iri::new(NS_IRI_XSD).unwrap();
}

lazy_static! {
    pub static ref PREFIX_DCAT: Prefix = Prefix::declare(PREFIX_NAME_DCAT, *NS_DCAT.deref());
    pub static ref PREFIX_OWL: Prefix = Prefix::declare(PREFIX_NAME_OWL, *NS_OWL.deref());
    pub static ref PREFIX_RDF: Prefix = Prefix::declare(PREFIX_NAME_RDF, *NS_RDF.deref());
    pub static ref PREFIX_RDFS: Prefix = Prefix::declare(PREFIX_NAME_RDFS, *NS_RDFS.deref());
    pub static ref PREFIX_SKOS: Prefix = Prefix::declare(PREFIX_NAME_SKOS, *NS_SKOS.deref());
    pub static ref PREFIX_XSD: Prefix = Prefix::declare(PREFIX_NAME_XSD, *NS_XSD.deref());
}
