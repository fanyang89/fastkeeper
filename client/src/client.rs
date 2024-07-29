use std::fmt::Display;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{self, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Error};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rand::prelude::IteratorRandom;
use rand::{random, thread_rng, Rng};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::error::Elapsed;
use tokio::time::{sleep, Instant, Timeout};
use tokio::{net, runtime, select, task, time};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, trace, warn};

use crate::config::Config;
use crate::error::ClientError;
use crate::event;
use crate::event::Event;
use crate::frame::FrameReadWriter;
use crate::hosts::{Hosts, ShuffleMode};
use crate::messages::proto::{
    ConnectRequest, ConnectResponse, GetDataRequest, GetDataResponse, ReplyHeader, RequestHeader,
};
use crate::messages::{self, data, proto, OpCode, RequestBody};
use crate::request::Request;
use crate::response::Response;
use crate::state::ProcessState;
use jute::{Deserialize, JuteError, Serialize, SerializeToBuffer};

pub type OnEvent = dyn Fn(event::Type, event::State, String) + Send + Sync + 'static; // type, state, path

pub struct Client {
    // runtime: Arc<Runtime>,
    token: CancellationToken,
    req_tx: UnboundedSender<Request>,
    state: ProcessState,
    pub completion_task: JoinHandle<()>,
    pub io_task: JoinHandle<()>,
}

impl Client {
    pub fn new(hosts: &Vec<String>, cb: &'static OnEvent, config: Config) -> Result<Self, Error> {
        if hosts.is_empty() {
            return Err(ClientError::InvalidConfig.into());
        }

        // let runtime = Arc::new(
        //     runtime::Builder::new_multi_thread()
        //         .worker_threads(config.worker_threads)
        //         .max_blocking_threads(config.max_blocking_threads)
        //         .enable_all()
        //         .build()
        //         .unwrap(),
        // );
        let token = CancellationToken::new();
        let state = ProcessState::new();

        // send request from guest to task-io
        let (req_tx, req_rx) = mpsc::unbounded_channel::<Request>();
        // send event from task-io to task-completion
        let (event_tx, event_rx) = mpsc::unbounded_channel::<Event>();
        // send waker from task-io to task-completion
        let (wake_tx, wake_rx) = mpsc::unbounded_channel::<Request>();
        // send response from task-io to task-completion
        let (rsp_tx, rsp_rx) = mpsc::unbounded_channel::<Response>();

        let completion_task = tokio::spawn(Client::task_completion(
            // runtime.clone(),
            token.clone(),
            cb,
            config.clone(),
            event_rx,
            wake_rx,
            rsp_rx,
        ));
        let io_task = tokio::spawn(Client::task_io(
            config.clone(),
            state.clone(),
            token.clone(),
            Hosts::new(hosts.clone(), &config.shuffle_mode),
            req_rx,
            wake_tx,
            event_tx.clone(),
            rsp_tx,
        ));

        Ok(Client {
            req_tx,
            // runtime,
            token,
            state,
            completion_task,
            io_task,
        })
    }

    #[async_backtrace::framed]
    async fn connect(host: &String) -> Result<TcpStream, Error> {
        let hosts = net::lookup_host(host).await?;
        let addr: SocketAddr = hosts
            .choose(&mut thread_rng())
            .ok_or_else(|| anyhow!("no resolve results"))?;
        Ok(TcpStream::connect(addr).await?)
    }

    #[async_backtrace::framed]
    async fn connect_timeout(host: &String, timeout: Duration) -> Result<TcpStream, Error> {
        Ok(tokio::time::timeout(timeout, Client::connect(host))
            .await
            .map_err(|_| anyhow!("connect timeout after {}ms", timeout.as_millis()))??)
    }

    #[async_backtrace::framed]
    async fn prime_connection(
        conn: &TcpStream,
        state: &ProcessState,
        config: &Config,
    ) -> Result<(), ClientError> {
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

    #[async_backtrace::framed]
    async fn prime_connection_response(
        conn: &mut TcpStream,
        state: &mut ProcessState,
    ) -> Result<(), ClientError> {
        let mut frame = conn.read_frame().await?;
        let rsp = ConnectResponse::from_buffer(&mut frame)?;
        let old_id = state.session_id();
        let new_id = rsp.session_id;
        if old_id != 0 && new_id != old_id {
            Err(ClientError::SessionExpired.into())
        } else {
            state.update_session(&rsp);
            info!(
                "Session established, timeout: {}ms, session id: {:#x}",
                rsp.timeout, rsp.session_id
            );
            Ok(())
        }
    }

    #[async_backtrace::framed]
    async fn task_completion(
        // runtime: Arc<Runtime>,
        token: CancellationToken,
        on_event: &'static OnEvent,
        config: Config,
        mut event_rx: UnboundedReceiver<Event>,
        mut wake_rx: UnboundedReceiver<Request>,
        mut rsp_rx: UnboundedReceiver<Response>,
    ) {
        let Config {
            completion_warn_timeout,
            ..
        } = config;
        loop {
            select! {
                _ = token.cancelled() => {
                    break;
                },

                event = event_rx.recv() => {
                    if let Some(e) = event {
                        let now = Instant::now();
                        task::spawn_blocking(|| on_event(e.r#type, e.state, e.path)).await.unwrap();
                        let elapsed = Instant::now() - now;
                        if elapsed > completion_warn_timeout {
                            warn!("Slow callback execution({}ms) detected", completion_warn_timeout.as_millis());
                        }
                    }
                }

                response = rsp_rx.recv() => {
                    if let Some(rsp) = response {
                        let req = wake_rx.recv().await.unwrap();
                        req.wake.unwrap().send(rsp).unwrap();
                    }
                }
            }
        }

        info!("Completion task exited");
    }

    #[async_backtrace::framed]
    async fn read_dispatch_response_timeout(
        mut conn: &mut TcpStream,
        timeout: Duration,
    ) -> Result<Response, ClientError> {
        let mut buf = tokio::time::timeout(timeout, conn.read_frame()).await??;
        Ok(Response::from_buffer(&mut buf)?)
    }

    #[async_backtrace::framed]
    async fn read_response(mut conn: &mut TcpStream) -> Result<Response, Error> {
        // read buffer from socket
        let mut size_buf = [0u8; 4];
        conn.read_exact(&mut size_buf).await?;

        let mut cursor = io::Cursor::new(size_buf);
        let size = ReadBytesExt::read_i32::<BigEndian>(&mut cursor).unwrap();
        trace!("Reading response, expected {} bytes", size);

        let mut buf = vec![0u8; size as usize];
        let n = conn.read_exact(&mut buf).await? as i32;
        trace!("Read {} bytes", n);
        Response::from_buffer(&mut Bytes::from(buf)).map_err(|e| e.into())
    }
    // 100.108.54.61

    #[async_backtrace::framed]
    async fn send_all(
        mut conn: &mut TcpStream,
        mut to_send: &mut UnboundedReceiver<Request>,
        mut to_recv: &mut UnboundedSender<Request>,
    ) -> Result<(), Error> {
        // packet layout:
        // frame size:   u32 = m + n
        // frame header: [u8; m] m = 8
        // frame body:   [u8; n]
        // --
        if let Some(req) = to_send.recv().await {
            let req_buf = req.to_buffer();
            let mut buf = BytesMut::with_capacity(size_of::<u32>() + req_buf.len());
            buf.put_u32(req_buf.len() as u32);
            buf.put(req_buf);
            conn.write_all_buf(&mut buf).await?;
            to_recv.send(req)?;
        }
        Ok(())
    }

    #[async_backtrace::framed]
    async fn process_loop(
        mut conn: &mut TcpStream,
        state: &mut ProcessState,
        config: Config,
        token: CancellationToken,
        to_send: &mut UnboundedReceiver<Request>,
        to_recv: &mut UnboundedSender<Request>,
        completion_tx: &mut UnboundedSender<Event>,
        rsp_tx: &mut UnboundedSender<Response>,
    ) -> Result<(), ClientError> {
        let Config {
            session_timeout,
            send_session_close_timeout,
            wait_close_timeout,
            ..
        } = config;

        let read_timeout = session_timeout / 3 * 2;
        let write_timeout = session_timeout / 3;
        trace!(
            "Read timeout: {:?}, write timeout: {:?}",
            read_timeout,
            write_timeout
        );

        // send connection request
        if let Err(e) = Client::prime_connection(&conn, &state, &config).await {
            return Err(e);
        }
        state.update_last_send();

        // read connection response
        tokio::time::timeout(read_timeout, Client::prime_connection_response(conn, state))
            .await??;

        // start ping timer
        let mut interval = tokio::time::interval(write_timeout);

        // io loop
        let mut interest = Interest::WRITABLE;
        loop {
            select! {
                _ = token.cancelled() => {
                    break;
                }

                ready = conn.ready(interest) => {
                    match ready {
                        Ok(ready) => {
                            if ready.is_readable() {
                                trace!("is_readable");
                                match Self::read_dispatch_response_timeout(&mut conn, read_timeout).await {
                                    Ok(rsp) => {
                                        state.update_last_recv();
                                        rsp_tx.send(rsp);
                                    },
                                    Err(e) => {
                                        // connection loss
                                        error!("Read response failed, last recv: {:?}, {}", state.last_recv(), e);
                                        break;
                                    }
                                }
                                interest = Interest::WRITABLE;
                                trace!("is_readable, done");
                            } else if ready.is_writable() {
                                // TODO: send ping here
                                trace!("is_writable");
                                if let Err(e) = time::timeout(write_timeout, Client::send_all(conn, to_send, to_recv)).await? {
                                    error!("Write failed, {}", e);
                                } else {
                                    state.update_last_send();
                                }
                                interest = Interest::READABLE;
                                trace!("is_writable, done");
                            }
                        }
                        Err(e) => {
                            trace!("Connection is gone, {}", e);
                            break;
                        }
                    }
                }

                _ = interval.tick() => {
                    let now = Instant::now();
                    if now - state.last_send() >= write_timeout {
                        Client::send_ping(&conn).await?;
                        state.update_last_send();
                    }
                }
            }
        }

        // send session close
        trace!("Closing session");
        time::timeout(
            send_session_close_timeout,
            Self::close_session(conn, state.get_xid()),
        )
        .await??;

        // wait for connection readable
        time::timeout(wait_close_timeout, conn.readable()).await??;
        trace!("Session closed");

        Ok(())
    }

    #[async_backtrace::framed]
    async fn send_ping(conn: &TcpStream) -> Result<(), ClientError> {
        let req = RequestHeader {
            xid: -2, // TODO replace magic number
            r#type: OpCode::OpPing.into(),
        };
        conn.write_frame(&req.to_buffer().freeze()).await?;
        trace!("Request ping sent");
        Ok(())
    }

    #[async_backtrace::framed]
    async fn task_io(
        config: Config,
        mut state: ProcessState,
        token: CancellationToken,
        mut host_provider: Hosts,
        mut to_send: UnboundedReceiver<Request>,
        mut to_recv: UnboundedSender<Request>,
        mut completion_tx: UnboundedSender<Event>,
        mut rsp_tx: UnboundedSender<Response>,
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

                connect_result = Client::connect_timeout(&host, connect_timeout) => {
                    match connect_result {
                        Ok(mut conn) => match Client::process_loop(
                            &mut conn, &mut state, config.clone(), token.clone(), &mut to_send, &mut to_recv, &mut completion_tx, &mut rsp_tx,
                        ).await {
                            Ok(_) => info!("Session closed"),
                            Err(e) => {
                                error!("Process error, {}", e);
                                if let ClientError::SessionExpired = e {
                                    token.cancel();
                                    break;
                                }
                            },
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
    ) -> Result<Output, Error> {
        let (wake_tx, wake_rx) = oneshot::channel::<Response>();
        self.req_tx
            .send(Request {
                header,
                body,
                wake: Some(wake_tx),
            })
            .unwrap();
        let rsp: Response = wake_rx.await?;
        Ok(Output::from_buffer(&mut rsp.body.unwrap())?)
    }

    pub async fn close_session(mut conn: &mut TcpStream, xid: i32) -> Result<(), ClientError> {
        conn.writable().await?;
        let req = Request {
            header: RequestHeader {
                xid,
                r#type: OpCode::OpClose.into(),
            },
            body: None,
            wake: None,
        };
        conn.write_frame(&req.to_buffer().freeze()).await?;
        Ok(())
    }

    pub async fn get(&mut self, path: &str, watch: bool) -> Result<GetDataResponse, Error> {
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
    }
}
