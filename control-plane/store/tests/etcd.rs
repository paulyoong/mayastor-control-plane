use composer::{Binary, Builder, ContainerSpec};
use oneshot::Receiver;
use serde::{Deserialize, Serialize};
use std::{
    io,
    net::{SocketAddr, TcpStream},
    str::FromStr,
    time::Duration,
};
use store::{
    etcd::Etcd,
    store::{ObjectKey, StorableObject, Store, WatchEvent},
    types::{v0::pool::PoolSpec, SpecState},
};
use tokio::task::JoinHandle;

static ETCD_ENDPOINT: &str = "0.0.0.0:2379";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestStruct {
    name: String,
    value: u64,
    msg: String,
}

#[tokio::test]
async fn etcd() {
    let _test = Builder::new()
        .name("etcd")
        .add_container_spec(
            ContainerSpec::from_binary(
                "etcd",
                Binary::from_nix("etcd").with_args(vec![
                    "--data-dir",
                    "/tmp/etcd-data",
                    "--advertise-client-urls",
                    "http://0.0.0.0:2379",
                    "--listen-client-urls",
                    "http://0.0.0.0:2379",
                ]),
            )
            .with_portmap("2379", "2379")
            .with_portmap("2380", "2380"),
        )
        .build()
        .await
        .unwrap();

    assert!(wait_for_etcd_ready(ETCD_ENDPOINT).is_ok(), "etcd not ready");

    let mut store = Etcd::new(ETCD_ENDPOINT)
        .await
        .expect("Failed to connect to etcd.");

    let mut pool_spec = PoolSpec {
        node: "test_node".into(),
        id: "9c87b978-4e0b-467f-80f1-ec518f8bb472".into(),
        disks: vec!["test_disk".into()],
        state: SpecState::Creating,
        labels: vec!["test_label".into()],
        updating: false,
    };

    // Add an entry to the store, read it back and make sure it is correct.
    store
        .put_obj(&pool_spec)
        .await
        .expect("Failed to 'put' pool spec in etcd");
    let persisted_pool_spec: PoolSpec = store
        .get_obj(&pool_spec.key())
        .await
        .expect("Failed to 'get' pool spec from etcd");
    assert_eq!(pool_spec, persisted_pool_spec);

    // Start a watcher which should send a message when the subsequent 'put'
    // event occurs.
    let (put_hdl, r) = spawn_watcher(&pool_spec.key(), &mut store).await;

    // Modify entry.
    pool_spec.node = "modified_node".into();
    store
        .put_obj(&pool_spec)
        .await
        .expect("Failed to 'put' pool spec in etcd");

    // Wait up to 1 second for the watcher to see the put event.
    let msg = r
        .recv_timeout(Duration::from_secs(1))
        .expect("Timed out waiting for message");
    let persisted_pool_spec: PoolSpec = match msg {
        WatchEvent::Put(_k, v) => serde_json::from_value(v).expect("Failed to deserialise value"),
        _ => panic!("Expected a 'put' event"),
    };
    assert_eq!(pool_spec, persisted_pool_spec);

    // Start a watcher which should send a message when the subsequent 'delete'
    // event occurs.
    let (del_hdl, r) = spawn_watcher(&pool_spec.key(), &mut store).await;
    store.delete_obj(&pool_spec.key()).await.unwrap();

    // Wait up to 1 second for the watcher to see the delete event.
    let msg = r
        .recv_timeout(Duration::from_secs(1))
        .expect("Timed out waiting for message");
    match msg {
        WatchEvent::Delete => {
            // The entry is deleted. Let's check that a subsequent 'get' fails.
            store
                .get_obj::<PoolSpec>(&pool_spec.key())
                .await
                .expect_err("Entry should have been deleted");
        }
        _ => panic!("Expected a 'delete' event"),
    };

    put_hdl.await.unwrap();
    del_hdl.await.unwrap();
}

/// Spawn a watcher thread which watches for a single change to the entry with
/// the given key.
async fn spawn_watcher<K: ObjectKey, W: Store>(
    key: &K,
    store: &mut W,
) -> (JoinHandle<()>, Receiver<WatchEvent>) {
    let (s, r) = oneshot::channel();
    let mut watcher = store.watch_obj(key).await.expect("Failed to watch");
    let hdl = tokio::spawn(async move {
        match watcher.recv().await.unwrap() {
            Ok(event) => {
                s.send(event).unwrap();
            }
            Err(_) => {
                panic!("Failed to receive event");
            }
        }
    });
    (hdl, r)
}

/// Wait to establish a connection to etcd.
/// Returns 'Ok' if connected otherwise 'Err' is returned.
fn wait_for_etcd_ready(endpoint: &str) -> io::Result<TcpStream> {
    let sa = SocketAddr::from_str(endpoint).unwrap();
    TcpStream::connect_timeout(&sa, Duration::from_secs(3))
}
