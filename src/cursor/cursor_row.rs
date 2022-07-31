// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

use std::convert::TryFrom;
use std::ffi::{c_ulong, CStr};
use std::panic::AssertUnwindSafe;
use std::ptr;

use crate::{DataType, DataValue, Error, OpenedCursor};
use crate::cursor::ResourceValue;
use crate::Error::UNKNOWN;
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
    pub fn resource_id(&self, term_index: u64) -> Result<u64, Error> {
        if let Some(argument_index) = self.opened.argument_indexes.get(term_index as usize) {
            if let Some(resource_id) = self.opened.arguments_buffer.get(*argument_index as usize) {
                Ok(*resource_id)
            } else {
                log::error!("Could not get the resource ID from the arguments buffer with argument index {argument_index} and term index {term_index}");
                Err(UNKNOWN)
            }
        } else {
            log::error!("Could not get the argument index for term index {term_index}");
            Err(UNKNOWN)
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
    pub fn resource_value(&self, resource_id: u64) -> Result<ResourceValue, Error> {
        let mut data: *const u8 = ptr::null_mut();
        let mut data_size: std::os::raw::c_ulong = 0;
        let mut prefix_data: *const u8 = ptr::null_mut();
        let mut prefix_data_size: std::os::raw::c_ulong = 0;
        let mut datatype_id = 0 as CDatatypeID;
        let mut resource_resolved = false;
        log::trace!("CCursor_getResourceValue({resource_id}):");
        CException::handle(AssertUnwindSafe(|| unsafe {
            CCursor_getResourceValue(
                self.opened.cursor.inner,
                resource_id,
                &mut data,
                &mut data_size,
                &mut prefix_data,
                &mut prefix_data_size,
                &mut datatype_id,
                &mut resource_resolved,
            )
        }))?;

        if !resource_resolved {
            log::error!("Call to cursor for resource id {resource_id} could not be resolved");
            return Err(UNKNOWN); // TODO: Make more specific error
        }
        if data_size == 0 {
            log::error!("Call to cursor for resource id {resource_id} could not be resolved, no data");
            return Err(UNKNOWN); // TODO: Make more specific error
        }
        if prefix_data_size == 0 {
            log::error!("Call to cursor for resource id {resource_id} could not be resolved, no prefix");
            return Err(UNKNOWN); // TODO: Make more specific error
        }

        log::info!("CCursor_getResourceValue({resource_id}): data_type={datatype_id} data_size={data_size} prefix_data_size={prefix_data_size}");

        fn ptr_to_cstr<'b>(data: *const u8, data_size: std::os::raw::c_ulong) -> Result<&'b CStr, Error> {
            unsafe {
                let slice = std::slice::from_raw_parts(data, data_size as usize);
                Ok(CStr::from_bytes_with_nul_unchecked(slice))
            }
        }

        let c_str_value = ptr_to_cstr(data, data_size)?;
        let c_str_prefix = ptr_to_cstr(prefix_data, prefix_data_size)?;
        log::info!("{c_str_prefix:?}{c_str_value:?}");

        Ok(ResourceValue {
            prefix: c_str_prefix.to_str().unwrap(),
            value: c_str_value.to_str().unwrap(),
        })
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
    pub fn resource_value_lexical_form(&self, resource_id: u64) -> Result<DataValue, Error> {
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
            return Err(UNKNOWN); // TODO: Make more specific error
        }

        let data_type = DataType::try_from(datatype_id).map_err(|_err| {
            log::error!("Cannot convert data type: {datatype_id:?}");
            UNKNOWN // TODO
        })?;

        log::trace!("CCursor_getResourceLexicalForm({resource_id}): data_type={datatype_id:?} lexical_form_size={lexical_form_size}");

        DataValue::from_type_and_c_buffer(data_type, &buffer)
    }
}