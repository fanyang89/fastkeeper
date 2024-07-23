use std::fmt::Display;
use std::future::Future;
use std::io;
use std::sync::atomic::{self, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::anyhow;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rand::{random, Rng};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::error::Elapsed;
use tokio::time::{sleep, Instant, Timeout};
use tokio::{runtime, select, task, time};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace, warn};

use crate::config::Config;
use crate::error::ClientError;
use crate::event;
use crate::frame::FrameReadWriter;
use crate::hosts::{Hosts, ShuffleMode};
use crate::messages::proto::{
    ConnectRequest, ConnectResponse, GetDataRequest, GetDataResponse, ReplyHeader, RequestHeader,
};
use crate::messages::{self, data, proto, OpCode, RequestBody};
use crate::request::Request;
use crate::response::Response;
use crate::state::ProcessState;
use jute::{Deserialize, Serialize, SerializeToBuffer};

pub type OnEvent = Box<dyn Fn(event::Type, event::State, String) + Send>; // type, state, path

pub struct Client {
    runtime: runtime::Runtime,
    token: CancellationToken,
    tasks: JoinSet<()>,
    req_tx: UnboundedSender<Request>,
    state: ProcessState,
}

impl Client {
    pub fn new(hosts: &Vec<String>, cb: OnEvent, config: Config) -> Result<Self, anyhow::Error> {
        if hosts.is_empty() {
            return Err(ClientError::InvalidConfig.into());
        }

        let runtime = runtime::Builder::new_multi_thread()
            .worker_threads(config.worker_threads)
            .max_blocking_threads(config.max_blocking_threads)
            .enable_all()
            .build()
            .unwrap();
        let token = CancellationToken::new();
        let mut tasks = JoinSet::new();
        let state = ProcessState::new();
        // send event from task-io to task-completion
        let (event_tx, event_rx) = mpsc::unbounded_channel::<event::Event>();
        // send request from guest to task-io
        let (req_tx, req_rx) = mpsc::unbounded_channel::<Request>();
        // send response from task-io to task-completion
        let (rsp_tx, rsp_rx) = mpsc::unbounded_channel::<(oneshot::Sender<Response>, Response)>();

        tasks.spawn_blocking(Client::task_completion(token.clone(), event_rx, rsp_rx, cb));

        tasks.spawn(Client::task_io(
            config.clone(),
            state.clone(),
            token.clone(),
            Hosts::new(hosts.clone(), &config.shuffle_mode),
            req_rx,
            rsp_tx,
            event_tx.clone(),
        ));

        Ok(Client {
            runtime,
            token,
            tasks,
            req_tx,
            state,
        })
    }

    async fn connect_timeout(host: &String, timeout: Duration) -> Result<TcpStream, anyhow::Error> {
        // TODO: async DNS resolve
        // https://docs.rs/tokio/latest/tokio/net/fn.lookup_host.html
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
        state: &ProcessState,
        config: &Config,
    ) -> Result<(), anyhow::Error> {
        let state = state.inner();
        let req = ConnectRequest {
            protocol_version: 0, // version 0
            last_zxid_seen: state.last_zxid,
            time_out: config.session_timeout.as_millis() as i32,
            session_id: state.session_id,
            passwd: state.passwd,
            read_only: config.read_only,
        };
        let buf = req.to_buffer().freeze();
        conn.write_frame(&buf).await?;
        Ok(())
    }

    async fn prime_connection_response(
        conn: &mut TcpStream,
        state: &mut ProcessState,
        config: &Config,
    ) -> Result<(), anyhow::Error> {
        let mut frame = conn.read_frame().await?;
        let rsp = ConnectResponse::from_buffer(&mut frame)?;
        let old_id = state.session_id();
        let new_id = rsp.session_id;
        if old_id != 0 && new_id != old_id {
            // session expired
            // TODO: send session expired event, close connection
        } else {
            state.update_session(&rsp);
            info!(
                "Session established, timeout: {}ms, session id: {:#x}",
                rsp.timeout, rsp.session_id
            );
        }
        Ok(())
    }

    async fn task_completion(
        token: CancellationToken,
        mut event_rx: UnboundedReceiver<event::Event>,
        mut rsp_rx: UnboundedReceiver<(oneshot::Sender<Response>, Response)>,
        on_event: OnEvent,
    ) {
        loop {
            select! {
                _ = token.cancelled() => {
                    break;
                },

                event = event_rx.recv() => {
                    if let Some(e) = event {
                        let now = Instant::now();
                        on_event(e.r#type, e.state, e.path);
                        let elapsed = Instant::now() - now;
                        if elapsed > Duration::from_secs(1) {
                            warn!("Slow callback execution detected");
                        }
                    }
                }

                r = rsp_rx.recv() => {
                    if let Some((wake, rsp)) = r {
                        wake.send(rsp).unwrap();
                    }
                }
            }
        }

        info!("Completion task exited");
    }

    async fn read_response(mut conn: &mut TcpStream) -> Result<(), anyhow::Error> {
        // read buffer from socket
        let mut size_buf = [0u8; 4];
        match conn.read_exact(&mut size_buf).await {
            Ok(_) => {
                let mut cursor = io::Cursor::new(size_buf);
                let size = ReadBytesExt::read_u32::<BigEndian>(&mut cursor).unwrap();
                let mut buf = BytesMut::with_capacity(size as usize);
                conn.read_exact(&mut buf).await?;
                let b = buf.freeze();
                todo!()
            }
            Err(e) => {
                return Err(e.into());
            }
        }
        // TODO: response type dispatch, deserialize
        Ok(())
    }

    async fn drain_send_queue(
        mut conn: &mut TcpStream,
        timeout: Duration,
        mut to_send: &mut UnboundedReceiver<Request>,
    ) -> Result<(), anyhow::Error> {
        let size = to_send.len();
        for _ in 0..size {
            if let Some(req) = to_send.recv().await {
                let buf = req.to_buffer().freeze();
                conn.try_write(&buf)?;
            }
        }
        Ok(())
    }

    async fn process_loop(
        mut conn: &mut TcpStream,
        state: &mut ProcessState,
        config: Config,
        token: CancellationToken,
        mut to_send: &mut UnboundedReceiver<Request>,
    ) -> Result<(), anyhow::Error> {
        let Config {
            session_timeout,
            send_session_close_timeout,
            wait_close_timeout,
            ..
        } = config;

        let read_timeout = session_timeout / 3 * 2;
        let write_timeout = session_timeout / 3;

        // send connection request
        if let Err(e) = Client::prime_connection(&conn, &state, &config).await {
            return Err(e);
        }
        state.update_last_send();

        // read connection response
        tokio::time::timeout(
            read_timeout,
            Client::prime_connection_response(conn, state, &config),
        )
        .await??;

        // start ping timer
        let mut interval = tokio::time::interval(write_timeout);

        // io loop
        loop {
            select! {
                _ = token.cancelled() => {
                    break;
                }

                _ = conn.readable() => {
                    if let Err(e) = tokio::time::timeout(read_timeout, Client::read_response(&mut conn)).await? {
                        error!("Read failed, {}", e);
                    }
                }

                _ = interval.tick() => {
                    let now = Instant::now();
                    if now - state.last_send() >= write_timeout {
                        Client::send_ping(&conn).await?;
                        state.update_last_send();
                    }
                }

                _ = conn.writable() => {
                    if let Err(e) = Self::drain_send_queue(&mut conn, write_timeout, &mut to_send).await {
                        error!("Write failed, {}", e);
                    }
                    state.update_last_send();
                }
            }
        }

        // send session close
        time::timeout(
            send_session_close_timeout,
            Self::close_session(state.get_xid(), conn),
        )
        .await??;

        // wait for connection readable
        time::timeout(wait_close_timeout, conn.readable()).await??;

        Ok(())
    }

    async fn send_ping(conn: &TcpStream) -> Result<(), anyhow::Error> {
        let req = RequestHeader {
            xid: -2, // TODO replace magic number
            r#type: OpCode::OpPing.into(),
        };
        conn.write_frame(&req.to_buffer().freeze()).await?;
        trace!("Request ping sent");
        Ok(())
    }

    // io task
    async fn task_io(
        config: Config,
        mut state: ProcessState,
        token: CancellationToken,
        mut host_provider: Hosts,
        mut to_send: UnboundedReceiver<Request>,
        mut to_recv: UnboundedSender<(Sender<Response>, Response)>,
        mut completion_tx: UnboundedSender<event::Event>,
    ) {
        let Config {
            connect_timeout,
            session_timeout,
            ..
        } = config;
        let (sent_tx, sent_rx) = mpsc::unbounded_channel::<RequestBody>();

        loop {
            let host = host_provider.next().unwrap();
            let connect_start = Instant::now();
            info!("Connecting to {}", host);

            select! {
                _ = token.cancelled() => {
                    break;
                }

                r = Client::connect_timeout(&host, connect_timeout) => {
                    match r {
                        Ok(mut conn) => match Client::process_loop(&mut conn, &mut state, config.clone(), token.clone(), &mut to_send).await {
                            Ok(_) => todo!(),
                            Err(_) => todo!(),
                        },

                        Err(e) => {
                            warn!("Connect failed, error: {}", e);
                            let connect_elapsed = Instant::now() - connect_start;
                            let connect_step = config.connect_attempt.unwrap_or(session_timeout / 3);
                            if connect_step > connect_elapsed {
                                sleep(connect_step - connect_elapsed).await;
                            }
                        }
                    }
                }
            }
        }

        info!("IO task exited");
    }

    fn get_xid(&mut self) -> i32 {
        self.state.get_xid()
    }
}

impl Client {
    pub async fn submit_request<Output: Deserialize>(
        &mut self,
        header: RequestHeader,
        body: Option<RequestBody>,
    ) -> Result<Output, anyhow::Error> {
        let (tx, rx) = oneshot::channel::<Response>();
        let req = Request {
            header,
            payload: body,
            wake: Some(tx),
        };
        self.req_tx.send(req)?;
        let rsp = rx.await?;
        Ok(Output::from_buffer(&mut rsp.payload.unwrap())?)
    }

    pub async fn close_session(xid: i32, mut conn: &mut TcpStream) -> Result<(), anyhow::Error> {
        conn.writable().await?;
        let req = Request {
            header: RequestHeader {
                xid,
                r#type: OpCode::OpClose.into(),
            },
            payload: None,
            wake: None,
        };
        conn.write_frame(&req.to_buffer().freeze()).await?;
        Ok(())
    }

    pub async fn get(&mut self, path: &str, watch: bool) -> Result<GetDataResponse, anyhow::Error> {
        let xid = self.get_xid();
        self.submit_request(
            RequestHeader::new(xid, OpCode::OpGetData),
            Some(RequestBody::GetDataRequest(GetDataRequest {
                path: path.to_string(),
                watch,
            })),
        )
        .await
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.token.cancel();
        task::block_in_place(async || while let _ = self.tasks.join_next().await {});
    }
}
