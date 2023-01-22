// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::mem::ManuallyDrop;

use iref::{Iri, IriBuf};

pub union LexicalValueUnion {
    pub iri:              ManuallyDrop<IriBuf>,
    pub string:           ManuallyDrop<String>,
    pub boolean:          bool,
    pub unsigned_integer: u64,
    pub signed_integer:   i64,
    pub blank_node:       ManuallyDrop<String>,
}

impl Default for LexicalValueUnion {
    fn default() -> Self {
        Self {
            boolean: false
        }
    }
}

impl LexicalValueUnion {
    pub fn new_string(str: &str) -> Self {
        LexicalValueUnion {
            string: ManuallyDrop::new(str.to_string()),
        }
    }

    pub fn new_iri(iri: &Iri) -> Self {
        LexicalValueUnion {
            iri: ManuallyDrop::new(IriBuf::from(iri)),
        }
    }

    pub fn new_boolean(boolean: bool) -> Self {
        LexicalValueUnion {
            boolean,
        }
    }

    pub fn new_unsigned_integer(unsigned_integer: u64) -> Self {
        LexicalValueUnion {
            unsigned_integer,
        }
    }

    pub fn new_signed_integer(signed_integer: i64) -> Self {
        LexicalValueUnion {
            signed_integer,
        }
    }

    pub fn new_blank_node(blank_node: &str) -> Self {
        LexicalValueUnion {
            blank_node: ManuallyDrop::new(blank_node.to_string()),
        }
    }
}
