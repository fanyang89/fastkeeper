fn main() {
    let client = zookeeper::Client::new(vec!["127.0.0.1:2181".into()]);
}
