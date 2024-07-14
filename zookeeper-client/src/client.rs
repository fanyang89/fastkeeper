use std::io;
use std::time::Duration;

use anyhow::anyhow;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::Bytes;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::Instant;
use tokio::{runtime, select, task};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use jute::{Deserialize, Serialize, SerializeToBuffer};

use crate::host_provider::HostProvider;
use crate::messages::proto::ConnectRequest;

pub struct Client {
    runtime: runtime::Runtime,
    token: CancellationToken,
    tasks: JoinSet<()>,
    completion_tx: mpsc::UnboundedSender<Box<CompletionCb>>,
    send_tx: mpsc::UnboundedSender<Request>,
    recv_tx: mpsc::UnboundedSender<Response>,
}

struct Request {
    payload: Box<dyn SerializeToBuffer>,
}

struct Response {
    payload: Box<dyn Deserialize>,
}

pub enum State {
    Connecting,
    Connected,
}

pub enum SessionEvent {
    Connected,    // state: Connecting -> Connected,
    Disconnected, // state: Connected -> Connecting,
    AuthFailed,   // state: Connecting -> AuthFailed,
}

pub enum EventType {
    Session,
}

#[derive(Clone, Debug)]
pub enum ShuffleMode {
    Disable,
    Enable,
    Once,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub shuffle_mode: ShuffleMode,
    pub connect_timeout: Duration,
    pub completion_warn_timeout: Duration,
}

pub type CompletionCb = dyn FnOnce(EventType, State, String); // type, state, path

impl Client {
    pub fn new(hosts: Vec<String>, config: Config) -> Self {
        let runtime = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .max_blocking_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let token = CancellationToken::new();
        let mut tasks = JoinSet::new();
        let (completion_tx, completion_rx) = mpsc::unbounded_channel::<Box<CompletionCb>>();
        tasks.spawn_blocking(async || {
            Client::task_completion(token.clone(), completion_rx).await;
        });

        let (send_tx, send_rx) = mpsc::unbounded_channel::<Request>();
        let (recv_tx, recv_rx) = mpsc::unbounded_channel::<Response>();
        tasks.spawn_blocking(async || {
            Client::task_io(
                token.clone(),
                HostProvider::new(hosts, true),
                send_rx,
                recv_rx,
                config.clone(),
            ).await;
        });

        Client {
            runtime,
            token,
            tasks,
            completion_tx,
            recv_tx,
            send_tx,
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.token.cancel();
        task::block_in_place(async { while let _ = self.tasks.join_next().await {} })
    }
}

impl Client {
    async fn connect_timeout(host: &String, timeout: Duration) -> Result<TcpStream, anyhow::Error> {
        match tokio::time::timeout(timeout, TcpStream::connect(host)).await {
            Ok(s) => match s {
                Ok(stream) => Ok(stream),
                Err(e) => Err(e.into()),
            },
            Err(_) => Err(anyhow!("connect timeout after {}ms", timeout.as_millis())),
        }
    }

    async fn prime_connection(
        conn: TcpStream,
        session_timeout: Duration,
    ) -> Result<(), anyhow::Error> {
        let req = ConnectRequest {
            protocol_version: 0, // version 0
            last_zxid_seen: 0,
            time_out: session_timeout.as_millis() as i32,
            session_id: todo!(),
            passwd: todo!(),
            read_only: todo!(),
        };
        let mut buf = bytes::BytesMut::with_capacity(size_of::<ConnectRequest>());
        req.write_buffer(&mut buf);
        conn.try_write(buf.as_ref())?;
        todo!()
    }

    async fn task_completion(
        token: CancellationToken,
        mut rx: UnboundedReceiver<Box<CompletionCb>>,
    ) {
        loop {
            select! {
                _ = token.cancelled() => {
                    break;
                },

                else => {
                    while let Some(cb) = rx.recv().await {
                        let now = Instant::now();
                        cb(EventType::Session, State::Connecting, "".to_string());
                        let elapsed = Instant::now() - now;
                        if elapsed > Duration::from_secs(1) {
                            warn!("Detected slow callback execution");
                        }
                    }
                }
            }
        }

        info!("Completion task exited");
    }

    // io task
    async fn task_io(
        token: CancellationToken,
        mut host_provider: HostProvider,
        mut to_send: UnboundedReceiver<Request>,
        to_recv: UnboundedReceiver<Response>,
        config: Config,
    ) {
        let Config {
            connect_timeout, ..
        } = config;
        let conn: Option<TcpStream> = None;

        loop {
            // try connect
            let host = host_provider.next().unwrap();
            match Client::connect_timeout(&host, connect_timeout).await {
                Ok(mut conn) => {
                    // prime connect request

                    // do io

                    let mut size_buf = [0u8; 4];

                    loop {
                        select! {
                            biased;

                            Some(req) = to_send.recv() => {
                                let buf = req.payload.to_buffer().freeze();
                                let size = buf.len() as u32;
                                let mut size_buf = Vec::new();
                                size_buf.write_u32::<BigEndian>(size).unwrap();
                                conn.try_write(&size_buf).await;
                                conn.try_write(&buf).await;
                            }

                            rsp = conn.try_read(&mut size_buf) => {
                                let size = read_u32(&size_buf);
                                let buf = read_buffer(&mut conn, size);
                                // TODO: response type dispatch, deserialize
                            }

                            _ = token.cancelled() => {
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Connect to {} failed, error: {}", host, e);
                }
            }
        }

        info!("IO task exited");
    }
}

fn read_u32(buf: &[u8; 4]) -> u32 {
    let mut cursor = io::Cursor::new(buf);
    cursor.read_u32::<BigEndian>().unwrap()
}

async fn read_buffer(mut conn: &mut TcpStream, len: u32) -> Result<Bytes, io::Error> {
    let mut buf = bytes::BytesMut::with_capacity(len as usize);
    conn.read_exact(&mut buf).await?;
    Ok(buf.freeze())
}
