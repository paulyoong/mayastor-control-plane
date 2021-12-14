mod registry;
pub mod service;
pub mod specs;

use std::{convert::TryInto, marker::PhantomData, sync::Arc};

use super::{core::registry::Registry, handler, impl_request_handler};
use async_trait::async_trait;
use common::{errors::SvcError, handler::*, Service};

// Pool Operations
use common_lib::types::v0::message_bus::{CreatePool, GetPools};
// Replica Operations
use common_lib::types::v0::message_bus::{
    CreateReplica, DestroyReplica, GetReplicas, ShareReplica, UnshareReplica,
};

pub(crate) async fn configure(builder: Service) -> Service {
    let registry = builder.get_shared_state::<Registry>().clone();
    let tmp = service::Service::new(registry);
    let pool_service = Arc::new(tmp.clone());
    builder
        .with_channel(ChannelVs::Pool)
        .with_default_liveness()
        .with_shared_state(tmp)
        .with_pool_transport(pool_service)
        .await
        .with_subscription(handler!(GetPools))
        .with_subscription(handler!(CreatePool))
        //.with_subscription(handler!(DestroyPool))
        .with_subscription(handler!(GetReplicas))
        .with_subscription(handler!(CreateReplica))
        .with_subscription(handler!(DestroyReplica))
        .with_subscription(handler!(ShareReplica))
        .with_subscription(handler!(UnshareReplica))
}

/// Pool Agent's Tests
#[cfg(test)]
mod tests;
