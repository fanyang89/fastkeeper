use raft::eraftpb::{Entry, Snapshot};
use raft::storage::Storage;
use raft::{GetEntriesContext, RaftState};

pub struct SledStorage {}

impl SledStorage {
    pub fn new() -> SledStorage {
        SledStorage {}
    }
}

impl Storage for SledStorage {
    fn initial_state(&self) -> raft::Result<RaftState> {
        todo!()
    }

    fn entries(
        &self,
        low: u64,
        high: u64,
        max_size: impl Into<Option<u64>>,
        context: GetEntriesContext,
    ) -> raft::Result<Vec<Entry>> {
        todo!()
    }

    fn term(&self, idx: u64) -> raft::Result<u64> {
        todo!()
    }

    fn first_index(&self) -> raft::Result<u64> {
        todo!()
    }

    fn last_index(&self) -> raft::Result<u64> {
        todo!()
    }

    fn snapshot(&self, request_index: u64, to: u64) -> raft::Result<Snapshot> {
        todo!()
    }
}
