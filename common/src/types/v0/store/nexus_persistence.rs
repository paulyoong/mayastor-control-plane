use crate::types::v0::{
    message_bus::{NexusId, ReplicaId, VolumeId},
    store::definitions::{ObjectKey, StorableObject, StorableObjectType},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Definition of the nexus information that gets saved in the persistent
/// store.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct NexusInfo {
    #[serde(skip)]
    /// uuid of the Nexus
    pub uuid: NexusId,
    #[serde(skip)]
    /// uuid of the Volume
    pub volume_uuid: Option<VolumeId>,
    /// Nexus destroyed successfully.
    pub clean_shutdown: bool,
    /// Information about children.
    pub children: Vec<ChildInfo>,
}

impl NexusInfo {
    /// Check if the provided replica is healthy or not
    pub fn is_replica_healthy(&self, replica: &ReplicaId) -> bool {
        match self.children.iter().find(|c| c.uuid == replica.as_str()) {
            Some(info) => info.healthy,
            None => false,
        }
    }
}

/// Definition of the child information that gets saved in the persistent
/// store.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ChildInfo {
    /// UUID of the child.
    pub uuid: String,
    /// Child's state of health.
    pub healthy: bool,
}

/// Key used by the store to uniquely identify a NexusInfo structure.
/// The volume is optional because a nexus can be created which is not associated with a volume.
pub struct NexusInfoKey {
    volume_id: Option<VolumeId>,
    nexus_id: NexusId,
}

impl From<&NexusId> for NexusInfoKey {
    fn from(nexus_id: &NexusId) -> Self {
        Self {
            volume_id: None,
            nexus_id: nexus_id.clone(),
        }
    }
}

impl From<(&VolumeId, &NexusId)> for NexusInfoKey {
    fn from((volume_id, nexus_id): (&VolumeId, &NexusId)) -> Self {
        Self {
            volume_id: Some(volume_id.clone()),
            nexus_id: nexus_id.clone(),
        }
    }
}

impl ObjectKey for NexusInfoKey {
    fn key(&self) -> String {
        let namespace = std::env::var("MY_POD_NAMESPACE").unwrap_or_else(|_| "default".into());
        let nexus_uuid = self.nexus_id.clone();
        match &self.volume_id {
            Some(volume_uuid) => {
                format!(
                    "/namespace/{}/volume/{}/nexus/{}/info",
                    namespace, volume_uuid, nexus_uuid
                )
            }
            None => {
                format!("/namespace/{}/nexus/{}/info", namespace, nexus_uuid)
            }
        }
    }

    fn key_type(&self) -> StorableObjectType {
        // The key is generated directly from the `key()` function above.
        unreachable!()
    }

    fn key_uuid(&self) -> String {
        // The key is generated directly from the `key()` function above.
        unreachable!()
    }
}

impl StorableObject for NexusInfo {
    type Key = NexusInfoKey;

    fn key(&self) -> Self::Key {
        NexusInfoKey {
            volume_id: self.volume_uuid.clone(),
            nexus_id: self.uuid.clone(),
        }
    }
}
