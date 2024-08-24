use bytes::Buf;
use std::collections::HashMap;

struct FileHeader {
    magic: i32,
    version: i32,
    dbid: i64,
}

impl Default for FileHeader {
    fn default() -> Self {
        Self {
            magic: bytes::Bytes::from("ZKSN".as_bytes()).get_i32(),
            version: 2,
            dbid: -1,
        }
    }
}

struct SnapLog {
    file_header: FileHeader,

    // session count: i32
    sessions: HashMap<i64, i32>, // id, timeout

                                 // acl cache
                                 // size: i32
}
