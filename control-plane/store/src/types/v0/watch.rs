use crate::store::{ObjectKey, StorableObjectType};
use mbus_api::v0;

impl ObjectKey for v0::WatchResourceId {
    fn key_type(&self) -> StorableObjectType {
        match &self {
            v0::WatchResourceId::NodeSpec(_) => StorableObjectType::Node,
            v0::WatchResourceId::PoolSpec(_) => StorableObjectType::Pool,
            v0::WatchResourceId::ReplicaState(_) => StorableObjectType::ReplicaState,
            v0::WatchResourceId::ReplicaSpec(_) => StorableObjectType::ReplicaSpec,
            v0::WatchResourceId::NexusState(_) => StorableObjectType::NexusState,
            v0::WatchResourceId::NexusSpec(_) => StorableObjectType::NexusSpec,
            v0::WatchResourceId::VolumeState(_) => StorableObjectType::VolumeState,
            v0::WatchResourceId::VolumeSpec(_) => StorableObjectType::VolumeSpec,
            v0::WatchResourceId::ChildState(_) => StorableObjectType::ChildState,
            v0::WatchResourceId::ChildSpec(_) => StorableObjectType::ChildSpec,
        }
    }
    fn key_uuid(&self) -> String {
        match &self {
            v0::WatchResourceId::NodeSpec(i) => i.to_string(),
            v0::WatchResourceId::PoolSpec(i) => i.to_string(),
            v0::WatchResourceId::ReplicaState(i) => i.to_string(),
            v0::WatchResourceId::ReplicaSpec(i) => i.to_string(),
            v0::WatchResourceId::NexusState(i) => i.to_string(),
            v0::WatchResourceId::NexusSpec(i) => i.to_string(),
            v0::WatchResourceId::VolumeState(i) => i.to_string(),
            v0::WatchResourceId::VolumeSpec(i) => i.to_string(),
            v0::WatchResourceId::ChildState(i) => i.to_string(),
            v0::WatchResourceId::ChildSpec(i) => i.to_string(),
        }
    }
}
