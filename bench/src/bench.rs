use crate::error::BenchError;
use crate::cli::ConnectOption;
use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fmt::{Display, Formatter};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use zookeeper::{Acl, CreateMode, WatchedEvent, ZkError, ZooKeeper, ZooKeeperExt};

pub(crate) struct LoggingWatcher;

impl zookeeper::Watcher for LoggingWatcher {
    fn handle(&self, event: WatchedEvent) {
        // TODO: handle session expired and disconnect event
        log::info!("Watcher receive new event: {:?}", event);
    }
}

#[derive(Clone, Debug)]
pub struct BenchOption {
    pub connect_option: ConnectOption,
    pub iteration: u32,
    pub threads: u32,
    pub ephemeral: bool,
    pub node_value: Vec<u8>,
    pub prefix: String,
}

pub struct BenchResult {
    pub elapsed: Duration,
    pub tps: f32,
    pub qps: f32,
}

impl Display for BenchResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TPS: {:.2}, QPS: {:.2}, Elapsed: {:.2}ms",
            self.tps,
            self.qps,
            self.elapsed.as_millis()
        )
    }
}

fn new_progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-")
}

fn skip_last<T>(mut iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    let last = iter.next();
    iter.scan(last, |state, item| std::mem::replace(state, Some(item)))
}

pub fn base_node(prefix: &String) -> &str {
    let p = prefix.rfind('/').unwrap_or_default();
    if p != 0 {
        &prefix[0..p]
    } else {
        prefix
    }
}

pub fn delete_prefix(zk: &ZooKeeper, prefix: &String) -> Result<()> {
    let prefix = base_node(prefix);
    match zk.delete_recursive(prefix) {
        Ok(_) => Ok(()),
        Err(e) if e == ZkError::NoNode => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn create_prefix(zk: &ZooKeeper, prefix: &String) -> Result<()> {
    let mut s = String::new();
    for p in skip_last(prefix.split("/")) {
        if p.is_empty() {
            continue;
        }

        s.push('/');
        s.push_str(p);

        match zk.create(
            s.as_str(),
            Vec::new(),
            Acl::open_unsafe().clone(),
            CreateMode::Persistent,
        ) {
            Ok(_) => {}
            Err(e) if e == ZkError::NodeExists => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

pub fn prepare(prefix: &String, opt: &ConnectOption) -> Result<()> {
    let zk = opt.connect()?;
    delete_prefix(&zk, prefix)?;
    create_prefix(&zk, prefix)?;
    Ok(())
}

fn do_bench<T>(opt: &BenchOption, bench_fn: T) -> Result<Duration>
where
    T: Fn(u32, ProgressBar, &BenchOption) -> Result<()> + Send + Sync + Copy,
{
    let bar = MultiProgress::new();
    let start = Instant::now();
    let mut is_err = false;
    thread::scope(|s| {
        let mut threads = Vec::new();
        for tid in 0..opt.threads {
            let pb = bar.add(ProgressBar::new((opt.iteration / opt.threads) as u64));
            pb.set_style(new_progress_style());
            pb.set_message(format!("Worker #{}", tid));
            threads.push(s.spawn(move || bench_fn(tid, pb, opt)));
        }
        for t in threads {
            match t.join().unwrap() {
                Ok(_) => {}
                Err(e) => {
                    is_err = true;
                    log::error!("Worker exit, {}", e);
                }
            }
        }
    });
    let elapsed = start.elapsed();
    if is_err {
        Err(BenchError::BenchFailed().into())
    } else {
        Ok(elapsed)
    }
}

pub fn bench(opt: &BenchOption) -> Result<BenchResult> {
    log::info!("Preparing...");
    prepare(&opt.prefix, &opt.connect_option)?;

    log::info!("Running TPS benchmark");
    let elapsed = do_bench(opt, do_tps_bench)?;
    let tps = opt.iteration as f32 / elapsed.as_secs_f32();

    log::info!("Running QPS benchmark");
    let elapsed = do_bench(opt, do_qps_bench)?;
    let qps = opt.iteration as f32 / elapsed.as_secs_f32();

    Ok(BenchResult { elapsed, tps, qps })
}

fn do_tps_bench(tid: u32, pb: ProgressBar, opt: &BenchOption) -> Result<()> {
    let zk = opt.connect_option.connect()?;
    pb.set_message("Connected");

    let count = opt.iteration / opt.threads;
    for i in tid * count..(tid + 1) * count {
        let path = opt.prefix.clone() + i.to_string().as_str();
        let mode = if opt.ephemeral {
            CreateMode::Ephemeral
        } else {
            CreateMode::Persistent
        };
        zk.create(
            path.as_str(),
            opt.node_value.to_vec(),
            Acl::open_unsafe().clone(),
            mode,
        )?;
        pb.inc(1);
        pb.set_message(format!("Created {}", path))
    }

    pb.finish_with_message(format!("Worker #{} finish", tid));
    Ok(())
}

fn do_qps_bench(tid: u32, pb: ProgressBar, opt: &BenchOption) -> Result<()> {
    let zk = opt.connect_option.connect()?;
    pb.set_message("Connected");

    let count = opt.iteration / opt.threads;
    for i in tid * count..(tid + 1) * count {
        let path = opt.prefix.clone() + i.to_string().as_str();
        zk.get_data(path.as_str(), false)?;
        pb.inc(1);
        pb.set_message(format!("get_data() {}", path))
    }

    pb.finish_with_message(format!("Worker #{} finish", tid));
    Ok(())
}
