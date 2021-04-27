#![allow(clippy::field_reassign_with_default)]
use super::*;
use paperclip::{
    actix::Apiv2Schema,
    v2::{
        models::{DataType, DataTypeFormat},
        schema::TypedData,
    },
};
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::{cmp::Ordering, fmt::Debug};
use strum_macros::{EnumString, ToString};

pub(super) const VERSION: &str = "v0";

/// Versioned Channels
#[derive(Clone, Debug, EnumString, ToString)]
#[strum(serialize_all = "camelCase")]
pub enum ChannelVs {
    /// Default
    Default,
    /// Registration of mayastor instances with the control plane
    Registry,
    /// Node Service which exposes the registered mayastor instances
    Node,
    /// Pool Service which manages mayastor pools and replicas
    Pool,
    /// Volume Service which manages mayastor volumes
    Volume,
    /// Nexus Service which manages mayastor nexuses
    Nexus,
    /// Keep it In Sync Service
    Kiiss,
    /// Json gRPC Agent
    JsonGrpc,
    /// Core Agent combines Node, Pool and Volume services
    Core,
    /// Watcher Agent
    Watcher,
}
impl Default for ChannelVs {
    fn default() -> Self {
        ChannelVs::Default
    }
}

impl From<ChannelVs> for Channel {
    fn from(channel: ChannelVs) -> Self {
        Channel::v0(channel)
    }
}

/// Versioned Message Id's
#[derive(Debug, PartialEq, Clone, ToString, EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum MessageIdVs {
    /// Default
    Default,
    /// Liveness Probe
    Liveness,
    /// Update Config
    ConfigUpdate,
    /// Request current Config
    ConfigGetCurrent,
    /// Register mayastor
    Register,
    /// Deregister mayastor
    Deregister,
    /// Node Service
    /// Get all node information
    GetNodes,
    /// Pool Service
    ///
    /// Get pools with filter
    GetPools,
    /// Create Pool,
    CreatePool,
    /// Destroy Pool,
    DestroyPool,
    /// Get replicas with filter
    GetReplicas,
    /// Create Replica,
    CreateReplica,
    /// Destroy Replica,
    DestroyReplica,
    /// Share Replica,
    ShareReplica,
    /// Unshare Replica,
    UnshareReplica,
    /// Volume Service
    ///
    /// Get nexuses with filter
    GetNexuses,
    /// Create nexus
    CreateNexus,
    /// Destroy Nexus
    DestroyNexus,
    /// Share Nexus
    ShareNexus,
    /// Unshare Nexus
    UnshareNexus,
    /// Remove a child from its parent nexus
    RemoveNexusChild,
    /// Add a child to a nexus
    AddNexusChild,
    /// Get all volumes
    GetVolumes,
    /// Create Volume,
    CreateVolume,
    /// Delete Volume
    DestroyVolume,
    /// Add nexus to volume
    AddVolumeNexus,
    /// Remove nexus from volume
    RemoveVolumeNexus,
    /// Generic JSON gRPC message
    JsonGrpc,
    /// Get block devices
    GetBlockDevices,
    /// Create new Resource Watch
    CreateWatch,
    /// Get watches
    GetWatches,
    /// Delete Resource Watch
    DeleteWatch,
    /// Get Specs
    GetSpecs,
}

// Only V0 should export this macro
// This allows the example code to use the v0 default
// Otherwise they have to impl whatever version they require
#[macro_export]
/// Use version 0 of the Message and Channel
macro_rules! impl_channel_id {
    ($I:ident, $C:ident) => {
        fn id(&self) -> MessageId {
            MessageId::v0(v0::MessageIdVs::$I)
        }
        fn channel(&self) -> Channel {
            Channel::v0(v0::ChannelVs::$C)
        }
    };
}

/// Liveness Probe
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Liveness {}
bus_impl_message_all!(Liveness, Liveness, (), Default);

/// Mayastor configurations
/// Currently, we have the global mayastor config and the child states config
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum Config {
    /// Mayastor global config
    MayastorConfig,
    /// Mayastor child states config
    ChildStatesConfig,
}
impl Default for Config {
    fn default() -> Self {
        Config::MayastorConfig
    }
}

/// Config Messages

/// Update mayastor configuration
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ConfigUpdate {
    /// type of config being updated
    pub kind: Config,
    /// actual config data
    pub data: Vec<u8>,
}
bus_impl_message_all!(ConfigUpdate, ConfigUpdate, (), Kiiss);

/// Request message configuration used by mayastor to request configuration
/// from a control plane service
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ConfigGetCurrent {
    /// type of config requested
    pub kind: Config,
}
/// Reply message configuration returned by a controle plane service to mayastor
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ReplyConfig {
    /// config data
    pub config: Vec<u8>,
}
bus_impl_message_all!(
    ConfigGetCurrent,
    ConfigGetCurrent,
    ReplyConfig,
    Kiiss,
    GetConfig
);

/// Registration

/// Register message payload
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    /// id of the mayastor instance
    pub id: NodeId,
    /// grpc_endpoint of the mayastor instance
    pub grpc_endpoint: String,
}
bus_impl_message_all!(Register, Register, (), Registry);

/// Deregister message payload
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Deregister {
    /// id of the mayastor instance
    pub id: NodeId,
}
bus_impl_message_all!(Deregister, Deregister, (), Registry);

/// Node Service
///
/// Get all the nodes
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GetNodes {}

/// State of the Node
#[derive(
    Serialize, Deserialize, Debug, Clone, EnumString, ToString, Eq, PartialEq, Apiv2Schema,
)]
pub enum NodeState {
    /// Node has unexpectedly disappeared
    Unknown,
    /// Node is deemed online if it has not missed the
    /// registration keep alive deadline
    Online,
    /// Node is deemed offline if has missed the
    /// registration keep alive deadline
    Offline,
}

impl Default for NodeState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Node information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    /// id of the mayastor instance
    pub id: NodeId,
    /// grpc_endpoint of the mayastor instance
    pub grpc_endpoint: String,
    /// deemed state of the node
    pub state: NodeState,
}

bus_impl_vector_request!(Nodes, Node);
bus_impl_message_all!(GetNodes, GetNodes, Nodes, Node);

/// Filter Objects based on one of the following criteria
/// # Example:
/// // Get all nexuses from the node `node_id`
/// let nexuses =
///     MessageBus::get_nexuses(Filter::Node(node_id)).await.unwrap();
#[derive(Serialize, Deserialize, Debug, Clone, strum_macros::ToString)] // likely this ToString does not do the right thing...
pub enum Filter {
    /// All objects
    None,
    /// Filter by Node id
    Node(NodeId),
    /// Pool filters
    ///
    /// Filter by Pool id
    Pool(PoolId),
    /// Filter by Node and Pool id
    NodePool(NodeId, PoolId),
    /// Filter by Node and Replica id
    NodeReplica(NodeId, ReplicaId),
    /// Filter by Node, Pool and Replica id
    NodePoolReplica(NodeId, PoolId, ReplicaId),
    /// Filter by Pool and Replica id
    PoolReplica(PoolId, ReplicaId),
    /// Filter by Replica id
    Replica(ReplicaId),
    /// Volume filters
    ///
    /// Filter by Node and Nexus
    NodeNexus(NodeId, NexusId),
    /// Filter by Nexus
    Nexus(NexusId),
    /// Filter by Node and Volume
    NodeVolume(NodeId, VolumeId),
    /// Filter by Volume
    Volume(VolumeId),
}
impl Default for Filter {
    fn default() -> Self {
        Self::None
    }
}

macro_rules! bus_impl_string_id_inner {
    ($Name:ident, $Doc:literal) => {
        #[doc = $Doc]
        #[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
        pub struct $Name(String);

        impl std::fmt::Display for $Name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl $Name {
            /// Build Self from a string trait id
            pub fn as_str<'a>(&'a self) -> &'a str {
                self.0.as_str()
            }
        }

        impl From<&str> for $Name {
            fn from(id: &str) -> Self {
                $Name::from(id)
            }
        }
        impl From<String> for $Name {
            fn from(id: String) -> Self {
                $Name::from(id.as_str())
            }
        }

        impl From<&$Name> for $Name {
            fn from(id: &$Name) -> $Name {
                id.clone()
            }
        }

        impl From<$Name> for String {
            fn from(id: $Name) -> String {
                id.to_string()
            }
        }
    };
}

macro_rules! bus_impl_string_id {
    ($Name:ident, $Doc:literal) => {
        bus_impl_string_id_inner!($Name, $Doc);
        impl Default for $Name {
            /// Generates new blank identifier
            fn default() -> Self {
                $Name(uuid::Uuid::default().to_string())
            }
        }
        impl $Name {
            /// Build Self from a string trait id
            pub fn from<T: Into<String>>(id: T) -> Self {
                $Name(id.into())
            }
            /// Generates new random identifier
            pub fn new() -> Self {
                $Name(uuid::Uuid::new_v4().to_string())
            }
        }
        impl TypedData for $Name {
            fn data_type() -> DataType {
                DataType::String
            }
            fn format() -> Option<DataTypeFormat> {
                None
            }
        }
    };
}

macro_rules! bus_impl_string_uuid {
    ($Name:ident, $Doc:literal) => {
        bus_impl_string_id_inner!($Name, $Doc);
        impl Default for $Name {
            /// Generates new blank identifier
            fn default() -> Self {
                $Name(uuid::Uuid::default().to_string())
            }
        }
        impl $Name {
            /// Build Self from a string trait id
            pub fn from<T: Into<String>>(id: T) -> Self {
                $Name(id.into())
            }
            /// Generates new random identifier
            pub fn new() -> Self {
                $Name(uuid::Uuid::new_v4().to_string())
            }
        }
        impl TypedData for $Name {
            fn data_type() -> DataType {
                DataType::String
            }
            fn format() -> Option<DataTypeFormat> {
                Some(DataTypeFormat::Uuid)
            }
        }
    };
}

macro_rules! bus_impl_string_id_percent_decoding {
    ($Name:ident, $Doc:literal) => {
        bus_impl_string_id_inner!($Name, $Doc);
        impl Default for $Name {
            fn default() -> Self {
                $Name("".to_string())
            }
        }
        impl $Name {
            /// Build Self from a string trait id
            pub fn from<T: Into<String>>(id: T) -> Self {
                let src: String = id.into();
                let decoded_src = percent_decode_str(src.clone().as_str())
                    .decode_utf8()
                    .unwrap_or(src.into())
                    .to_string();
                $Name(decoded_src)
            }
        }
        impl TypedData for $Name {
            fn data_type() -> DataType {
                DataType::String
            }
            fn format() -> Option<DataTypeFormat> {
                None
            }
        }
    };
}

bus_impl_string_id!(NodeId, "ID of a mayastor node");
bus_impl_string_id!(PoolId, "ID of a mayastor pool");
bus_impl_string_uuid!(ReplicaId, "UUID of a mayastor pool replica");
bus_impl_string_uuid!(NexusId, "UUID of a mayastor nexus");
bus_impl_string_id_percent_decoding!(ChildUri, "URI of a mayastor nexus child");
bus_impl_string_uuid!(VolumeId, "UUID of a mayastor volume");
bus_impl_string_id!(JsonGrpcMethod, "JSON gRPC method");
bus_impl_string_id!(
    JsonGrpcParams,
    "Parameters to be passed to a JSON gRPC method"
);

/// Pool Service
/// Get all the pools from specific node or None for all nodes
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GetPools {
    /// Filter request
    pub filter: Filter,
}

/// State of the Pool
#[derive(
    Serialize, Deserialize, Debug, Clone, EnumString, ToString, Eq, PartialEq, Apiv2Schema,
)]
pub enum PoolState {
    /// unknown state
    Unknown = 0,
    /// the pool is in normal working order
    Online = 1,
    /// the pool has experienced a failure but can still function
    Degraded = 2,
    /// the pool is completely inaccessible
    Faulted = 3,
}

impl Default for PoolState {
    fn default() -> Self {
        Self::Unknown
    }
}
impl From<i32> for PoolState {
    fn from(src: i32) -> Self {
        match src {
            1 => Self::Online,
            2 => Self::Degraded,
            3 => Self::Faulted,
            _ => Self::Unknown,
        }
    }
}

/// Pool information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    /// id of the mayastor instance
    pub node: NodeId,
    /// id of the pool
    pub id: PoolId,
    /// absolute disk paths claimed by the pool
    pub disks: Vec<String>,
    /// current state of the pool
    pub state: PoolState,
    /// size of the pool in bytes
    pub capacity: u64,
    /// used bytes from the pool
    pub used: u64,
}

// online > degraded > unknown/faulted
impl PartialOrd for PoolState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            PoolState::Unknown => match other {
                PoolState::Unknown => None,
                PoolState::Online => Some(Ordering::Less),
                PoolState::Degraded => Some(Ordering::Less),
                PoolState::Faulted => None,
            },
            PoolState::Online => match other {
                PoolState::Unknown => Some(Ordering::Greater),
                PoolState::Online => Some(Ordering::Equal),
                PoolState::Degraded => Some(Ordering::Greater),
                PoolState::Faulted => Some(Ordering::Greater),
            },
            PoolState::Degraded => match other {
                PoolState::Unknown => Some(Ordering::Greater),
                PoolState::Online => Some(Ordering::Less),
                PoolState::Degraded => Some(Ordering::Equal),
                PoolState::Faulted => Some(Ordering::Greater),
            },
            PoolState::Faulted => match other {
                PoolState::Unknown => None,
                PoolState::Online => Some(Ordering::Less),
                PoolState::Degraded => Some(Ordering::Less),
                PoolState::Faulted => Some(Ordering::Equal),
            },
        }
    }
}

/// Create Pool Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePool {
    /// id of the mayastor instance
    pub node: NodeId,
    /// id of the pool
    pub id: PoolId,
    /// disk device paths or URIs to be claimed by the pool
    pub disks: Vec<String>,
}
bus_impl_message_all!(CreatePool, CreatePool, Pool, Pool);

/// Destroy Pool Request
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DestroyPool {
    /// id of the mayastor instance
    pub node: NodeId,
    /// id of the pool
    pub id: PoolId,
}
bus_impl_message_all!(DestroyPool, DestroyPool, (), Pool);

bus_impl_vector_request!(Pools, Pool);
bus_impl_message_all!(GetPools, GetPools, Pools, Pool);

/// Get all the replicas from specific node and pool
/// or None for all nodes or all pools
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GetReplicas {
    /// Filter request
    pub filter: Filter,
}

/// Replica information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct Replica {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the replica
    pub uuid: ReplicaId,
    /// id of the pool
    pub pool: PoolId,
    /// thin provisioning
    pub thin: bool,
    /// size of the replica in bytes
    pub size: u64,
    /// protocol used for exposing the replica
    pub share: Protocol,
    /// uri usable by nexus to access it
    pub uri: String,
    /// state of the replica
    pub state: ReplicaState,
}

bus_impl_vector_request!(Replicas, Replica);
bus_impl_message_all!(GetReplicas, GetReplicas, Replicas, Pool);

impl From<Replica> for DestroyReplica {
    fn from(replica: Replica) -> Self {
        Self {
            node: replica.node,
            pool: replica.pool,
            uuid: replica.uuid,
        }
    }
}

/// Create Replica Request
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateReplica {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the replica
    pub uuid: ReplicaId,
    /// id of the pool
    pub pool: PoolId,
    /// size of the replica in bytes
    pub size: u64,
    /// thin provisioning
    pub thin: bool,
    /// protocol to expose the replica over
    pub share: Protocol,
    /// Managed by our control plane
    pub managed: bool,
    /// Owners of the resource
    pub owners: ReplicaOwners,
}
bus_impl_message_all!(CreateReplica, CreateReplica, Replica, Pool);

/// Replica owners which is a volume or none and a list of nexuses
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct ReplicaOwners {
    volume: Option<v0::VolumeId>,
    nexuses: Vec<NexusId>,
}
impl ReplicaOwners {
    /// Check if this replica is owned by any nexuses or a volume
    pub fn is_owned(&self) -> bool {
        self.volume.is_some() || !self.nexuses.is_empty()
    }
    /// Check if this replica is owned by this volume
    pub fn owned_by(&self, id: &v0::VolumeId) -> bool {
        self.volume.as_ref() == Some(id)
    }
    /// Create new owners from the volume Id
    pub fn new(volume: &VolumeId) -> Self {
        Self {
            volume: Some(volume.clone()),
            nexuses: vec![],
        }
    }
}

/// Destroy Replica Request
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DestroyReplica {
    /// id of the mayastor instance
    pub node: NodeId,
    /// id of the pool
    pub pool: PoolId,
    /// uuid of the replica
    pub uuid: ReplicaId,
}
bus_impl_message_all!(DestroyReplica, DestroyReplica, (), Pool);

/// Share Replica Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareReplica {
    /// id of the mayastor instance
    pub node: NodeId,
    /// id of the pool
    pub pool: PoolId,
    /// uuid of the replica
    pub uuid: ReplicaId,
    /// protocol used for exposing the replica
    pub protocol: ReplicaShareProtocol,
}
bus_impl_message_all!(ShareReplica, ShareReplica, String, Pool);

impl From<ShareReplica> for UnshareReplica {
    fn from(share: ShareReplica) -> Self {
        Self {
            node: share.node,
            pool: share.pool,
            uuid: share.uuid,
        }
    }
}
impl From<&Replica> for ShareReplica {
    fn from(from: &Replica) -> Self {
        Self {
            node: from.node.clone(),
            pool: from.pool.clone(),
            uuid: from.uuid.clone(),
            protocol: ReplicaShareProtocol::Nvmf,
        }
    }
}
impl From<&Replica> for UnshareReplica {
    fn from(from: &Replica) -> Self {
        Self {
            node: from.node.clone(),
            pool: from.pool.clone(),
            uuid: from.uuid.clone(),
        }
    }
}
impl From<UnshareReplica> for ShareReplica {
    fn from(share: UnshareReplica) -> Self {
        Self {
            node: share.node,
            pool: share.pool,
            uuid: share.uuid,
            protocol: ReplicaShareProtocol::Nvmf,
        }
    }
}

/// Unshare Replica Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UnshareReplica {
    /// id of the mayastor instance
    pub node: NodeId,
    /// id of the pool
    pub pool: PoolId,
    /// uuid of the replica
    pub uuid: ReplicaId,
}
bus_impl_message_all!(UnshareReplica, UnshareReplica, (), Pool);

/// Indicates what protocol the bdev is shared as
#[derive(
    Serialize, Deserialize, Debug, Clone, EnumString, ToString, Eq, PartialEq, Apiv2Schema,
)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum Protocol {
    /// not shared by any of the variants
    Off = 0,
    /// shared as NVMe-oF TCP
    Nvmf = 1,
    /// shared as iSCSI
    Iscsi = 2,
    /// shared as NBD
    Nbd = 3,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::Off
    }
}
impl From<i32> for Protocol {
    fn from(src: i32) -> Self {
        match src {
            0 => Self::Off,
            1 => Self::Nvmf,
            2 => Self::Iscsi,
            _ => Self::Off,
        }
    }
}
impl From<ReplicaShareProtocol> for Protocol {
    fn from(src: ReplicaShareProtocol) -> Self {
        match src {
            ReplicaShareProtocol::Nvmf => Self::Nvmf,
        }
    }
}
impl From<NexusShareProtocol> for Protocol {
    fn from(src: NexusShareProtocol) -> Self {
        match src {
            NexusShareProtocol::Nvmf => Self::Nvmf,
            NexusShareProtocol::Iscsi => Self::Iscsi,
        }
    }
}

/// The protocol used to share the nexus.
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, EnumString, ToString, Eq, PartialEq, Apiv2Schema,
)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum NexusShareProtocol {
    /// shared as NVMe-oF TCP
    Nvmf = 1,
    /// shared as iSCSI
    Iscsi = 2,
}

impl Default for NexusShareProtocol {
    fn default() -> Self {
        Self::Nvmf
    }
}
impl From<i32> for NexusShareProtocol {
    fn from(src: i32) -> Self {
        match src {
            1 => Self::Nvmf,
            2 => Self::Iscsi,
            _ => panic!("Invalid nexus share protocol {}", src),
        }
    }
}

/// The protocol used to share the replica.
#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, EnumString, ToString, Eq, PartialEq, Apiv2Schema,
)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum ReplicaShareProtocol {
    /// shared as NVMe-oF TCP
    Nvmf = 1,
}

impl Default for ReplicaShareProtocol {
    fn default() -> Self {
        Self::Nvmf
    }
}
impl From<i32> for ReplicaShareProtocol {
    fn from(src: i32) -> Self {
        match src {
            1 => Self::Nvmf,
            _ => panic!("Invalid replica share protocol {}", src),
        }
    }
}

/// State of the Replica
#[derive(
    Serialize, Deserialize, Debug, Clone, EnumString, ToString, Eq, PartialEq, Apiv2Schema,
)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum ReplicaState {
    /// unknown state
    Unknown = 0,
    /// the replica is in normal working order
    Online = 1,
    /// the replica has experienced a failure but can still function
    Degraded = 2,
    /// the replica is completely inaccessible
    Faulted = 3,
}

impl Default for ReplicaState {
    fn default() -> Self {
        Self::Unknown
    }
}
impl From<i32> for ReplicaState {
    fn from(src: i32) -> Self {
        match src {
            1 => Self::Online,
            2 => Self::Degraded,
            3 => Self::Faulted,
            _ => Self::Unknown,
        }
    }
}

/// Volume Nexuses
///
/// Get all the nexuses with a filter selection
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GetNexuses {
    /// Filter request
    pub filter: Filter,
}

/// Nexus information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct Nexus {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the nexus
    pub uuid: NexusId,
    /// size of the volume in bytes
    pub size: u64,
    /// current state of the nexus
    pub state: NexusState,
    /// array of children
    pub children: Vec<Child>,
    /// URI of the device for the volume (missing if not published).
    /// Missing property and empty string are treated the same.
    pub device_uri: String,
    /// total number of rebuild tasks
    pub rebuilds: u32,
    /// protocol used for exposing the nexus
    pub share: Protocol,
}

/// Child information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct Child {
    /// uri of the child device
    pub uri: ChildUri,
    /// state of the child
    pub state: ChildState,
    /// current rebuild progress (%)
    pub rebuild_progress: Option<i32>,
}

/// Child State information
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
pub enum ChildState {
    /// Default Unknown state
    Unknown = 0,
    /// healthy and contains the latest bits
    Online = 1,
    /// rebuild is in progress (or other recoverable error)
    Degraded = 2,
    /// unrecoverable error (control plane must act)
    Faulted = 3,
}
impl Default for ChildState {
    fn default() -> Self {
        Self::Unknown
    }
}
impl From<i32> for ChildState {
    fn from(src: i32) -> Self {
        match src {
            1 => Self::Online,
            2 => Self::Degraded,
            3 => Self::Faulted,
            _ => Self::Unknown,
        }
    }
}

/// Nexus State information
#[derive(
    Serialize, Deserialize, Debug, Clone, EnumString, ToString, Eq, PartialEq, Apiv2Schema,
)]
pub enum NexusState {
    /// Default Unknown state
    Unknown = 0,
    /// healthy and working
    Online = 1,
    /// not healthy but is able to serve IO (i.e. rebuild is in progress)
    Degraded = 2,
    /// broken and unable to serve IO
    Faulted = 3,
}
impl Default for NexusState {
    fn default() -> Self {
        Self::Unknown
    }
}
impl From<i32> for NexusState {
    fn from(src: i32) -> Self {
        match src {
            1 => Self::Online,
            2 => Self::Degraded,
            3 => Self::Faulted,
            _ => Self::Unknown,
        }
    }
}

bus_impl_vector_request!(Nexuses, Nexus);
bus_impl_message_all!(GetNexuses, GetNexuses, Nexuses, Nexus);

/// Create Nexus Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateNexus {
    /// id of the mayastor instance
    pub node: NodeId,
    /// the nexus uuid will be set to this
    pub uuid: NexusId,
    /// size of the device in bytes
    pub size: u64,
    /// replica can be iscsi and nvmf remote targets or a local spdk bdev
    /// (i.e. bdev:///name-of-the-bdev).
    ///
    /// uris to the targets we connect to
    pub children: Vec<ChildUri>,
    /// Managed by our control plane
    pub managed: bool,
    /// Volume which owns this nexus, if any
    pub owner: Option<v0::VolumeId>,
}
bus_impl_message_all!(CreateNexus, CreateNexus, Nexus, Nexus);

/// Destroy Nexus Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DestroyNexus {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the nexus
    pub uuid: NexusId,
}
bus_impl_message_all!(DestroyNexus, DestroyNexus, (), Nexus);

/// Share Nexus Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShareNexus {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the nexus
    pub uuid: NexusId,
    /// encryption key
    pub key: Option<String>,
    /// share protocol
    pub protocol: NexusShareProtocol,
}
bus_impl_message_all!(ShareNexus, ShareNexus, String, Nexus);

impl From<(&Nexus, Option<String>, NexusShareProtocol)> for ShareNexus {
    fn from((nexus, key, protocol): (&Nexus, Option<String>, NexusShareProtocol)) -> Self {
        Self {
            node: nexus.node.clone(),
            uuid: nexus.uuid.clone(),
            key,
            protocol,
        }
    }
}
impl From<&Nexus> for UnshareNexus {
    fn from(from: &Nexus) -> Self {
        Self {
            node: from.node.clone(),
            uuid: from.uuid.clone(),
        }
    }
}
impl From<ShareNexus> for UnshareNexus {
    fn from(share: ShareNexus) -> Self {
        Self {
            node: share.node,
            uuid: share.uuid,
        }
    }
}

/// Unshare Nexus Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UnshareNexus {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the nexus
    pub uuid: NexusId,
}
bus_impl_message_all!(UnshareNexus, UnshareNexus, (), Nexus);

/// Remove Child from Nexus Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RemoveNexusChild {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the nexus
    pub nexus: NexusId,
    /// URI of the child device to be removed
    pub uri: ChildUri,
}
bus_impl_message_all!(RemoveNexusChild, RemoveNexusChild, (), Nexus);

impl From<AddNexusChild> for RemoveNexusChild {
    fn from(add: AddNexusChild) -> Self {
        Self {
            node: add.node,
            nexus: add.nexus,
            uri: add.uri,
        }
    }
}

/// Add child to Nexus Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AddNexusChild {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the nexus
    pub nexus: NexusId,
    /// URI of the child device to be added
    pub uri: ChildUri,
    /// auto start rebuilding
    pub auto_rebuild: bool,
}
bus_impl_message_all!(AddNexusChild, AddNexusChild, Child, Nexus);

/// Volumes
///
/// Volume information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    /// name of the volume
    pub uuid: VolumeId,
    /// size of the volume in bytes
    pub size: u64,
    /// current state of the volume
    pub state: VolumeState,
    /// array of children nexuses
    pub children: Vec<Nexus>,
}

/// Volume State information
/// Currently it's the same as the nexus
pub type VolumeState = NexusState;

/// Get volumes
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetVolumes {
    /// filter volumes
    pub filter: Filter,
}
bus_impl_vector_request!(Volumes, Volume);
bus_impl_message_all!(GetVolumes, GetVolumes, Volumes, Volume);

/// Create volume
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateVolume {
    /// uuid of the volume
    pub uuid: VolumeId,
    /// size of the volume in bytes
    pub size: u64,
    /// number of children nexuses (ANA)
    pub nexuses: u64,
    /// number of replicas per nexus
    pub replicas: u64,
    /// only these nodes can be used for the replicas
    #[serde(default)]
    pub allowed_nodes: Vec<NodeId>,
    /// preferred nodes for the replicas
    #[serde(default)]
    pub preferred_nodes: Vec<NodeId>,
    /// preferred nodes for the nexuses
    #[serde(default)]
    pub preferred_nexus_nodes: Vec<NodeId>,
}
bus_impl_message_all!(CreateVolume, CreateVolume, Volume, Volume);

/// Delete volume
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DestroyVolume {
    /// uuid of the volume
    pub uuid: VolumeId,
}
bus_impl_message_all!(DestroyVolume, DestroyVolume, (), Volume);

/// Add ANA Nexus to volume
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddVolumeNexus {
    /// uuid of the volume
    pub uuid: VolumeId,
    /// preferred node id for the nexus
    pub preferred_node: Option<NodeId>,
}
bus_impl_message_all!(AddVolumeNexus, AddVolumeNexus, Nexus, Volume);

/// Add ANA Nexus to volume
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoveVolumeNexus {
    /// uuid of the volume
    pub uuid: VolumeId,
    /// id of the node where the nexus lives
    pub node: Option<NodeId>,
}
bus_impl_message_all!(RemoveVolumeNexus, RemoveVolumeNexus, (), Volume);

/// Generic JSON gRPC request
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsonGrpcRequest {
    /// id of the mayastor instance
    pub node: NodeId,
    /// JSON gRPC method to call
    pub method: JsonGrpcMethod,
    /// parameters to be passed to the above method
    pub params: JsonGrpcParams,
}

bus_impl_message_all!(JsonGrpcRequest, JsonGrpc, Value, JsonGrpc);

/// Partition information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
pub struct Partition {
    /// devname of parent device to which this partition belongs
    pub parent: String,
    /// partition number
    pub number: u32,
    /// partition name
    pub name: String,
    /// partition scheme: gpt, dos, ...
    pub scheme: String,
    /// partition type identifier
    pub typeid: String,
    /// UUID identifying partition
    pub uuid: String,
}

/// Filesystem information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
pub struct Filesystem {
    /// filesystem type: ext3, ntfs, ...
    pub fstype: String,
    /// volume label
    pub label: String,
    /// UUID identifying the volume (filesystem)
    pub uuid: String,
    /// path where filesystem is currently mounted
    pub mountpoint: String,
}

/// Block device information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq, Apiv2Schema)]
#[serde(rename_all = "camelCase")]
pub struct BlockDevice {
    /// entry in /dev associated with device
    pub devname: String,
    /// currently "disk" or "partition"
    pub devtype: String,
    /// major device number
    pub devmajor: u32,
    /// minor device number
    pub devminor: u32,
    /// device model - useful for identifying mayastor devices
    pub model: String,
    /// official device path
    pub devpath: String,
    /// list of udev generated symlinks by which device may be identified
    pub devlinks: Vec<String>,
    /// size of device in (512 byte) blocks
    pub size: u64,
    /// partition information in case where device represents a partition
    pub partition: Partition,
    /// filesystem information in case where a filesystem is present
    pub filesystem: Filesystem,
    /// identifies if device is available for use (ie. is not "currently" in
    /// use)
    pub available: bool,
}
/// Get block devices
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetBlockDevices {
    /// id of the mayastor instance
    pub node: NodeId,
    /// specifies whether to get all devices or only usable devices
    pub all: bool,
}
bus_impl_vector_request!(BlockDevices, BlockDevice);
bus_impl_message_all!(GetBlockDevices, GetBlockDevices, BlockDevices, Node);

///
/// Watcher Agent

/// Create new Resource Watch
/// Uniquely identifiable by resource_id and callback
pub type CreateWatch = Watch;
bus_impl_message_all!(CreateWatch, CreateWatch, (), Watcher);

/// Watch Resource in the store
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Watch {
    /// id of the resource to watch on
    pub id: WatchResourceId,
    /// callback used to notify the watcher of a change
    pub callback: WatchCallback,
    /// type of watch
    pub watch_type: WatchType,
}

bus_impl_vector_request!(Watches, Watch);

/// Get Resource Watches
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetWatchers {
    /// id of the resource to get
    pub resource: WatchResourceId,
}

bus_impl_message_all!(GetWatchers, GetWatches, Watches, Watcher);

/// Uniquely Identify a Resource
pub type Resource = WatchResourceId;

/// The different resource types that can be watched
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum WatchResourceId {
    /// nodes
    Node(NodeId),
    /// pools
    Pool(PoolId),
    /// replicas
    Replica(ReplicaId),
    /// replica state
    ReplicaState(ReplicaId),
    /// replica spec
    ReplicaSpec(ReplicaId),
    /// nexuses
    Nexus(NexusId),
    /// volumes
    Volume(VolumeId),
}
impl Default for WatchResourceId {
    fn default() -> Self {
        Self::Node(Default::default())
    }
}
impl ToString for WatchResourceId {
    fn to_string(&self) -> String {
        match self {
            WatchResourceId::Node(id) => format!("node/{}", id.to_string()),
            WatchResourceId::Pool(id) => format!("pool/{}", id.to_string()),
            WatchResourceId::Replica(id) => {
                format!("replica/{}", id.to_string())
            }
            WatchResourceId::ReplicaState(id) => {
                format!("replica_state/{}", id.to_string())
            }
            WatchResourceId::ReplicaSpec(id) => {
                format!("replica_spec/{}", id.to_string())
            }
            WatchResourceId::Nexus(id) => format!("nexus/{}", id.to_string()),
            WatchResourceId::Volume(id) => format!("volume/{}", id.to_string()),
        }
    }
}

/// The difference types of watches
#[derive(Serialize, Deserialize, Debug, Clone, Apiv2Schema, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum WatchType {
    /// Watch for changes on the desired state
    Desired,
    /// Watch for changes on the actual state
    Actual,
    /// Watch for both `Desired` and `Actual` changes
    All,
}
impl Default for WatchType {
    fn default() -> Self {
        Self::All
    }
}

/// Delete Watch which was previously created by CreateWatcher
/// Fields should match the ones used for the creation
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWatch {
    /// id of the resource to delete the watch from
    pub id: WatchResourceId,
    /// callback to be deleted
    pub callback: WatchCallback,
    /// type of watch to be deleted
    pub watch_type: WatchType,
}
bus_impl_message_all!(DeleteWatch, DeleteWatch, (), Watcher);

/// Watcher Callback types
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum WatchCallback {
    /// HTTP URI callback
    Uri(String),
}
impl Default for WatchCallback {
    fn default() -> Self {
        Self::Uri(Default::default())
    }
}
