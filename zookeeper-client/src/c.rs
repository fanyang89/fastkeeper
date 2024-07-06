#![allow(nonstandard_style)]

use crate::messages::data::Stat;
use crate::{messages, Client};
use std::convert::Into;
use std::ffi::{c_char, c_int, c_void, CStr};

/// Creates a new [`CStr`] from a string literal.
/// The string literal should not contain any `NUL` bytes.
/// https://github.com/torvalds/linux/blob/master/rust/kernel/str.rs
#[macro_export]
macro_rules! c_str {
    ($str:expr) => {{
        const S: &str = concat!($str, "\0");
        const C: &CStr = match CStr::from_bytes_with_nul(S.as_bytes()) {
            Ok(v) => v,
            Err(_) => panic!("string contains interior NUL"),
        };
        C
    }};
}

pub enum ZOO_ERRORS {
    ZOK = 0,
    ZSYSTEMERROR = -1,
    ZRUNTIMEINCONSISTENCY = -2,
    ZDATAINCONSISTENCY = -3,
    ZCONNECTIONLOSS = -4,
    ZMARSHALLINGERROR = -5,
    ZUNIMPLEMENTED = -6,
    ZOPERATIONTIMEOUT = -7,
    ZBADARGUMENTS = -8,
    ZINVALIDSTATE = -9,
    ZNEWCONFIGNOQUORUM = -13,
    ZRECONFIGINPROGRESS = -14,
    ZSSLCONNECTIONERROR = -15,

    // API errors
    ZAPIERROR = -100,
    ZNONODE = -101,
    ZNOAUTH = -102,
    ZBADVERSION = -103,
    ZNOCHILDRENFOREPHEMERALS = -108,
    ZNODEEXISTS = -110,
    ZNOTEMPTY = -111,
    ZSESSIONEXPIRED = -112,
    ZINVALIDCALLBACK = -113,
    ZINVALIDACL = -114,
    ZAUTHFAILED = -115,
    ZCLOSING = -116,
    ZNOTHING = -117,
    ZSESSIONMOVED = -118,
    ZNOTREADONLY = -119,
    ZEPHEMERALONLOCALSESSION = -120,
    ZNOWATCHER = -121,
    ZRECONFIGDISABLED = -123,

    ZSESSIONCLOSEDREQUIRESASLAUTH = -124,
    ZTHROTTLEDOP = -127,
}

pub enum ZooLogLevel {
    ZOO_LOG_LEVEL_ERROR = 1,
    ZOO_LOG_LEVEL_WARN = 2,
    ZOO_LOG_LEVEL_INFO = 3,
    ZOO_LOG_LEVEL_DEBUG = 4,
}

pub const ZOO_PERM_READ: c_int = 1 << 0;
pub const ZOO_PERM_WRITE: c_int = 1 << 1;
pub const ZOO_PERM_CREATE: c_int = 1 << 2;
pub const ZOO_PERM_DELETE: c_int = 1 << 3;
pub const ZOO_PERM_ADMIN: c_int = 1 << 4;
pub const ZOO_PERM_ALL: c_int = 0x1f;

// ignore by cbindgen
// > WARN: Can't find CStr. This usually means that this type was incompatible or not found.
// define this const at cbindgen.toml
#[no_mangle]
pub static ZOO_CONFIG_NODE: &'static CStr = c_str!("/zookeeper/config");

pub const ZOO_READONLY: c_int = 1;

pub const ZOO_NO_LOG_CLIENTENV: c_int = 2;

pub struct Id {
    scheme: &'static str,
    id: &'static str,
}

impl Into<messages::data::Id> for Id {
    fn into(self) -> messages::data::Id {
        messages::data::Id {
            scheme: self.scheme.to_string(),
            id: self.id.to_string(),
        }
    }
}

#[no_mangle]
pub static ZOO_ANYONE_ID_UNSAFE: &'static Id = &Id {
    scheme: "world",
    id: "anyone",
};

#[no_mangle]
pub static ZOO_AUTH_IDS: &'static Id = &Id {
    scheme: "auth",
    id: "",
};

// TODO
// /** This is a completely open ACL*/
// extern ZOOAPI struct ACL_vector ZOO_OPEN_ACL_UNSAFE;
// /** This ACL gives the world the ability to read. */
// extern ZOOAPI struct ACL_vector ZOO_READ_ACL_UNSAFE;
// /** This ACL gives the creators authentication id's all permissions. */
// extern ZOOAPI struct ACL_vector ZOO_CREATOR_ALL_ACL;

pub const ZOOKEEPER_WRITE: i32 = 1 << 0;
pub const ZOOKEEPER_READ: i32 = 1 << 1;

pub const ZOO_PERSISTENT: i32 = 0;
pub const ZOO_EPHEMERAL: i32 = 1;
pub const ZOO_PERSISTENT_SEQUENTIAL: i32 = 2;
pub const ZOO_EPHEMERAL_SEQUENTIAL: i32 = 3;
pub const ZOO_CONTAINER: i32 = 4;
pub const ZOO_PERSISTENT_WITH_TTL: i32 = 5;
pub const ZOO_PERSISTENT_SEQUENTIAL_WITH_TTL: i32 = 6;

pub const ZOO_SEQUENCE: i32 = 1 << 1;

pub const ZOO_EXPIRED_SESSION_STATE: i32 = -112;
pub const ZOO_AUTH_FAILED_STATE: i32 = -113;
pub const ZOO_CONNECTING_STATE: i32 = 1;
pub const ZOO_ASSOCIATING_STATE: i32 = 2;
pub const ZOO_CONNECTED_STATE: i32 = 3;
pub const ZOO_READONLY_STATE: i32 = 5;
pub const ZOO_SSL_CONNECTING_STATE: i32 = 7;
pub const ZOO_NOTCONNECTED_STATE: i32 = 999;

pub const ZOO_CREATED_EVENT: i32 = 1;
pub const ZOO_DELETED_EVENT: i32 = 2;
pub const ZOO_CHANGED_EVENT: i32 = 3;
pub const ZOO_CHILD_EVENT: i32 = 4;
pub const ZOO_SESSION_EVENT: i32 = -1;
pub const ZOO_NOTWATCHING_EVENT: i32 = -2;

// TODO zoo_op

pub struct zhandle_t {
    client: Client,
}

pub struct clientid_t {
    client_id: i64,
    passwd: [c_char; 16],
}

#[no_mangle]
pub extern "C" fn zookeeper_init(
    host: *const c_char,
    watcher: watcher_fn,
    recv_timeout: i32,
    client_id: &clientid_t,
    context: *mut c_void,
    flags: i32,
) -> &zhandle_t {
    todo!();
}

// TODO zookeeper_init2

#[no_mangle]
pub extern "C" fn zookeeper_close(zh: &zhandle_t) {}

#[no_mangle]
pub extern "C" fn zoo_get(
    zh: &zhandle_t,
    path: *const c_char,
    watch: i32,
    buffer: *mut c_char,
    buffer_len: *mut c_int,
    stat: &Stat,
) -> i32 {
    todo!();
}

#[no_mangle]
pub type watcher_fn = fn(
    zhandle_t: *mut zhandle_t,
    r#type: i32,
    state: i32,
    path: *const c_char,
    watcherCtx: *mut c_void,
) -> i32;
