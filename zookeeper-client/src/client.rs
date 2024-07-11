use crate::host_provider::HostProvider;
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
}

impl Client {
    pub fn new(hosts: Vec<String>) -> Self {
        let runtime = runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .max_blocking_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let token = CancellationToken::new();
        let token2 = token.clone();
        let host_provider = HostProvider::new(hosts, true);
        let main_task = task::spawn(async move {
            let state = State::Connecting;
            loop {
                select! {
                    _ = token.cancelled() => {
                        break;
                    },

                    else => {
                        match state {
                            State::Connecting => {}
                            State::Connected => {}
                        }
                    }
                }
            }
        });

        Client {
            runtime,
            main_task,
            token: token2,
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
