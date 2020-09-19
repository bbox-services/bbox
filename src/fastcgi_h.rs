// Constants/types from fastcgi.h
// bindgen --no-layout-tests fastcgi.h -o fastcgi_h.rs
// Manually changed some const types from u32 to u8

#![allow(dead_code)]
#![allow(non_snake_case)]

pub const FCGI_LISTENSOCK_FILENO: u32 = 0;
pub const FCGI_MAX_LENGTH: u32 = 65535;
pub const FCGI_HEADER_LEN: u32 = 8;
pub const FCGI_VERSION_1: u8 = 1;
pub const FCGI_BEGIN_REQUEST: u8 = 1;
pub const FCGI_ABORT_REQUEST: u8 = 2;
pub const FCGI_END_REQUEST: u8 = 3;
pub const FCGI_PARAMS: u8 = 4;
pub const FCGI_STDIN: u8 = 5;
pub const FCGI_STDOUT: u8 = 6;
pub const FCGI_STDERR: u8 = 7;
pub const FCGI_DATA: u8 = 8;
pub const FCGI_GET_VALUES: u8 = 9;
pub const FCGI_GET_VALUES_RESULT: u8 = 10;
pub const FCGI_UNKNOWN_TYPE: u8 = 11;
pub const FCGI_MAXTYPE: u8 = 11;
pub const FCGI_NULL_REQUEST_ID: u8 = 0;
pub const FCGI_KEEP_CONN: u8 = 1;
pub const FCGI_RESPONDER: u8 = 1;
pub const FCGI_AUTHORIZER: u8 = 2;
pub const FCGI_FILTER: u8 = 3;
pub const FCGI_REQUEST_COMPLETE: u8 = 0;
pub const FCGI_CANT_MPX_CONN: u8 = 1;
pub const FCGI_OVERLOADED: u8 = 2;
pub const FCGI_UNKNOWN_ROLE: u8 = 3;
pub const FCGI_MAX_CONNS: &'static [u8; 15usize] = b"FCGI_MAX_CONNS\0";
pub const FCGI_MAX_REQS: &'static [u8; 14usize] = b"FCGI_MAX_REQS\0";
pub const FCGI_MPXS_CONNS: &'static [u8; 16usize] = b"FCGI_MPXS_CONNS\0";
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FCGI_Header {
    pub version: ::std::os::raw::c_uchar,
    pub type_: ::std::os::raw::c_uchar,
    pub requestIdB1: ::std::os::raw::c_uchar,
    pub requestIdB0: ::std::os::raw::c_uchar,
    pub contentLengthB1: ::std::os::raw::c_uchar,
    pub contentLengthB0: ::std::os::raw::c_uchar,
    pub paddingLength: ::std::os::raw::c_uchar,
    pub reserved: ::std::os::raw::c_uchar,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FCGI_BeginRequestBody {
    pub roleB1: ::std::os::raw::c_uchar,
    pub roleB0: ::std::os::raw::c_uchar,
    pub flags: ::std::os::raw::c_uchar,
    pub reserved: [::std::os::raw::c_uchar; 5usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FCGI_BeginRequestRecord {
    pub header: FCGI_Header,
    pub body: FCGI_BeginRequestBody,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FCGI_EndRequestBody {
    pub appStatusB3: ::std::os::raw::c_uchar,
    pub appStatusB2: ::std::os::raw::c_uchar,
    pub appStatusB1: ::std::os::raw::c_uchar,
    pub appStatusB0: ::std::os::raw::c_uchar,
    pub protocolStatus: ::std::os::raw::c_uchar,
    pub reserved: [::std::os::raw::c_uchar; 3usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FCGI_EndRequestRecord {
    pub header: FCGI_Header,
    pub body: FCGI_EndRequestBody,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FCGI_UnknownTypeBody {
    pub type_: ::std::os::raw::c_uchar,
    pub reserved: [::std::os::raw::c_uchar; 7usize],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FCGI_UnknownTypeRecord {
    pub header: FCGI_Header,
    pub body: FCGI_UnknownTypeBody,
}
