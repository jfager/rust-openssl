pub use self::SslError::*;
pub use self::OpensslError::*;

use libc::c_ulong;
use std::error;
use std::io::IoError;
use std::c_str::CString;

use ffi;

/// An SSL error
#[derive(Show, Clone, PartialEq, Eq)]
pub enum SslError {
    /// The underlying stream reported an error
    StreamError(IoError),
    /// The SSL session has been closed by the other end
    SslSessionClosed,
    /// An error in the OpenSSL library
    OpenSslErrors(Vec<OpensslError>)
}

impl error::Error for SslError {
    fn description(&self) -> &str {
        match *self {
            StreamError(_) => "The underlying stream reported an error",
            SslSessionClosed => "The SSL session has been closed by the other end",
            OpenSslErrors(_) => "An error in the OpenSSL library",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            StreamError(ref err) => Some(err as &error::Error),
            _ => None
        }
    }
}

/// An error from the OpenSSL library
#[derive(Show, Clone, PartialEq, Eq)]
pub enum OpensslError {
    /// An unknown error
    UnknownError {
        /// The library reporting the error
        library: String,
        /// The function reporting the error
        function: String,
        /// The reason for the error
        reason: String
    }
}

fn get_lib(err: c_ulong) -> String {
    unsafe { CString::new(ffi::ERR_lib_error_string(err), false) }.to_string()
}

fn get_func(err: c_ulong) -> String {
    unsafe { CString::new(ffi::ERR_func_error_string(err), false).to_string() }
}

fn get_reason(err: c_ulong) -> String {
    unsafe { CString::new(ffi::ERR_reason_error_string(err), false).to_string() }
}

impl SslError {
    /// Creates a new `OpenSslErrors` with the current contents of the error
    /// stack.
    pub fn get() -> SslError {
        let mut errs = vec!();
        loop {
            match unsafe { ffi::ERR_get_error() } {
                0 => break,
                err => errs.push(SslError::from_error_code(err))
            }
        }
        OpenSslErrors(errs)
    }

    /// Creates an `SslError` from the raw numeric error code.
    pub fn from_error(err: c_ulong) -> SslError {
        OpenSslErrors(vec![SslError::from_error_code(err)])
    }

    fn from_error_code(err: c_ulong) -> OpensslError {
        ffi::init();
        UnknownError {
            library: get_lib(err),
            function: get_func(err),
            reason: get_reason(err)
        }
    }
}

#[test]
fn test_uknown_error_should_have_correct_messages() {
    let errs = match SslError::from_error(336032784) {
        OpenSslErrors(errs) => errs,
        _ => panic!("This should always be an `OpenSslErrors` variant.")
    };

    let UnknownError { ref library, ref function, ref reason } = errs[0];

    assert_eq!(library.as_slice(),"SSL routines");
    assert_eq!(function.as_slice(), "SSL23_GET_SERVER_HELLO");
    assert_eq!(reason.as_slice(), "sslv3 alert handshake failure");
}
