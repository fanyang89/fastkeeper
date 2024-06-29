struct Client {
    runtime: tokio::runtime::Runtime,
}

impl Client {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .max_blocking_threads(1)
            .enable_all()
            .build()
            .unwrap();
        Client { runtime }
    }

    pub fn close(self) {
        self.runtime.shutdown_background();
    }
}
