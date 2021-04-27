#![cfg(test)]

use common::v0::GetSpecs;
use mbus_api::{v0::*, *};
use std::time::Duration;
use store::types::v0::nexus::NexusSpec;
use testlib::ClusterBuilder;

#[actix_rt::test]
async fn volume() {
    let cluster = ClusterBuilder::builder()
        .with_rest(false)
        .with_agents(vec!["core"])
        .with_mayastors(2)
        .build()
        .await
        .unwrap();

    let mayastor = cluster.node(0).to_string();
    let mayastor2 = cluster.node(1).to_string();

    let nodes = GetNodes {}.request().await.unwrap();
    tracing::info!("Nodes: {:?}", nodes);

    prepare_pools(&mayastor, &mayastor2).await;
    test_nexus(&mayastor, &mayastor2).await;
    test_volume().await;

    assert!(GetNexuses::default().request().await.unwrap().0.is_empty());
}

async fn prepare_pools(mayastor: &str, mayastor2: &str) {
    CreatePool {
        node: mayastor.into(),
        id: "pooloop".into(),
        disks: vec!["malloc:///disk0?size_mb=100".into()],
    }
    .request()
    .await
    .unwrap();

    CreatePool {
        node: mayastor2.into(),
        id: "pooloop2".into(),
        disks: vec!["malloc:///disk0?size_mb=100".into()],
    }
    .request()
    .await
    .unwrap();

    let pools = GetPools::default().request().await.unwrap();
    tracing::info!("Pools: {:?}", pools);
}

async fn test_nexus(mayastor: &str, mayastor2: &str) {
    let replica = CreateReplica {
        node: mayastor2.into(),
        uuid: "replica".into(),
        pool: "pooloop2".into(),
        size: 12582912, /* actual size will be a multiple of 4MB so just
                         * create it like so */
        thin: true,
        share: Protocol::Nvmf,
        ..Default::default()
    }
    .request()
    .await
    .unwrap();

    let local = "malloc:///local?size_mb=12".into();

    let nexus = CreateNexus {
        node: mayastor.into(),
        uuid: "f086f12c-1728-449e-be32-9415051090d6".into(),
        size: 5242880,
        children: vec![replica.uri.into(), local],
        ..Default::default()
    }
    .request()
    .await
    .unwrap();

    let nexuses = GetNexuses::default().request().await.unwrap().0;
    tracing::info!("Nexuses: {:?}", nexuses);
    assert_eq!(Some(&nexus), nexuses.first());

    ShareNexus {
        node: mayastor.into(),
        uuid: "f086f12c-1728-449e-be32-9415051090d6".into(),
        key: None,
        protocol: NexusShareProtocol::Nvmf,
    }
    .request()
    .await
    .unwrap();

    DestroyNexus {
        node: mayastor.into(),
        uuid: "f086f12c-1728-449e-be32-9415051090d6".into(),
    }
    .request()
    .await
    .unwrap();

    DestroyReplica {
        node: replica.node,
        pool: replica.pool,
        uuid: replica.uuid,
    }
    .request()
    .await
    .unwrap();

    assert!(GetNexuses::default().request().await.unwrap().0.is_empty());
}

async fn test_volume() {
    let volume = CreateVolume {
        uuid: "359b7e1a-b724-443b-98b4-e6d97fabbb40".into(),
        size: 5242880,
        nexuses: 1,
        replicas: 2,
        allowed_nodes: vec![],
        preferred_nodes: vec![],
        preferred_nexus_nodes: vec![],
    };

    let volume = volume.request().await.unwrap();
    let volumes = GetVolumes::default().request().await.unwrap().0;
    tracing::info!("Volumes: {:?}", volumes);

    assert_eq!(Some(&volume), volumes.first());

    DestroyVolume {
        uuid: "359b7e1a-b724-443b-98b4-e6d97fabbb40".into(),
    }
    .request()
    .await
    .unwrap();

    assert!(GetVolumes::default().request().await.unwrap().0.is_empty());
    assert!(GetNexuses::default().request().await.unwrap().0.is_empty());
    assert!(GetReplicas::default().request().await.unwrap().0.is_empty());
}

/// default timeout options for every bus request
fn bus_timeout_opts() -> TimeoutOptions {
    TimeoutOptions::default()
        .with_max_retries(0)
        .with_timeout(Duration::from_millis(250))
}

/// Get the nexus spec
async fn nexus_spec(replica: &Nexus) -> Option<NexusSpec> {
    let specs = GetSpecs {}.request().await.unwrap().nexuses;
    specs.iter().find(|r| r.uuid == replica.uuid).cloned()
}

#[actix_rt::test]
async fn nexus_share_transaction() {
    let cluster = ClusterBuilder::builder()
        .with_rest(false)
        .with_pools(1)
        .with_agents(vec!["core"])
        .with_node_timeouts(Duration::from_millis(250), Duration::from_millis(500))
        .with_bus_timeouts(bus_timeout_opts())
        .build()
        .await
        .unwrap();
    let mayastor = cluster.node(0);

    let nodes = GetNodes {}.request().await.unwrap();
    tracing::info!("Nodes: {:?}", nodes);

    let local = "malloc:///local?size_mb=12".into();
    let nexus = CreateNexus {
        node: mayastor.clone(),
        uuid: "f086f12c-1728-449e-be32-9415051090d6".into(),
        size: 5242880,
        children: vec![local],
        ..Default::default()
    }
    .request()
    .await
    .unwrap();
    let share = ShareNexus::from((&nexus, None, NexusShareProtocol::Nvmf));

    async fn check_share_operation(nexus: &Nexus, protocol: Protocol) {
        // operation in progress
        assert!(nexus_spec(&nexus).await.unwrap().operation.is_some());
        tokio::time::delay_for(std::time::Duration::from_millis(500)).await;
        // operation is completed
        assert!(nexus_spec(&nexus).await.unwrap().operation.is_none());
        assert_eq!(nexus_spec(&nexus).await.unwrap().share, protocol);
    }

    // pause mayastor
    cluster.composer().pause(mayastor.as_str()).await.unwrap();

    share
        .request_ext(bus_timeout_opts())
        .await
        .expect_err("mayastor is down");

    check_share_operation(&nexus, Protocol::Off).await;

    // unpause mayastor
    cluster.composer().thaw(mayastor.as_str()).await.unwrap();

    // now it should be shared successfully
    let uri = share.request().await.unwrap();
    println!("Share uri: {}", uri);

    cluster.composer().pause(mayastor.as_str()).await.unwrap();

    UnshareNexus::from(&nexus)
        .request_ext(bus_timeout_opts())
        .await
        .expect_err("mayastor down");

    check_share_operation(&nexus, Protocol::Nvmf).await;

    cluster.composer().thaw(mayastor.as_str()).await.unwrap();

    UnshareNexus::from(&nexus).request().await.unwrap();

    assert_eq!(nexus_spec(&nexus).await.unwrap().share, Protocol::Off);
}

#[actix_rt::test]
async fn nexus_share_transaction_store() {
    let store_timeout = Duration::from_millis(250);
    let reconcile_period = Duration::from_millis(250);
    let cluster = ClusterBuilder::builder()
        .with_rest(false)
        .with_pools(1)
        .with_agents(vec!["core"])
        .with_node_timeouts(Duration::from_millis(500), Duration::from_millis(500))
        .with_reconcile_period(reconcile_period, reconcile_period)
        .with_store_timeout(store_timeout)
        .with_bus_timeouts(bus_timeout_opts())
        .build()
        .await
        .unwrap();
    let mayastor = cluster.node(0);

    let local = "malloc:///local?size_mb=12".into();
    let nexus = CreateNexus {
        node: mayastor.clone(),
        uuid: "f086f12c-1728-449e-be32-9415051090d6".into(),
        size: 5242880,
        children: vec![local],
        ..Default::default()
    }
    .request()
    .await
    .unwrap();

    let share = ShareNexus::from((&nexus, None, NexusShareProtocol::Nvmf));

    // pause mayastor
    cluster.composer().pause(mayastor.as_str()).await.unwrap();

    share
        .request_ext(bus_timeout_opts())
        .await
        .expect_err("mayastor down");

    // ensure the share will succeed but etcd store will fail
    // by pausing etcd and releasing mayastor
    cluster.composer().pause("etcd").await.unwrap();
    cluster.composer().thaw(mayastor.as_str()).await.unwrap();

    // hopefully we have enough time before the store times outs
    let spec = nexus_spec(&nexus).await.unwrap();
    assert!(spec.operation.unwrap().result.is_none());

    // let the store write time out
    tokio::time::delay_for(store_timeout * 2).await;

    // and now we have a result but the operation is still pending until
    // we can sync the spec
    let spec = nexus_spec(&nexus).await.unwrap();
    assert!(spec.operation.unwrap().result.is_some());

    // thaw etcd allowing the worker thread to sync the "dirty" spec
    cluster.composer().thaw("etcd").await.unwrap();

    // wait for the reconciler to do its thing
    tokio::time::delay_for(reconcile_period * 2).await;

    // and now we're in sync and the pending operation is no more
    let spec = nexus_spec(&nexus).await.unwrap();
    assert!(spec.operation.is_none());
    assert_eq!(spec.share, Protocol::Nvmf);

    share
        .request_ext(bus_timeout_opts())
        .await
        .expect_err("already shared via nvmf");
}

#[actix_rt::test]
async fn nexus_add_child_transaction() {
    let cluster = ClusterBuilder::builder()
        .with_rest(false)
        .with_pools(1)
        .with_agents(vec!["core"])
        .with_node_timeouts(Duration::from_millis(250), Duration::from_millis(500))
        .with_bus_timeouts(bus_timeout_opts())
        .build()
        .await
        .unwrap();
    let mayastor = cluster.node(0);

    let nodes = GetNodes {}.request().await.unwrap();
    tracing::info!("Nodes: {:?}", nodes);

    let child2 = "malloc:///ch2?size_mb=12";
    let nexus = CreateNexus {
        node: mayastor.clone(),
        uuid: "f086f12c-1728-449e-be32-9415051090d6".into(),
        size: 5242880,
        children: vec!["malloc:///ch1?size_mb=12".into()],
        ..Default::default()
    }
    .request()
    .await
    .unwrap();
    let add_child = AddNexusChild {
        node: mayastor.clone(),
        nexus: nexus.uuid.clone(),
        uri: child2.into(),
        auto_rebuild: true,
    };
    let rm_child = RemoveNexusChild {
        node: mayastor.clone(),
        nexus: nexus.uuid.clone(),
        uri: child2.into(),
    };

    async fn check_add_operation(nexus: &Nexus, children: usize) {
        // operation in progress
        assert!(nexus_spec(&nexus).await.unwrap().operation.is_some());
        tokio::time::delay_for(std::time::Duration::from_millis(500)).await;
        // operation is complete
        assert!(nexus_spec(&nexus).await.unwrap().operation.is_none());
        assert_eq!(nexus_spec(&nexus).await.unwrap().children.len(), children);
    }

    // pause mayastor
    cluster.composer().pause(mayastor.as_str()).await.unwrap();

    add_child
        .request_ext(bus_timeout_opts())
        .await
        .expect_err("mayastor is down");

    check_add_operation(&nexus, 1).await;

    // unpause mayastor
    cluster.composer().thaw(mayastor.as_str()).await.unwrap();

    // now it should be shared successfully
    let uri = add_child.request().await.unwrap();
    println!("Share uri: {:?}", uri);

    cluster.composer().pause(mayastor.as_str()).await.unwrap();

    rm_child
        .request_ext(bus_timeout_opts())
        .await
        .expect_err("mayastor down");

    check_add_operation(&nexus, 2).await;

    cluster.composer().thaw(mayastor.as_str()).await.unwrap();

    rm_child.request().await.unwrap();

    assert_eq!(nexus_spec(&nexus).await.unwrap().children.len(), 1);
}

#[actix_rt::test]
async fn nexus_add_child_transaction_store() {
    let store_timeout = Duration::from_millis(250);
    let reconcile_period = Duration::from_millis(250);
    let cluster = ClusterBuilder::builder()
        .with_rest(false)
        .with_pools(1)
        .with_agents(vec!["core"])
        .with_node_timeouts(Duration::from_millis(500), Duration::from_millis(500))
        .with_reconcile_period(reconcile_period, reconcile_period)
        .with_store_timeout(store_timeout)
        .with_bus_timeouts(bus_timeout_opts())
        .build()
        .await
        .unwrap();
    let mayastor = cluster.node(0);

    let child2 = "malloc:///ch2?size_mb=12";
    let nexus = CreateNexus {
        node: mayastor.clone(),
        uuid: "f086f12c-1728-449e-be32-9415051090d6".into(),
        size: 5242880,
        children: vec!["malloc:///ch1?size_mb=12".into()],
        ..Default::default()
    }
    .request()
    .await
    .unwrap();
    let add_child = AddNexusChild {
        node: mayastor.clone(),
        nexus: nexus.uuid.clone(),
        uri: child2.into(),
        auto_rebuild: true,
    };

    // pause mayastor
    cluster.composer().pause(mayastor.as_str()).await.unwrap();

    add_child
        .request_ext(bus_timeout_opts())
        .await
        .expect_err("mayastor down");

    // ensure the share will succeed but etcd store will fail
    // by pausing etcd and releasing mayastor
    cluster.composer().pause("etcd").await.unwrap();
    cluster.composer().thaw(mayastor.as_str()).await.unwrap();

    // hopefully we have enough time before the store times outs
    let spec = nexus_spec(&nexus).await.unwrap();
    assert!(spec.operation.unwrap().result.is_none());

    // let the store write time out
    tokio::time::delay_for(store_timeout * 2).await;

    // and now we have a result but the operation is still pending until
    // we can sync the spec
    let spec = nexus_spec(&nexus).await.unwrap();
    assert!(spec.operation.unwrap().result.is_some());

    // thaw etcd allowing the worker thread to sync the "dirty" spec
    cluster.composer().thaw("etcd").await.unwrap();

    // wait for the reconciler to do its thing
    tokio::time::delay_for(reconcile_period * 2).await;

    // and now we're in sync and the pending operation is no more
    let spec = nexus_spec(&nexus).await.unwrap();
    assert!(spec.operation.is_none());
    assert_eq!(spec.children.len(), 2);

    add_child
        .request_ext(bus_timeout_opts())
        .await
        .expect_err("child already added");
}
