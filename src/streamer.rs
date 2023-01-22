use {
    crate::{
        database_call,
        root::{
            CDataStoreConnection,
            CDataStoreConnection_evaluateStatement,
            COutputStream,
            CPrefixes,
        },
        DataStoreConnection,
        Parameters,
        Statement,
    },
    mime::Mime,
    rdf_store_rs::{ptr_to_cstr, Prefix, RDFStoreError},
    std::{
        ffi::{c_void, CString},
        fmt::Debug,
        io::Write,
        ptr,
        sync::Arc,
    },
};

#[derive(PartialEq, Debug)]
struct RefToSelf<'a, W: 'a + Write> {
    streamer: *mut Streamer<'a, W>,
}

impl<'a, W: 'a + Write> Drop for RefToSelf<'a, W> {
    fn drop(&mut self) {
        tracing::trace!(
            "{:p}: Dropping RefToSelf ({self:p})",
            self.streamer
        );
    }
}

/// A `Streamer` is a helper-object that's created by `evaluate_to_stream`
/// to handle the various callbacks from the underlying C-API to RDFox.
#[derive(Debug)]
pub struct Streamer<'a, W: 'a + Write> {
    pub connection:   Arc<DataStoreConnection>,
    pub writer:       W,
    pub statement:    &'a Statement,
    pub mime_type:    &'static Mime,
    pub base_iri:     Prefix,
    pub instant:      std::time::Instant,
    self_p:           String,
    remaining_buffer: std::cell::RefCell<Option<String>>,
}

impl<'a, W: 'a + Write> Drop for Streamer<'a, W> {
    fn drop(&mut self) {
        tracing::trace!("{}: Dropped streamer", self.self_p);
    }
}

impl<'a, W: 'a + Write> Streamer<'a, W> {
    pub fn run(
        connection: &Arc<DataStoreConnection>,
        writer: W,
        statement: &'a Statement,
        mime_type: &'static Mime,
        base_iri: Prefix,
    ) -> Result<Self, RDFStoreError> {
        let streamer = Self {
            connection: connection.clone(),
            writer,
            statement,
            mime_type,
            base_iri,
            instant: std::time::Instant::now(),
            self_p: "".to_string(),
            remaining_buffer: std::cell::RefCell::default(),
        };
        streamer.evaluate()
    }

    /// Evaluate/execute the statement and stream all content to the given
    /// writer, then return the streamer (i.e. self).
    fn evaluate(mut self) -> Result<Self, RDFStoreError> {
        let statement_text = self.statement.as_c_string()?;
        let statement_text_len = statement_text.as_bytes().len();
        let parameters = Parameters::empty()?.fact_domain(crate::FactDomain::ALL)?;
        let query_answer_format_name = CString::new(self.mime_type.as_ref())?;
        let mut number_of_solutions = 0_usize;
        let prefixes_ptr = self.prefixes_ptr();
        let connection_ptr = self.connection_ptr();
        let c_base_iri = CString::new(self.base_iri.iri.as_str()).unwrap();

        let self_p = format!("{:p}", &self);
        self.self_p = self_p.clone();

        tracing::debug!("{self_p}: evaluate statement with mime={query_answer_format_name:?}");

        let ref_to_self = Box::new(RefToSelf { streamer: &mut self as *mut Self });
        let ref_to_self_raw_ptr = Box::into_raw(ref_to_self);

        let stream = Box::new(COutputStream {
            context: ref_to_self_raw_ptr as *mut _,
            flushFn: Some(Self::flush_function),
            writeFn: Some(Self::write_function),
        });
        let stream_raw_ptr = Box::into_raw(stream);

        let result = database_call! {
            "evaluating a statement",
            CDataStoreConnection_evaluateStatement(
                connection_ptr,
                c_base_iri.as_ptr(),
                prefixes_ptr,
                statement_text.as_ptr(),
                statement_text_len,
                parameters.inner,
                stream_raw_ptr as *const COutputStream,
                query_answer_format_name.as_ptr(),
                &mut number_of_solutions,
            )
        };
        // std::thread::sleep(std::time::Duration::from_millis(1000));
        // Explicitly clean up the two boxes that we allocated
        unsafe {
            ptr::drop_in_place(ref_to_self_raw_ptr);
            ptr::drop_in_place(stream_raw_ptr);
        }

        result?; // we're doing this after the drop_in_place calls to avoid memory leak

        tracing::debug!("{self_p}: number_of_solutions={number_of_solutions}");
        Ok(self)
    }

    unsafe fn context_as_ref_to_self(context: *mut c_void) -> &'a mut RefToSelf<'a, W> {
        let ref_to_self = context as *mut RefToSelf<'a, W>;
        &mut *ref_to_self
    }

    extern "C" fn flush_function(context: *mut c_void) -> bool {
        let ref_to_self = unsafe { Self::context_as_ref_to_self(context) };
        let streamer = unsafe { &mut *ref_to_self.streamer };
        tracing::trace!("{streamer:p}: flush_function");
        streamer.flush()
    }

    extern "C" fn write_function(
        context: *mut c_void,
        data: *const c_void,
        number_of_bytes_to_write: usize,
    ) -> bool {
        let ref_to_self = unsafe { Self::context_as_ref_to_self(context) };
        let streamer = unsafe { &mut *ref_to_self.streamer };

        tracing::trace!("{streamer:p}: write_function");

        let result = match ptr_to_cstr(
            data as *const u8,
            number_of_bytes_to_write as usize,
        ) {
            Ok(data_c_str) => {
                tracing::trace!("{streamer:p}: writing {number_of_bytes_to_write} bytes (a)");
                let data = if streamer.remaining_buffer.borrow().is_some() {
                    // If we have some remaining bytes from the previous call to `write_function`
                    // then concatenate them here with the new buffer..
                    [
                        streamer
                            .remaining_buffer
                            .borrow()
                            .as_ref()
                            .unwrap()
                            .as_bytes(),
                        data_c_str.to_bytes_with_nul(),
                    ]
                    .concat()
                } else {
                    data_c_str.to_bytes_with_nul().to_vec()
                };
                let data_len = data.len();
                match streamer.writer.write(&data) {
                    Ok(len) => {
                        tracing::trace!(
                            "{streamer:p}: wrote {len} bytes out of {}",
                            data_len
                        );
                        if len < data_len {
                            // When we didn't process the last part of the buffer (probably because
                            // the last N-Triple line was not complete), then save the remainder
                            // in `remaining_buffer` for the next call to `write_function`
                            streamer.remaining_buffer.replace(Some(unsafe {
                                String::from_utf8_unchecked(data[len..].to_vec())
                            }));
                            tracing::trace!(
                                "{streamer:p}: remaining buffer: {}",
                                streamer.remaining_buffer.borrow().as_ref().unwrap()
                            );
                        } else {
                            streamer.remaining_buffer.replace(None);
                        }
                        true
                    },
                    Err(err) => {
                        panic!("{streamer:p}: could not write: {err:?}")
                    },
                }
            },
            Err(error) => {
                tracing::error!("{streamer:p}: could not write: {error:?}");
                false
            },
        };
        tracing::trace!("{streamer:p}: write_function result={result}");
        result
    }

    fn prefixes_ptr(&self) -> *mut CPrefixes { self.statement.prefixes.c_mut_ptr() }

    fn connection_ptr(&self) -> *mut CDataStoreConnection { self.connection.inner }
}

trait StreamerWithCallbacks {
    fn flush(&mut self) -> bool;
    // fn write(&mut self, data: &[u8]) -> bool;
}

impl<'a, W: 'a + Write> StreamerWithCallbacks for Streamer<'a, W> {
    fn flush(&mut self) -> bool {
        tracing::trace!("{self:p}: flush");
        let y = if let Err(err) = self.writer.flush() {
            panic!("{self:p}: Could not flush: {err:?}")
        } else {
            true
        };
        tracing::trace!("{self:p}: flush returns {y:?}");
        y
    }

    // fn write(&mut self, data: &[u8]) -> bool {
    //     tracing::trace!("{self:p}: writing {} bytes (b)", data.len());
    //     match self.writer.write(data) {
    //         Ok(len) => {
    //             tracing::trace!("{self:p}: wrote {len} bytes out of {}",
    // data.len());             true
    //         },
    //         Err(err) => {
    //             panic!("{self:p}: could not write: {err:?}")
    //         },
    //     }
    // }
}
