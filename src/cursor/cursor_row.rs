// Copyright (c) 2018-2023, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use {
    crate::{database_call, rdfox_api::CCursor_appendResourceLexicalForm, OpenedCursor},
    rdf_store_rs::{
        consts::LOG_TARGET_DATABASE,
        DataType,
        Literal,
        RDFStoreError::{self, Unknown},
    },
    tracing::event_enabled,
};

/// A `CursorRow` is a row of a [`Cursor`](crate::Cursor), i.e., a set of
/// bindings for the variables in the cursor's answer.
pub struct CursorRow<'a> {
    pub opened:       &'a OpenedCursor<'a>,
    pub multiplicity: &'a usize,
    pub count:        &'a usize,
    pub rowid:        &'a usize,
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
    /// Returns the resource bound to the given index in the current answer row.
    fn lexical_value_with_id(&self, term_index: usize) -> Result<Option<Literal>, RDFStoreError> {
        let mut buffer = [0u8; 102400]; // TODO: Make this dependent on returned info about buffer size too small
        let mut lexical_form_size = 0_usize;
        let mut datatype_id: u8 = DataType::UnboundValue as u8;
        let mut resource_resolved = false;
        tracing::trace!(
            target: LOG_TARGET_DATABASE,
            "CCursor_appendResourceLexicalForm({term_index}):"
        );

        // CCursor_appendResourceLexicalForm(cursor, termIndex, lexicalFormBuffer,
        // sizeof(lexicalFormBuffer), &lexicalFormSize, &datatypeID, &resourceResolved);

        database_call!(
            "getting a resource value in lexical form",
            CCursor_appendResourceLexicalForm(
                self.opened.cursor.inner,
                term_index,
                buffer.as_mut_ptr() as *mut i8,
                buffer.len(),
                &mut lexical_form_size,
                &mut datatype_id as *mut u8,
                &mut resource_resolved,
            )
        )?;
        if !resource_resolved {
            tracing::error!(
                target: LOG_TARGET_DATABASE,
                "Call to cursor for resource value in column #{term_index} could not be resolved"
            );
            return Err(Unknown) // TODO: Make more specific error
        }

        let data_type = DataType::from_datatype_id(datatype_id)?;

        if event_enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                target: LOG_TARGET_DATABASE,
                "CCursor_appendResourceLexicalForm({term_index}): data_type={datatype_id:?} \
                 lexical_form_size={lexical_form_size:?}"
            );
        }

        Literal::from_type_and_c_buffer(data_type, &buffer)
    }

    /// Get the value in lexical form of a term in the current solution /
    /// current row with the given term index.
    pub fn lexical_value(&self, term_index: usize) -> Result<Option<Literal>, RDFStoreError> {
        if event_enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                target: LOG_TARGET_DATABASE,
                "row={rowid} multiplicity={multiplicity} term_index={term_index}:",
                rowid = self.rowid,
                multiplicity = self.multiplicity
            );
        }
        self.lexical_value_with_id(term_index)
    }
}
