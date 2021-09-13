use super::*;

use crate::types::v0::store::nexus_child::NexusChild;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt::Debug};
use strum_macros::{EnumString, ToString};

/// Volume Nexuses
///
/// Get all the nexuses with a filter selection
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GetNexuses {
    /// Filter request
    pub filter: Filter,
}

/// Nexus information
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Nexus {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the nexus
    pub uuid: NexusId,
    /// size of the volume in bytes
    pub size: u64,
    /// current status of the nexus
    pub status: NexusStatus,
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
impl Nexus {
    /// Check if the nexus contains the provided `ChildUri`
    pub fn contains_child(&self, uri: &ChildUri) -> bool {
        self.children.iter().any(|c| &c.uri == uri)
    }
}

impl From<Nexus> for models::Nexus {
    fn from(src: Nexus) -> Self {
        models::Nexus::new(
            src.children,
            src.device_uri,
            src.node,
            src.rebuilds,
            src.share,
            src.size,
            src.status,
            apis::Uuid::try_from(src.uuid).unwrap(),
        )
    }
}
impl From<models::Nexus> for Nexus {
    fn from(src: models::Nexus) -> Self {
        Self {
            node: src.node.into(),
            uuid: src.uuid.to_string().into(),
            status: src.state.into(),
            children: src.children.into_iter().map(From::from).collect(),
            device_uri: src.device_uri,
            rebuilds: src.rebuilds,
            size: src.size,
            share: src.share.into(),
        }
    }
}

bus_impl_string_uuid!(NexusId, "UUID of a mayastor nexus");

/// Nexus State information
#[derive(Serialize, Deserialize, Debug, Clone, EnumString, ToString, Eq, PartialEq)]
pub enum NexusStatus {
    /// Default Unknown state
    Unknown = 0,
    /// healthy and working
    Online = 1,
    /// not healthy but is able to serve IO (i.e. rebuild is in progress)
    Degraded = 2,
    /// broken and unable to serve IO
    Faulted = 3,
}
impl Default for NexusStatus {
    fn default() -> Self {
        Self::Unknown
    }
}
impl From<i32> for NexusStatus {
    fn from(src: i32) -> Self {
        match src {
            1 => Self::Online,
            2 => Self::Degraded,
            3 => Self::Faulted,
            _ => Self::Unknown,
        }
    }
}
impl From<NexusStatus> for models::NexusState {
    fn from(src: NexusStatus) -> Self {
        match src {
            NexusStatus::Unknown => Self::Unknown,
            NexusStatus::Online => Self::Online,
            NexusStatus::Degraded => Self::Degraded,
            NexusStatus::Faulted => Self::Faulted,
        }
    }
}
impl From<models::NexusState> for NexusStatus {
    fn from(src: models::NexusState) -> Self {
        match src {
            models::NexusState::Unknown => Self::Unknown,
            models::NexusState::Online => Self::Online,
            models::NexusState::Degraded => Self::Degraded,
            models::NexusState::Faulted => Self::Faulted,
        }
    }
}

/// The protocol used to share the nexus.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, EnumString, ToString, Eq, PartialEq)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub enum NexusShareProtocol {
    /// shared as NVMe-oF TCP
    Nvmf = 1,
    /// shared as iSCSI
    Iscsi = 2,
}

impl std::cmp::PartialEq<Protocol> for NexusShareProtocol {
    fn eq(&self, other: &Protocol) -> bool {
        &Protocol::from(*self) == other
    }
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

impl From<NexusShareProtocol> for Protocol {
    fn from(src: NexusShareProtocol) -> Self {
        match src {
            NexusShareProtocol::Nvmf => Self::Nvmf,
            NexusShareProtocol::Iscsi => Self::Iscsi,
        }
    }
}
impl From<models::NexusShareProtocol> for NexusShareProtocol {
    fn from(src: models::NexusShareProtocol) -> Self {
        match src {
            models::NexusShareProtocol::Nvmf => Self::Nvmf,
            models::NexusShareProtocol::Iscsi => Self::Iscsi,
        }
    }
}
impl TryFrom<Protocol> for NexusShareProtocol {
    type Error = String;

    fn try_from(value: Protocol) -> Result<Self, Self::Error> {
        match value {
            Protocol::None => Err(format!("Invalid protocol: {:?}", value)),
            Protocol::Nvmf => Ok(Self::Nvmf),
            Protocol::Iscsi => Ok(Self::Iscsi),
            Protocol::Nbd => Err(format!("Invalid protocol: {:?}", value)),
        }
    }
}

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
    pub children: Vec<NexusChild>,
    /// Managed by our control plane
    pub managed: bool,
    /// Volume which owns this nexus, if any
    pub owner: Option<VolumeId>,
}

impl CreateNexus {
    /// Create new `Self` from the given parameters
    pub fn new(
        node: &NodeId,
        uuid: &NexusId,
        size: u64,
        children: &[NexusChild],
        managed: bool,
        owner: Option<&VolumeId>,
    ) -> Self {
        Self {
            node: node.clone(),
            uuid: uuid.clone(),
            size,
            children: children.to_owned(),
            managed,
            owner: owner.cloned(),
        }
    }
}

/// Destroy Nexus Request
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DestroyNexus {
    /// id of the mayastor instance
    pub node: NodeId,
    /// uuid of the nexus
    pub uuid: NexusId,
}

impl From<Nexus> for DestroyNexus {
    fn from(nexus: Nexus) -> Self {
        Self {
            node: nexus.node,
            uuid: nexus.uuid,
        }
    }
}

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
