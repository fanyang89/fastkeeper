use std::future::Future;
use std::io;
use std::time::Duration;

use anyhow::{anyhow, Error};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::error::Elapsed;
use tokio::time::{sleep, Instant, Timeout};
use tokio::{runtime, select, task};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

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

impl SerializeToBuffer for Request {
    fn to_buffer(&self) -> BytesMut {
        let mut b = BytesMut::new();
        let buf = self.payload.to_buffer().freeze();
        let size = buf.len() as u32;
        b.put_u32(size);
        b.put(buf);
        b
    }
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
    pub session_timeout: Duration,
    pub read_only: bool,
}

#[derive(Clone, Debug)]
pub enum IOInterest {
    Read,
    Write,
    PrimeConnect,
    PrimeConnectResponse,
    Reconnect,
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
            )
            .await;
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
        conn: &TcpStream,
        last_zxid: i64,
        session_id: i64,
        passwd: &jute::Buffer,
        config: Config,
    ) -> Result<(), anyhow::Error> {
        let Config {
            session_timeout,
            read_only,
            ..
        } = config;
        let req = ConnectRequest {
            protocol_version: 0, // version 0
            last_zxid_seen: last_zxid,
            time_out: session_timeout.as_millis() as i32,
            session_id,
            passwd: passwd.clone(),
            read_only,
        };
        let mut buf = BytesMut::with_capacity(size_of::<ConnectRequest>());
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
                            warn!("Slow callback execution detected");
                        }
                    }
                }
            }
        }

        info!("Completion task exited");
    }

    async fn do_read(mut conn: &mut TcpStream) -> Result<(), anyhow::Error> {
        loop {
            if !conn.readable() {
                break;
            }
            let mut size_buf = [0u8; 4];
            match conn.read_exact(&mut size_buf).await {
                Ok(_) => {
                    let mut cursor = io::Cursor::new(size_buf);
                    let size = cursor.read_u32::<BigEndian>().unwrap();
                    let mut buf = BytesMut::with_capacity(size as usize);
                    conn.read_exact(&mut buf).await?;
                    let b = buf.freeze();
                }
                Err(_) => {}
            }
            // TODO: response type dispatch, deserialize
        }
        Ok(())
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
            connect_timeout,
            session_timeout,
            ..
        } = config;
        let mut interest = IOInterest::PrimeConnect;
        let mut last_zxid: i64 = 0;
        let mut session_id: i64 = 0;
        let mut passwd: jute::Buffer = Vec::with_capacity(16);

        loop {
            // try connect
            let host = host_provider.next().unwrap();
            let connect_start = Instant::now();
            info!("Connecting to {}", host);
            match Client::connect_timeout(&host, connect_timeout).await {
                Ok(mut conn) => {
                    while !token.is_cancelled() {
                        match interest {
                            IOInterest::Reconnect => {
                                break;
                            }

                            IOInterest::PrimeConnect => {
                                match Client::prime_connection(
                                    &conn,
                                    last_zxid,
                                    session_id,
                                    &passwd,
                                    config.clone(),
                                )
                                .await
                                {
                                    Ok(_) => {
                                        interest = IOInterest::PrimeConnectResponse;
                                    }
                                    Err(_) => {
                                        interest = IOInterest::PrimeConnect;
                                    }
                                }
                            }

                            IOInterest::PrimeConnectResponse => {
                                interest = IOInterest::Write;
                            }

                            IOInterest::Read => match tokio::time::timeout(
                                session_timeout / 3 * 2,
                                Self::do_read(&mut conn),
                            )
                            .await
                            {
                                Ok(r) => match r {
                                    Ok(_) => {
                                        interest = IOInterest::Write;
                                    }
                                    Err(e) => {
                                        error!("Read failed, {}", e);
                                        interest = IOInterest::Reconnect;
                                    }
                                },
                                Err(e) => {
                                    error!("Read timeout, {}", e);
                                    interest = IOInterest::Reconnect;
                                }
                            },

                            IOInterest::Write => {
                                let size = to_send.len();
                                for _ in 0..size {
                                    if let Some(req) = to_send.recv().await {
                                        let buf = req.to_buffer().freeze();
                                        conn.try_write(&buf).await;
                                    }
                                }
                                interest = IOInterest::Read;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Connect to {} failed, error: {}", host, e);
                    let connect_elapsed = Instant::now() - connect_start;
                    let connect_step = session_timeout / 3;
                    if connect_step > connect_elapsed {
                        sleep(connect_elapsed - connect_elapsed).await;
                    }
                    interest = IOInterest::PrimeConnect;
                }
            }
        }

        info!("IO task exited");
    }
}
