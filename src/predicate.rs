use crate::prefix::Prefix;

pub struct Predicate<'a> {
    pub namespace:  &'a Prefix,
    pub local_name: String,
}

impl<'a> std::fmt::Display for Predicate<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}{}>", self.namespace.iri, self.local_name)
    }
}

impl<'a> Predicate<'a> {
    pub fn display_turtle<'b>(&'a self) -> impl std::fmt::Display + 'a + 'b
    where 'a: 'b {
        struct TurtlePredicate<'b>(&'b Predicate<'b>);
        impl<'b> std::fmt::Display for TurtlePredicate<'b> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}{}", self.0.namespace.name, self.0.local_name)
            }
        }
        TurtlePredicate(self)
    }

    pub fn declare(namespace: &'a Prefix, local_name: &str) -> Self {
        Self {
            namespace,
            local_name: local_name.to_string(),
        }
    }

    #[cfg(feature = "rdftk_support")]
    pub fn as_rdftk_iri_ref(&self) -> Result<rdftk_iri::IRIRef, rdftk_iri::error::Error> {
        Ok(rdftk_iri::IRIRef::new(self.as_rdftk_iri()?))
    }

    #[cfg(feature = "rdftk_support")]
    pub fn as_rdftk_iri(&self) -> Result<rdftk_iri::IRI, rdftk_iri::error::Error> {
        use std::str::FromStr;
        let path = format!(
            "{}{}",
            self.namespace.iri.path().as_str(),
            self.local_name.as_str()
        );
        Ok(self
            .namespace
            .as_rdftk_iri()?
            .with_new_path(rdftk_iri::Path::from_str(path.as_str())?))
    }
}

#[cfg(test)]
mod tests {
    use iref::Iri;

    use crate::{predicate::Predicate, prefix::Prefix};

    #[test]
    fn test_predicate() {
        let ns = Prefix::declare("abc:", Iri::new("https://whatever.kg/def/").unwrap());
        let prd = Predicate::declare(&ns, "xyz");

        let str_prd = format!("{:}", prd);

        assert_eq!(str_prd.as_str(), "<https://whatever.kg/def/xyz>");

        let str_prd = format!("{}", prd.display_turtle());

        assert_eq!(str_prd.as_str(), "abc:xyz");
    }

    #[cfg(feature = "rdftk_support")]
    #[test]
    fn test_predicate_as_iri_ref() {
        let ns = Prefix::declare("abc:", Iri::new("https://whatever.kg/def/").unwrap());
        let prd = Predicate::declare(&ns, "xyz");

        let iri_ref_result = prd.as_rdftk_iri_ref();
        assert!(iri_ref_result.is_ok());
        let iri_ref = iri_ref_result.unwrap();

        let str_prd = format!("{:}", iri_ref);

        assert_eq!(str_prd.as_str(), "https://whatever.kg/def/xyz");
    }
}
