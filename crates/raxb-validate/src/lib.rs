#![doc = include_str!("../README.md")]
/*
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
*/
use std::cmp::Ordering;
use std::ptr::null_mut;
use std::{ffi::CStr, sync::Mutex};

use raxb_libxml2_sys::{
    _xmlError, xmlCharEncoding_XML_CHAR_ENCODING_UTF8, xmlInitParser,
    xmlParserInputBufferCreateMem, xmlRegisterInputCallbacks, xmlSAXHandler,
    xmlSchemaFreeParserCtxt, xmlSchemaFreeValidCtxt, xmlSchemaNewMemParserCtxt,
    xmlSchemaNewValidCtxt, xmlSchemaParse, xmlSchemaParserCtxtPtr, xmlSchemaPtr,
    xmlSchemaSetValidStructuredErrors, xmlSchemaValidCtxtPtr, xmlSchemaValidateStream,
};

use libc::{c_char, c_int, c_void, memcpy, size_t};
use once_cell::sync::Lazy;
use raxb_xmlschema::reader::XmlSchemaResolver;
use thiserror::Error;

#[derive(Clone)]
pub struct XmlSchemaPtr(pub xmlSchemaPtr);
unsafe impl Send for XmlSchemaPtr {}
unsafe impl Sync for XmlSchemaPtr {}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum ErrorLevel {
    None = 0,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone)]
pub struct XmlValidationErrorEntry {
    pub message: String,
    pub line: i32,
    pub level: ErrorLevel,
}

#[derive(Debug, Default)]
pub struct XmlValidationError {
    pub errors: Vec<XmlValidationErrorEntry>,
}

impl std::fmt::Display for XmlValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "Xml Validation errors:")?;
        for err in self.errors.iter() {
            write!(f, "- {:?} at line {}: {}", err.level, err.line, err.message)?;
        }
        Ok(())
    }
}
impl std::error::Error for XmlValidationError {}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("unable to lock reader")]
    Lock,
    #[error("libxml2 internal error")]
    Internal,
    #[error(transparent)]
    Validation(#[from] XmlValidationError),
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub struct InitState;

impl InitState {
    fn get(&self) -> bool {
        true
    }
}
static ACTIVE_READER: Lazy<Mutex<()>> = Lazy::new(Mutex::default);
static INIT_STATE: Lazy<InitState> = Lazy::new(|| {
    unsafe {
        xmlInitParser();
        xmlRegisterInputCallbacks(
            Some(match_runtime_fn),
            Some(open_runtime_fn),
            Some(read_runtime_fn),
            Some(close_runtime_fn),
        );
    }
    // init_with_runtime_io(match_runtime_fn, open_runtime_fn);
    InitState
});
struct ActiveSchemaResolver(Box<dyn XmlSchemaResolver>);
static mut ACTIVE_BUNDLE_PTR: *mut ActiveSchemaResolver = null_mut();

pub fn is_initialized() -> bool {
    INIT_STATE.get()
}

#[derive(Debug)]
struct ReadCtx {
    remaining_length: c_int,
    offset: isize,
    root: *const c_char,
}

extern "C" fn error_cb(ctx: *mut c_void, error: *const _xmlError) {
    unsafe {
        let m = CStr::from_ptr((*error).message);
        (*(ctx as *mut XmlValidationError))
            .errors
            .push(XmlValidationErrorEntry {
                line: (*error).line,
                level: match (*error).level {
                    1 => ErrorLevel::Warning,
                    2 => ErrorLevel::Error,
                    3 => ErrorLevel::Fatal,
                    _ => ErrorLevel::None,
                },
                message: m.to_string_lossy().to_string(),
            });
    }
}

#[no_mangle]
unsafe extern "C" fn match_runtime_fn(filename: *const c_char) -> c_int {
    let filename_cstr = unsafe { CStr::from_ptr(filename) };
    let filename = filename_cstr.to_str().unwrap();
    unsafe {
        if (*ACTIVE_BUNDLE_PTR).0.resolve(filename).is_some() {
            return 1;
        }
    }
    0
}

#[no_mangle]
extern "C" fn open_runtime_fn(filename: *const c_char) -> *mut c_void {
    let filename_cstr = unsafe { CStr::from_ptr(filename) };
    let filename = filename_cstr.to_str().unwrap();
    unsafe {
        if let Some(b) = (*ACTIVE_BUNDLE_PTR).0.resolve(filename) {
            let result = Box::<ReadCtx>::into_raw(Box::new(ReadCtx {
                root: b.as_ptr() as *const c_char,
                offset: 0,
                remaining_length: b.len() as c_int,
            }));
            return result as *mut c_void;
        }
    }
    null_mut()
}

#[no_mangle]
extern "C" fn read_runtime_fn(context: *mut c_void, buffer: *mut c_char, len: c_int) -> c_int {
    let mut l = len;
    unsafe {
        let ctx = context as *mut ReadCtx;
        let ptr = (*ctx).root.offset((*ctx).offset) as *mut c_char;
        if l > (*ctx).remaining_length {
            l = (*ctx).remaining_length;
        }
        memcpy(buffer as *mut c_void, ptr as *mut c_void, l as size_t);
        (*ctx).remaining_length -= l;
        (*ctx).offset += l as isize;
        l
    }
}

#[no_mangle]
extern "C" fn close_runtime_fn(context: *mut c_void) -> c_int {
    unsafe {
        let _ = Box::from_raw(context as *mut ReadCtx);
    }
    0
}

pub fn read_schema_bundle<T>(bundle: T) -> ValidationResult<XmlSchemaPtr>
where
    T: XmlSchemaResolver + Send + Sync + 'static,
{
    let mut schema_resolver = ActiveSchemaResolver(Box::new(bundle));
    let _active_reader_lock = ACTIVE_READER.lock().map_err(|_| ValidationError::Lock)?;
    unsafe {
        ACTIVE_BUNDLE_PTR = &mut schema_resolver as *mut ActiveSchemaResolver;
        let buffer = (*ACTIVE_BUNDLE_PTR).0.entrypoint();
        let l = buffer.len() - 1;
        let parser: xmlSchemaParserCtxtPtr =
            xmlSchemaNewMemParserCtxt(buffer.as_ptr() as *const c_char, l as c_int);
        let ptr: xmlSchemaPtr = xmlSchemaParse(parser);
        xmlSchemaFreeParserCtxt(parser);
        Ok(XmlSchemaPtr(ptr))
    }
}

pub fn validate_xml(xml: &[u8], schema: &XmlSchemaPtr) -> ValidationResult<()> {
    let mut error_ctx = XmlValidationError::default();
    let result = unsafe {
        let buffer = xml.as_ptr() as *const c_char;
        let len = xml.len() as c_int;
        let input =
            xmlParserInputBufferCreateMem(buffer, len, xmlCharEncoding_XML_CHAR_ENCODING_UTF8);
        let ctx: xmlSchemaValidCtxtPtr = xmlSchemaNewValidCtxt(schema.0);
        xmlSchemaSetValidStructuredErrors(
            ctx,
            Some(error_cb),
            &mut error_ctx as *mut XmlValidationError as *mut c_void,
        );
        let result = xmlSchemaValidateStream(
            ctx,
            input,
            xmlCharEncoding_XML_CHAR_ENCODING_UTF8,
            null_mut::<xmlSAXHandler>(),
            null_mut(),
        );
        xmlSchemaFreeValidCtxt(ctx);
        result
    };
    match result.cmp(&0) {
        Ordering::Equal => Ok(()),
        Ordering::Less => Err(ValidationError::Internal),
        Ordering::Greater => Err(ValidationError::Validation(error_ctx)),
    }
}
