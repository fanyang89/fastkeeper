use crate::host_provider::HostProvider;
use crate::messages::proto::{AuthPacket, ConnectRequest};
use anyhow::anyhow;
use jute::Serialize;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::{runtime, select, task};
use tokio_util::sync::CancellationToken;

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
        let main_task = task::spawn(Client::do_io(token.clone(), host_provider, config));

        Client {
            runtime,
            main_task,
            token,
        }
    }
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

    async fn do_io(token: CancellationToken, mut hosts: HostProvider, config: Config) {
        let mut state = State::Connecting;
        let mut conn: Option<TcpStream> = None;

        loop {
            select! {
                _ = token.cancelled() => {
                    break;
                },

                else => {
                    match state {
                        State::Connecting => {
                            let host = hosts.next().unwrap();
                            match Client::connect_timeout(host, config.connect_timeout).await {
                                Ok(s) => {
                                    state = State::Connected;
                                    conn = Some(s);
                                }
                                Err(_) => {}
                            }
                        }

                        State::Connected => {}
                    }
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
