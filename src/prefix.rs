use iref::{Iri, IriBuf};

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
