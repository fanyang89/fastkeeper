use rand::prelude::SliceRandom;

use crate::client::ShuffleMode;

pub struct HostProvider {
    hosts: Vec<String>,
    next: usize,
    shuffle_mode: ShuffleMode,
}

impl HostProvider {
    pub fn new(mut hosts: Vec<String>, shuffle_mode: &ShuffleMode) -> Self {
        let mut p = Self {
            hosts,
            next: 0,
            shuffle_mode: shuffle_mode.clone(),
        };
        if p.shuffle_mode != ShuffleMode::Disable {
            p.shuffle();
        }
        p
    }

    pub fn shuffle(&mut self) {
        self.hosts.shuffle(&mut rand::thread_rng());
    }
}

impl Iterator for HostProvider {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.next;
        self.next += 1;
        if self.next >= self.hosts.len() {
            self.next = 0;
            if self.shuffle_mode == ShuffleMode::Enable {
                self.shuffle();
            }
        }
        Some(self.hosts[i].clone())
    }
}
