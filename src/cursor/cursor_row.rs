// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::{ffi::c_ulong, ptr};

use crate::{
    database_call,
    root::{CCursor_getResourceLexicalForm, CCursor_getResourceValue, CDatatypeID},
    DataType,
    Error,
    Error::Unknown,
    LexicalValue,
    OpenedCursor,
    ResourceValue,
};

#[derive(Debug)]
pub struct CursorRow<'a> {
    pub opened:       &'a OpenedCursor<'a>,
    pub multiplicity: u64,
    pub count:        u64,
    pub rowid:        u64,
}

impl<'a> CursorRow<'a> {
    /// Get the resource ID from the arguments buffer which dynamically changes
    /// after each cursor advance.
    fn resource_id(&self, term_index: u16) -> Result<Option<u64>, Error> {
        self.opened.resource_id(term_index)
    }

    /// Returns the resource bound to the given index in the current answer row.
    fn resource_value_with_id(&self, resource_id: u64) -> Result<ResourceValue, Error> {
        let mut data: *const u8 = ptr::null_mut();
        let mut data_size: std::os::raw::c_ulong = 0;
        let mut namespace: *const u8 = ptr::null_mut();
        let mut namespace_size: std::os::raw::c_ulong = 0;
        let mut datatype_id = 0 as CDatatypeID;
        let mut resource_resolved = false;
        log::trace!("CCursor_getResourceValue({resource_id}):");
        database_call!(
            "getting a resource value",
            CCursor_getResourceValue(
                self.opened.cursor.inner,
                resource_id,
                &mut data,
                &mut data_size,
                &mut namespace,
                &mut namespace_size,
                &mut datatype_id,
                &mut resource_resolved,
            )
        )?;
        if !resource_resolved {
            log::error!(
                "Call to cursor (row {}) for resource id {resource_id} could not be resolved",
                self.rowid
            );
            return Err(Unknown) // TODO: Make more specific error
        }

        let data_type = DataType::from_datatype_id(datatype_id)?;

        log::trace!(
            "row={}: CCursor_getResourceValue({resource_id}): data_type={datatype_id} \
             len={data_size} namespace_len={namespace_size}",
            self.rowid
        );

        if data_size == 0 {
            log::error!(
                "Call to cursor (row {}) resource id {resource_id} could not be resolved, no data",
                self.rowid
            );
            return Err(Unknown) // TODO: Make more specific error
        }

        ResourceValue::from(
            data_type,
            namespace,
            namespace_size as usize,
            data,
            data_size as usize,
        )
    }

    /// Get the value of a term in the current solution / current row with the
    /// given term index.
    pub fn resource_value(&self, term_index: u16) -> Result<Option<ResourceValue>, Error> {
        let resource_id = self.resource_id(term_index)?;
        log::debug!(
            "row={rowid} multiplicity={multiplicity} term_index={term_index} \
             resource_id={resource_id:?}:",
            rowid = self.rowid,
            multiplicity = self.multiplicity
        );
        if let Some(resource_id) = resource_id {
            let value = self.resource_value_with_id(resource_id)?;
            log::debug!("{value:?}");
            Ok(Some(value))
        } else {
            log::debug!("None");
            Ok(None)
        }
    }

    /// Returns the resource bound to the given index in the current answer row.
    fn lexical_value_with_id(&self, resource_id: u64) -> Result<Option<LexicalValue>, Error> {
        let mut buffer = [0u8; 1024];
        let mut lexical_form_size = 0 as c_ulong;
        let mut datatype_id: u8 = DataType::UnboundValue as u8;
        let mut resource_resolved = false;
        log::trace!("CCursor_getResourceLexicalForm({resource_id}):");
        database_call!(
            "getting a resource value in lexical form",
            CCursor_getResourceLexicalForm(
                self.opened.cursor.inner,
                resource_id,
                buffer.as_mut_ptr() as *mut i8,
                buffer.len() as c_ulong,
                &mut lexical_form_size,
                &mut datatype_id as *mut u8,
                &mut resource_resolved,
            )
        )?;
        if !resource_resolved {
            log::error!("Call to cursor for resource id {resource_id} could not be resolved");
            return Err(Unknown) // TODO: Make more specific error
        }

        let data_type = DataType::from_datatype_id(datatype_id)?;

        log::trace!(
            "CCursor_getResourceLexicalForm({resource_id}): data_type={datatype_id:?} \
             lexical_form_size={lexical_form_size:?}"
        );

        LexicalValue::from_type_and_c_buffer(data_type, &buffer)
    }

    /// Get the value in lexical form of a term in the current solution /
    /// current row with the given term index.
    pub fn lexical_value(&self, term_index: u16) -> Result<Option<LexicalValue>, Error> {
        let resource_id = self.resource_id(term_index)?;
        log::debug!(
            "row={rowid} multiplicity={multiplicity} term_index={term_index} \
             resource_id={resource_id:?}:",
            rowid = self.rowid,
            multiplicity = self.multiplicity
        );
        if let Some(resource_id) = resource_id {
            self.lexical_value_with_id(resource_id)
        } else {
            Ok(None)
        }
    }
}
