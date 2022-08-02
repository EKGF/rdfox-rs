// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::ffi::c_ulong;
use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::{DataType, Error, LexicalValue, OpenedCursor};
use crate::cursor::ResourceValue;
use crate::Error::{Unknown};
use crate::root::{
    CCursor_getResourceLexicalForm,
    CCursor_getResourceValue,
    CDatatypeID,
    CException,
};

#[derive(Debug)]
pub struct CursorRow<'a> {
    pub opened: &'a OpenedCursor<'a>,
    pub multiplicity: u64,
    pub count: u64,
    pub rowid: u64,
}

impl<'a> CursorRow<'a> {
    pub fn resource_id(&self, term_index: u16) -> Result<u64, Error> {
        if let Some(argument_index) = self.opened.argument_indexes.get(term_index as usize) {
            if let Some(resource_id) = self.opened.arguments_buffer.get(*argument_index as usize) {
                Ok(*resource_id)
            } else {
                log::error!("Could not get the resource ID from the arguments buffer with argument index {argument_index} and term index {term_index}");
                Err(Unknown)
            }
        } else {
            log::error!("Could not get the argument index for term index {term_index}");
            Err(Unknown)
        }
    }

    // pub fn CCursor_getResourceValue(
    //     cursor: *mut root::CCursor,
    //     resourceID: root::CResourceID,
    //     data: *mut *const root::byte_t,
    //     dataSize: *mut root::size_t,
    //     prefixData: *mut *const root::byte_t,
    //     prefixDataSize: *mut root::size_t,
    //     datatypeID: *mut root::CDatatypeID,
    //     resourceResolved: *mut bool,
    // ) -> *const root::CException;
    /// Returns the resource bound to the given index in the current answer row.
    pub fn resource_value_with_id(&self, resource_id: u64) -> Result<ResourceValue, Error> {
        let mut data: *const u8 = ptr::null_mut();
        let mut data_size: std::os::raw::c_ulong = 0;
        let mut namespace: *const u8 = ptr::null_mut();
        let mut namespace_size: std::os::raw::c_ulong = 0;
        let mut datatype_id = 0 as CDatatypeID;
        let mut resource_resolved = false;
        log::trace!("CCursor_getResourceValue({resource_id}):");
        CException::handle(AssertUnwindSafe(|| unsafe {
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
        }))?;

        if !resource_resolved {
            log::error!("Call to cursor (row {}) for resource id {resource_id} could not be resolved", self.rowid);
            return Err(Unknown); // TODO: Make more specific error
        }

        let data_type = DataType::from_datatype_id(datatype_id)?;

        log::debug!("row={}: CCursor_getResourceValue({resource_id}): data_type={datatype_id} len={data_size} namespace_len={namespace_size}", self.rowid);

        if data_size == 0 {
            log::error!("Call to cursor (row {}) resource id {resource_id} could not be resolved, no data", self.rowid);
            return Err(Unknown); // TODO: Make more specific error
        }

        ResourceValue::from(
            data_type,
            namespace, namespace_size as usize,
            data, data_size as usize
        )
    }

    /// Get the value of a term in the current solution / current row with the given term index.
    pub fn resource_value(&self, term_index: u16) -> Result<ResourceValue, Error> {
        let resource_id = self.resource_id(term_index)?;
        log::debug!(
                "row={rowid} multiplicity={multiplicity} \
                 term_index={term_index} resource_id={resource_id}:",
                rowid = self.rowid,
                multiplicity = self.multiplicity
            );
        let value = self.resource_value_with_id(resource_id)?;
        log::debug!("{value:?}");
        Ok(value)
    }


    // pub fn CCursor_getResourceLexicalForm(
    //     cursor: *mut root::CCursor,
    //     resourceID: root::CResourceID,
    //     buffer: *mut ::std::os::raw::c_char,
    //     bufferSize: root::size_t,
    //     lexicalFormSize: *mut root::size_t,
    //     datatypeID: *mut root::CDatatypeID,
    //     resourceResolved: *mut bool,
    // ) -> *const root::CException;
    /// Returns the resource bound to the given index in the current answer row.
    pub fn lexical_value_with_id(&self, resource_id: u64) -> Result<LexicalValue, Error> {
        let mut buffer = [0u8; 1024];
        let mut lexical_form_size = 0 as c_ulong;
        let mut datatype_id: u8 = DataType::UnboundValue as u8;
        let mut resource_resolved = false;
        log::trace!("CCursor_getResourceLexicalForm({resource_id}):");
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_getResourceLexicalForm(
                self.opened.cursor.inner,
                resource_id,
                buffer.as_mut_ptr() as *mut i8,
                buffer.len() as c_ulong,
                &mut lexical_form_size,
                &mut datatype_id as *mut u8,
                &mut resource_resolved,
            )
        }))?;

        if !resource_resolved {
            log::error!("Call to cursor for resource id {resource_id} could not be resolved");
            return Err(Unknown); // TODO: Make more specific error
        }

        let data_type = DataType::from_datatype_id(datatype_id)?;

        log::trace!("CCursor_getResourceLexicalForm({resource_id}): data_type={datatype_id:?} lexical_form_size={lexical_form_size}");

        LexicalValue::from_type_and_c_buffer(data_type, &buffer)
    }

    /// Get the value in lexical form of a term in the current solution / current row with the given term index.
    pub fn lexical_value(&self, term_index: u16) -> Result<LexicalValue, Error> {
        let resource_id = self.resource_id(term_index)?;
        log::debug!(
                "row={rowid} multiplicity={multiplicity} \
                 term_index={term_index} resource_id={resource_id}:",
                rowid = self.rowid,
                multiplicity = self.multiplicity
            );
        self.lexical_value_with_id(resource_id)
    }
}

