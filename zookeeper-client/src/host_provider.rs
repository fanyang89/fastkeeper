use rand::prelude::SliceRandom;

pub struct HostProvider {
    hosts: Vec<String>,
    next: usize,
}

impl HostProvider {
    pub fn new(mut hosts: Vec<String>, shuffle: bool) -> Self {
        if shuffle {
            hosts.shuffle(&mut rand::thread_rng());
        }
        Self { hosts, next: 0 }
    }
}

impl Iterator for HostProvider {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let r = &self.hosts[self.next % self.hosts.len()];
        self.next += 1;
        Some(r.clone())
    }
}
