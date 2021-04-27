use composer::*;
use deployer_lib::{
    infra::{Components, Error, Mayastor},
    *,
};
use opentelemetry::{
    global,
    sdk::{propagation::TraceContextPropagator, trace::Tracer},
};

use opentelemetry_jaeger::Uninstall;
pub use rest_client::{
    versions::v0::{self, Message, RestClient},
    ActixRestClient,
    ClientError,
};
use std::time::Duration;

#[actix_rt::test]
#[ignore]
async fn smoke_test() {
    // make sure the cluster can bootstrap properly
    let _cluster = ClusterBuilder::builder()
        .build()
        .await
        .expect("Should bootstrap the cluster!");
}

/// Default options to create a cluster
pub fn default_options() -> StartOptions {
    StartOptions::default()
        .with_agents(default_agents().split(',').collect())
        .with_jaeger(true)
        .with_mayastors(1)
        .with_show_info(true)
        .with_cluster_name("rest_cluster")
}

/// Cluster with the composer, the rest client and the jaeger pipeline#
#[allow(unused)]
pub struct Cluster {
    composer: ComposeTest,
    rest_client: ActixRestClient,
    jaeger: (Tracer, Uninstall),
    builder: ClusterBuilder,
}

impl Cluster {
    /// compose utility
    pub fn composer(&self) -> &ComposeTest {
        &self.composer
    }

    /// node id for `index`
    pub fn node(&self, index: u32) -> v0::NodeId {
        Mayastor::name(index, &self.builder.opts).into()
    }

    /// node ip for `index`
    pub fn node_ip(&self, index: u32) -> String {
        let name = self.node(index);
        self.composer.container_ip(name.as_str())
    }

    /// pool id for `pool` index on `node` index
    pub fn pool(&self, node: u32, pool: u32) -> v0::PoolId {
        format!("{}-pool-{}", self.node(node), pool + 1).into()
    }

    /// replica id with index for `pool` index and `replica` index
    pub fn replica(node: u32, pool: usize, replica: u32) -> v0::ReplicaId {
        let mut uuid = v0::ReplicaId::default().to_string();
        let _ = uuid.drain(24 .. uuid.len());
        format!(
            "{}{:01x}{:01x}{:08x}",
            uuid, node as u8, pool as u8, replica
        )
        .into()
    }

    /// rest client v0
    pub fn rest_v0(&self) -> impl RestClient {
        self.rest_client.v0()
    }

    /// New cluster
    async fn new(
        trace_rest: bool,
        timeout_rest: std::time::Duration,
        bus_timeout: TimeoutOptions,
        bearer_token: Option<String>,
        components: Components,
        composer: ComposeTest,
        jaeger: (Tracer, Uninstall),
    ) -> Result<Cluster, Error> {
        let rest_client = ActixRestClient::new_timeout(
            "https://localhost:8080",
            trace_rest,
            bearer_token,
            timeout_rest,
        )
        .unwrap();

        components
            .start_wait(&composer, std::time::Duration::from_secs(10))
            .await?;

        // the deployer uses a "fake" message bus so now it's time to
        // connect to the "real" message bus
        composer.connect_to_bus_timeout("nats", bus_timeout).await;

        let cluster = Cluster {
            composer,
            rest_client,
            jaeger,
            builder: ClusterBuilder::builder(),
        };

        Ok(cluster)
    }
}

fn option_str<F: ToString>(input: Option<F>) -> String {
    match input {
        Some(input) => input.to_string(),
        None => "?".into(),
    }
}

/// Run future and compare result with what's expected
/// Expected result should be in the form Result<TestValue,TestValue>
/// where TestValue is a useful value which will be added to the returned error
/// string Eg, testing the replica share protocol:
/// test_result(Ok(Nvmf), async move { ... })
/// test_result(Err(NBD), async move { ... })
pub async fn test_result<F, O, E, T>(
    expected: &Result<O, E>,
    future: F,
) -> Result<(), anyhow::Error>
where
    F: std::future::Future<Output = Result<T, rest_client::ClientError>>,
    E: std::fmt::Debug,
    O: std::fmt::Debug,
{
    match future.await {
        Ok(_) if expected.is_ok() => Ok(()),
        Err(error) if expected.is_err() => match error {
            ClientError::RestServer {
                ..
            } => Ok(()),
            _ => {
                // not the error we were waiting for
                Err(anyhow::anyhow!("Invalid rest response: {}", error))
            }
        },
        Err(error) => Err(anyhow::anyhow!(
            "Expected '{:#?}' but failed with '{}'!",
            expected,
            error
        )),
        Ok(_) => Err(anyhow::anyhow!("Expected '{:#?}' but succeeded!", expected)),
    }
}

#[macro_export]
macro_rules! result_either {
    ($test:expr) => {
        match $test {
            Ok(v) => v,
            Err(v) => v,
        }
    };
}

#[derive(Clone)]
enum PoolDisk {
    Malloc(u64),
    Uri(String),
}

/// Builder for the Cluster
pub struct ClusterBuilder {
    opts: StartOptions,
    pools: Vec<PoolDisk>,
    replicas: Replica,
    trace: bool,
    bearer_token: Option<String>,
    rest_timeout: std::time::Duration,
    bus_timeout: TimeoutOptions,
}

#[derive(Default)]
struct Replica {
    count: u32,
    size: u64,
    share: v0::Protocol,
}

/// default timeout options for every bus request
fn bus_timeout_opts() -> TimeoutOptions {
    TimeoutOptions::default()
        .with_timeout(Duration::from_millis(500))
        .with_timeout_backoff(Duration::from_millis(500))
        .with_max_retries(2)
}

impl ClusterBuilder {
    /// Cluster Builder with default options
    pub fn builder() -> Self {
        ClusterBuilder {
            opts: default_options(),
            pools: vec![],
            replicas: Default::default(),
            trace: true,
            bearer_token: None,
            rest_timeout: std::time::Duration::from_secs(3),
            bus_timeout: bus_timeout_opts(),
        }
    }
    /// Update the start options
    pub fn with_options<F>(mut self, set: F) -> Self
    where
        F: Fn(StartOptions) -> StartOptions,
    {
        self.opts = set(self.opts);
        self
    }
    /// Enable/Disable jaeger tracing
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.trace = enabled;
        self
    }
    /// Rest request timeout
    pub fn with_rest_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.rest_timeout = timeout;
        self
    }
    /// Add `count` malloc pools (100MiB size) to each node
    pub fn with_pools(mut self, count: u32) -> Self {
        for _ in 0 .. count {
            self.pools.push(PoolDisk::Malloc(100 * 1024 * 1024));
        }
        self
    }
    /// Add pool with `disk` to each node
    pub fn with_pool(mut self, disk: &str) -> Self {
        self.pools.push(PoolDisk::Uri(disk.to_string()));
        self
    }
    /// Specify `count` replicas to add to each node per pool
    pub fn with_replicas(mut self, count: u32, size: u64, share: v0::Protocol) -> Self {
        self.replicas = Replica {
            count,
            size,
            share,
        };
        self
    }
    /// Specify `count` mayastors for the cluster
    pub fn with_mayastors(mut self, count: u32) -> Self {
        self.opts = self.opts.with_mayastors(count);
        self
    }
    /// Specify which agents to use
    pub fn with_agents(mut self, agents: Vec<&str>) -> Self {
        self.opts = self.opts.with_agents(agents);
        self
    }
    /// Specify the node deadline for the node agent
    /// eg: 2s
    pub fn with_node_deadline(mut self, deadline: &str) -> Self {
        self.opts = self.opts.with_node_deadline(deadline);
        self
    }
    /// With reconcile period
    pub fn with_reconcile_period(mut self, work: Duration, idle: Duration) -> Self {
        self.opts = self.opts.with_reconcile_period(work, idle);
        self
    }
    /// With store operation timeout
    pub fn with_store_timeout(mut self, timeout: Duration) -> Self {
        self.opts = self.opts.with_store_timeout(timeout);
        self
    }
    /// Specify the node connect and request timeouts
    pub fn with_node_timeouts(mut self, connect: Duration, request: Duration) -> Self {
        self.opts = self.opts.with_node_timeouts(connect, request);
        self
    }
    /// Specify the message bus timeout options
    pub fn with_bus_timeouts(mut self, timeout: TimeoutOptions) -> Self {
        self.bus_timeout = timeout;
        self
    }
    /// Specify whether rest is enabled or not
    pub fn with_rest(mut self, enabled: bool) -> Self {
        self.opts = self.opts.with_rest(enabled);
        self
    }
    /// Build into the resulting Cluster using a composer closure, eg:
    /// .compose_build(|c| c.with_logs(false))
    pub async fn compose_build<F>(self, set: F) -> Result<Cluster, Error>
    where
        F: Fn(Builder) -> Builder,
    {
        let (components, composer) = self.build_prepare()?;
        let composer = set(composer);
        let mut cluster = self.new_cluster(components, composer).await?;
        cluster.builder = self;
        Ok(cluster)
    }
    /// Build into the resulting Cluster
    pub async fn build(self) -> Result<Cluster, Error> {
        let (components, composer) = self.build_prepare()?;
        let mut cluster = self.new_cluster(components, composer).await?;
        cluster.builder = self;
        Ok(cluster)
    }
    fn build_prepare(&self) -> Result<(Components, Builder), Error> {
        let components = Components::new(self.opts.clone());
        let composer = Builder::new()
            .name(&self.opts.cluster_name)
            .configure(components.clone())?
            .with_base_image(self.opts.base_image.clone())
            .autorun(false)
            .with_default_tracing()
            .with_clean(true)
            // test script will clean up containers if ran on CI/CD
            .with_clean_on_panic(false)
            .with_logs(true);
        Ok((components, composer))
    }
    async fn new_cluster(
        &self,
        components: Components,
        compose_builder: Builder,
    ) -> Result<Cluster, Error> {
        global::set_text_map_propagator(TraceContextPropagator::new());
        let jaeger = opentelemetry_jaeger::new_pipeline()
            .with_service_name("tests-client")
            .install()
            .unwrap();

        let composer = compose_builder.build().await?;

        let cluster = Cluster::new(
            self.trace,
            self.rest_timeout,
            self.bus_timeout.clone(),
            self.bearer_token.clone(),
            components,
            composer,
            jaeger,
        )
        .await?;

        if self.opts.show_info {
            for container in cluster.composer.list_cluster_containers().await? {
                let networks = container.network_settings.unwrap().networks.unwrap();
                let ip = networks
                    .get(&self.opts.cluster_name)
                    .unwrap()
                    .ip_address
                    .clone();
                tracing::debug!(
                    "{:?} [{}] {}",
                    container.names.clone().unwrap_or_default(),
                    ip.clone().unwrap_or_default(),
                    option_str(container.command.clone())
                );
            }
        }

        for pool in &self.pools() {
            v0::CreatePool {
                node: pool.node.clone().into(),
                id: pool.id(),
                disks: vec![pool.disk()],
            }
            .request()
            .await
            .unwrap();

            for replica in &pool.replicas {
                replica.request().await.unwrap();
            }
        }

        Ok(cluster)
    }
    fn pools(&self) -> Vec<Pool> {
        let mut pools = vec![];

        for node in 0 .. self.opts.mayastors {
            for pool_index in 0 .. self.pools.len() {
                let pool = &self.pools[pool_index];
                let mut pool = Pool {
                    node: Mayastor::name(node, &self.opts),
                    disk: pool.clone(),
                    index: (pool_index + 1) as u32,
                    replicas: vec![],
                };
                for replica_index in 0 .. self.replicas.count {
                    pool.replicas.push(v0::CreateReplica {
                        node: pool.node.clone().into(),
                        uuid: Cluster::replica(node, pool_index, replica_index),
                        pool: pool.id(),
                        size: self.replicas.size,
                        thin: false,
                        share: self.replicas.share.clone(),
                        managed: false,
                        owners: Default::default(),
                    });
                }
                pools.push(pool);
            }
        }
        pools
    }
}

struct Pool {
    node: String,
    disk: PoolDisk,
    index: u32,
    replicas: Vec<v0::CreateReplica>,
}

impl Pool {
    fn id(&self) -> v0::PoolId {
        format!("{}-pool-{}", self.node, self.index).into()
    }
    fn disk(&self) -> String {
        match &self.disk {
            PoolDisk::Malloc(size) => {
                let size = size / (1024 * 1024);
                format!("malloc:///disk{}?size_mb={}", self.index, size)
            }
            PoolDisk::Uri(uri) => uri.clone(),
        }
    }
}
