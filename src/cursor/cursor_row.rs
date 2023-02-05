// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{
        database_call,
        root::CCursor_appendResourceLexicalForm,
        DataType,
        Literal,
        OpenedCursor,
        RDFStoreError::{self, Unknown},
    },
    tracing::event_enabled,
};

pub struct CursorRow<'a> {
    pub opened:       &'a OpenedCursor<'a>,
    pub multiplicity: u64,
    pub count:        u64,
    pub rowid:        u64,
}

impl<'a> std::fmt::Debug for CursorRow<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Row(")?;
        for term_index in 0..self.opened.arity {
            match self.lexical_value(term_index) {
                Ok(some_value) => {
                    if let Some(value) = some_value {
                        write!(f, "{term_index}={:?}:{value:},", value.data_type)?;
                    } else {
                        write!(f, "{term_index}=UNDEF,")?
                    }
                },
                Err(err) => write!(f, "{term_index}=ERROR: {err:?},")?,
            }
        }
        write!(f, ")")
    }
}

impl<'a> CursorRow<'a> {
    /// Get the resource ID from the arguments buffer which dynamically changes
    /// after each cursor advance.
    fn resource_id(&self, term_index: usize) -> Result<Option<u64>, RDFStoreError> {
        self.opened.resource_id(term_index)
    }

    /// Returns the resource bound to the given index in the current answer row.
    fn lexical_value_with_id(&self, resource_id: u64) -> Result<Option<Literal>, RDFStoreError> {
        let mut buffer = [0u8; 102400]; // TODO: Make this dependent on returned info about buffer size too small
        let mut lexical_form_size = 0 as usize;
        let mut datatype_id: u8 = DataType::UnboundValue as u8;
        let mut resource_resolved = false;
        tracing::trace!("CCursor_getResourceLexicalForm({resource_id}):");
        database_call!(
            "getting a resource value in lexical form",
            CCursor_appendResourceLexicalForm(
                self.opened.cursor.inner,
                resource_id,
                buffer.as_mut_ptr() as *mut i8,
                buffer.len(),
                &mut lexical_form_size,
                &mut datatype_id as *mut u8,
                &mut resource_resolved,
            )
        )?;
        if !resource_resolved {
            tracing::error!("Call to cursor for resource id {resource_id} could not be resolved");
            return Err(Unknown) // TODO: Make more specific error
        }

        let data_type = DataType::from_datatype_id(datatype_id)?;

        if event_enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                "CCursor_getResourceLexicalForm({resource_id}): data_type={datatype_id:?} \
                 lexical_form_size={lexical_form_size:?}"
            );
        }

        Literal::from_type_and_c_buffer(data_type, &buffer)
    }

    /// Get the value in lexical form of a term in the current solution /
    /// current row with the given term index.
    pub fn lexical_value(&self, term_index: usize) -> Result<Option<Literal>, RDFStoreError> {
        let resource_id = self.resource_id(term_index)?;
        if event_enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                "row={rowid} multiplicity={multiplicity} term_index={term_index} \
                 resource_id={resource_id:?}:",
                rowid = self.rowid,
                multiplicity = self.multiplicity
            );
        }
        if let Some(resource_id) = resource_id {
            self.lexical_value_with_id(resource_id)
        } else {
            Ok(None)
        }
    }
}
