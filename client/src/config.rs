use crate::hosts::ShuffleMode;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    pub shuffle_mode: ShuffleMode,
    pub connect_timeout: Duration,
    pub connect_attempt: Option<Duration>,
    pub completion_warn_timeout: Duration,
    pub session_timeout: Duration,
    pub read_only: bool,
    pub worker_threads: usize,
    pub(crate) max_blocking_threads: usize,
    pub send_session_close_timeout: Duration,
    pub wait_close_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shuffle_mode: ShuffleMode::Enable,
            connect_timeout: Duration::from_secs(1),
            connect_attempt: Some(Duration::from_secs(1)),
            completion_warn_timeout: Duration::from_secs(1),
            session_timeout: Duration::from_secs(6),
            read_only: false,
            worker_threads: 1,
            max_blocking_threads: 1,
            send_session_close_timeout: Duration::from_secs(3),
            wait_close_timeout: Duration::from_millis(1500),
        }
    }
}
