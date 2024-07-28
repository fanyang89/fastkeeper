use crate::messages::proto::ConnectResponse;
use rand::random;
use std::sync::{Arc, Mutex};
use tokio::time::Instant;

pub struct ProcessState {
    inner: Arc<Mutex<ProcessStateInner>>,
}

impl Clone for ProcessState {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl ProcessState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProcessStateInner {
                last_zxid: 0,
                session_id: 0,
                passwd: Vec::with_capacity(16),
                last_send: Instant::now(),
                last_recv: Instant::now(),
                xid: random::<i32>(),
            })),
        }
    }

    pub fn inner(&self) -> ProcessStateInner {
        self.inner.lock().unwrap().clone()
    }

    pub fn update_last_send(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.last_send = Instant::now();
    }

    pub fn update_last_recv(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        inner.last_recv = Instant::now();
    }

    pub fn last_recv(&mut self) -> Instant {
        self.inner.lock().unwrap().last_recv
    }

    pub fn last_send(&self) -> Instant {
        self.inner.lock().unwrap().last_send
    }

    pub fn last_zxid(&self) -> i64 {
        self.inner.lock().unwrap().last_zxid
    }

    pub fn session_id(&self) -> i64 {
        self.inner.lock().unwrap().session_id
    }

    pub fn update_session(&mut self, rsp: &ConnectResponse) {
        let mut inner = self.inner.lock().unwrap();
        inner.session_id = rsp.session_id;
        inner.passwd = rsp.passwd.clone();
    }

    pub fn get_xid(&mut self) -> i32 {
        let mut inner = self.inner.lock().unwrap();
        let xid = inner.xid;
        inner.xid += 1;
        xid
    }
}

#[derive(Clone)]
pub struct ProcessStateInner {
    pub last_zxid: i64,
    pub session_id: i64,
    pub passwd: jute::Buffer,
    pub last_send: Instant,
    pub last_recv: Instant,
    pub xid: i32,
}
