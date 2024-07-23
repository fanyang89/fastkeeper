use rand::prelude::SliceRandom;

#[derive(Clone, Debug, PartialEq)]
pub enum ShuffleMode {
    Disable,
    Enable,
    Once,
}

pub struct Hosts {
    hosts: Vec<String>,
    next: usize,
    shuffle_mode: ShuffleMode,
    shuffled: bool,
}

impl Hosts {
    pub fn new(mut hosts: Vec<String>, shuffle_mode: &ShuffleMode) -> Self {
        let mut provider = Self {
            hosts,
            next: 0,
            shuffle_mode: shuffle_mode.clone(),
            shuffled: false,
        };
        provider.shuffle();
        provider
    }

    pub fn shuffle(&mut self) {
        if match self.shuffle_mode {
            ShuffleMode::Disable => false,
            ShuffleMode::Enable => true,
            ShuffleMode::Once => !self.shuffled,
        } {
            self.hosts.shuffle(&mut rand::thread_rng());
        }
        self.shuffled = self.shuffle_mode == ShuffleMode::Once
    }
}

impl Iterator for Hosts {
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
