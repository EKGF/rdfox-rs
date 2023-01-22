use std::str::FromStr;

// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------
use iref::Iri;

use super::LexicalValue;
use crate::{DataType, Error};

/// An RDF Term is either an IRI, a literal or a blank node.
/// See https://www.w3.org/TR/rdf11-concepts/#section-triples
#[derive(Debug)]
pub enum Term {
    Iri(LexicalValue),
    Literal(LexicalValue),
    BlankNode(LexicalValue),
}

impl Term {
    pub fn new_iri(iri: &Iri) -> Result<Self, Error> { Ok(Term::Iri(LexicalValue::from_iri(iri)?)) }

    pub fn new_iri_from_str(iri_str: &str) -> Result<Self, Error> {
        Term::new_iri(&Iri::new(iri_str)?)
    }

    pub fn new_str(str: &str) -> Result<Self, Error> {
        Ok(Term::Literal(LexicalValue::from_str(str)?))
    }

    pub fn new_blank_node(str: &str) -> Result<Self, Error> {
        Ok(Term::BlankNode(
            LexicalValue::from_type_and_buffer(DataType::BlankNode, str)?.unwrap(),
        ))
    }

    pub fn display_turtle<'a, 'b>(&'a self) -> impl std::fmt::Display + 'a + 'b
    where 'a: 'b {
        struct TurtleTerm<'b>(&'b Term);
        impl<'b> std::fmt::Display for TurtleTerm<'b> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let value = match self.0 {
                    Term::Iri(value) => value,
                    Term::Literal(value) => value,
                    Term::BlankNode(value) => value,
                };
                value.display_turtle().fmt(f)
            }
        }
        TurtleTerm(self)
    }
}

impl FromStr for Term {
    type Err = Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> { Term::new_str(str) }
}

impl From<LexicalValue> for Term {
    fn from(value: LexicalValue) -> Self { value.as_term() }
}

#[cfg(test)]
mod tests {
    use iref::Iri;

    use crate::{rdf::Term, Error};

    #[test_log::test]
    fn test_term_01() {
        let term = Term::new_iri(&Iri::new("https://whatever.url").unwrap()).unwrap();

        let turtle = format!("{}", term.display_turtle());

        assert_eq!(turtle, "<https://whatever.url>");
    }

    #[test_log::test]
    fn test_term_02() {
        let term = Term::new_iri(&Iri::new("unknown-protocol://whatever.url").unwrap()).unwrap();

        let turtle = format!("{}", term.display_turtle());

        assert_eq!(turtle, "<unknown-protocol://whatever.url>");
    }

    #[test_log::test]
    fn test_term_03() {
        // At the moment, we're even accepting wrongly formatted IRIs, we may want to
        // validate each IRI
        let term = Term::new_iri(&Iri::new("https:/x/whatever.url").unwrap()).unwrap();

        let turtle = format!("{}", term.display_turtle());

        assert_eq!(turtle, "<https:/x/whatever.url>");
    }

    #[test_log::test]
    fn test_term_04() {
        let term = Term::new_str("some string").unwrap();

        let turtle = format!("{}", term.display_turtle());

        assert_eq!(turtle, "\"some string\"");
    }

    #[test_log::test]
    fn test_term_05() -> Result<(), Error> {
        let term: Term = "some string".parse()?;

        let turtle = format!("{}", term.display_turtle());

        assert_eq!(turtle, "\"some string\"");

        Ok(())
    }

    #[test_log::test]
    fn test_term_06() -> Result<(), Error> {
        let term: Term = "\"some string\"^^xsd:string".parse()?;

        let turtle = format!("{}", term.display_turtle());

        assert_eq!(turtle, "\"\\\"some string\\\"^^xsd:string\""); // TODO: This is incorrect, recognise the XSD data type suffix and process it

        Ok(())
    }
}
