use std::io::Cursor;
use std::time::Duration;

use anyhow::anyhow;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use jute::{Deserialize, SerializeToBuffer};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::{runtime, select, task};
use tokio_util::sync::CancellationToken;

use crate::host_provider::HostProvider;
use crate::messages::proto::{ConnectRequest, ConnectResponse};

pub struct Client {
    runtime: runtime::Runtime,
    token: CancellationToken,
    main_task: JoinHandle<()>,
}

pub enum State {
    Connecting,
    Connected,
}

pub enum ShuffleMode {
    Disable,
    Enable,
    Once,
}

pub struct Config {
    pub shuffle_mode: ShuffleMode,
    pub connect_timeout: Duration,
    pub session_timeout: Duration,
}

impl Client {
    pub fn new(hosts: Vec<String>, config: Config) -> Self {
        let runtime = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .max_blocking_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let token = CancellationToken::new();
        let host_provider = HostProvider::new(hosts, true);
        let main_task = task::spawn(Client::task_io(token.clone(), host_provider, config));

        Client {
            runtime,
            main_task,
            token,
        }
    }
}

struct WireFrame {
    buf: bytes::BytesMut,
}

impl From<bytes::BytesMut> for WireFrame {
    fn from(buf: bytes::BytesMut) -> Self {
        Self { buf }
    }
}

impl WireFrame {
    pub async fn send(&self, conn: &TcpStream) -> Result<(), anyhow::Error> {
        let size = self.buf.len() as u32;
        let mut size_buf: Vec<u8> = vec![];
        size_buf.write_u32::<BigEndian>(size)?;
        conn.try_write(&size_buf)?;
        conn.try_write(&self.buf)?;
        Ok(())
    }

    pub async fn recv(conn: &TcpStream) -> Result<WireFrame, anyhow::Error> {
        let mut size_buf = [0u8; 4];
        conn.try_read(&mut size_buf)?;
        let mut cursor = Cursor::new(size_buf);
        let size = cursor.read_u32::<BigEndian>()?;
        let mut buf = bytes::BytesMut::with_capacity(size as usize);
        conn.try_read(&mut buf)?;
        Ok(WireFrame { buf })
    }

    pub fn freeze(self) -> bytes::Bytes {
        self.buf.freeze()
    }
}

async fn prime_connection(
    conn: &TcpStream,
    session_timeout: Duration,
) -> Result<ConnectResponse, anyhow::Error> {
    let req = ConnectRequest {
        protocol_version: 0, // version 0
        last_zxid_seen: 0,
        time_out: session_timeout.as_millis() as i32,
        session_id: todo!(),
        passwd: todo!(),
        read_only: todo!(),
    };
    let frame: WireFrame = req.to_buffer().into();
    frame.send(&conn).await?;

    let frame = WireFrame::recv(&conn).await?;
    let rsp = ConnectResponse::from_buffer(&mut frame.freeze())?;
    Ok(rsp)
}

async fn do_io() -> Result<(), anyhow::Error> {
    match state {
        State::Connecting => {
            let host = hosts.next().unwrap();
            match Client::connect_timeout(host, config.connect_timeout).await {
                Ok(s) => {
                    state = State::Connected;
                    Self::prime_connection(&s, config.session_timeout).await?;
                    conn = Some(s);
                }
                Err(_) => {}
            }
        }

        State::Connected => {}
    }

    todo!()
}

impl Client {
    async fn connect_timeout(host: String, timeout: Duration) -> Result<TcpStream, anyhow::Error> {
        match tokio::time::timeout(timeout, TcpStream::connect(host)).await {
            Ok(s) => match s {
                Ok(stream) => Ok(stream),
                Err(e) => Err(e.into()),
            },
            Err(_) => Err(anyhow!("connect timeout after {}ms", timeout.as_millis())),
        }
    }

    async fn task_io(token: CancellationToken, mut hosts: HostProvider, config: Config) {
        let mut state = State::Connecting;
        let mut conn: Option<TcpStream> = None;

        loop {
            select! {
                _ = token.cancelled() => {
                    break;
                },

                else => {
                    let next_state = match do_io().await {
                        Ok(_) => todo!(),
                        Err(_) => todo!(),
                    };
                    state = next_state;
                }
            }
        }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.token.cancel();
        task::block_in_place(|| {
            self.runtime.handle().block_on(&mut self.main_task).unwrap();
        })
    }
}
