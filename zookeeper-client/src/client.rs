use rand::prelude::SliceRandom;
use tokio::runtime;

pub struct Client {
    runtime: runtime::Runtime,
    hosts: HostProvider,
}

pub struct HostProvider {
    hosts: Vec<String>,
    next: usize,
}

impl HostProvider {
    pub fn new(mut hosts: Vec<String>) -> Self {
        hosts.shuffle(&mut rand::thread_rng());
        Self { hosts, next: 0 }
    }

    pub fn resolve(&mut self) {}
}

impl Iterator for HostProvider {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Client {
    pub fn new(hosts: Vec<String>) -> Self {
        let client = Client {
            runtime: runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .max_blocking_threads(1)
                .enable_all()
                .build()
                .unwrap(),
            hosts: HostProvider::new(hosts),
        };
        client.connect();
        client
    }

    fn connect(&self) {}
}
