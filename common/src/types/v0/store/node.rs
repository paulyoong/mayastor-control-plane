//! Definition of node types that can be saved to the persistent store.

use crate::types::v0::{
    message_bus::{self, NodeId},
    openapi::models,
    store::{
        definitions::{ObjectKey, StorableObject, StorableObjectType},
        ResourceUuid,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type NodeLabels = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Node {
    /// Node information.
    node: message_bus::NodeState,
    /// Node labels.
    labels: NodeLabels,
}

pub struct NodeState {
    /// Node information
    pub node: message_bus::NodeState,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Default, Clone)]
pub struct NodeSpec {
    /// Node identification.
    id: NodeId,
    /// Endpoint of the io-engine instance (gRPC)
    endpoint: String,
    /// Node labels.
    labels: NodeLabels,
    /// Cordon labels.
    cordon_labels: Vec<String>,
}
impl NodeSpec {
    /// Return a new `Self`
    pub fn new(
        id: NodeId,
        endpoint: String,
        labels: NodeLabels,
        cordon_label: Option<Vec<String>>,
    ) -> Self {
        Self {
            id,
            endpoint,
            labels,
            cordon_labels: cordon_label.unwrap_or_default(),
        }
    }
    /// Node identification
    pub fn id(&self) -> &NodeId {
        &self.id
    }
    /// Node gRPC endpoint
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    /// Node labels
    pub fn labels(&self) -> &NodeLabels {
        &self.labels
    }
    /// Node gRPC endpoint
    pub fn set_endpoint(&mut self, endpoint: String) {
        self.endpoint = endpoint
    }
    /// Cordon node by applying the label.
    pub fn cordon(&mut self, label: String) {
        self.cordon_labels.push(label);
    }
    /// Uncordon node by removing the corresponding label.
    pub fn uncordon(&mut self, label: String) {
        if let Ok(index) = self.cordon_labels.binary_search(&label) {
            self.cordon_labels.remove(index);
        }
    }
    /// Returns whether or not the node is cordoned.
    pub fn cordoned(&self) -> bool {
        !self.cordon_labels.is_empty()
    }
    /// Returns the cordon labels
    pub fn cordon_labels(&self) -> Vec<String> {
        self.cordon_labels.clone()
    }
}

impl From<NodeSpec> for models::NodeSpec {
    fn from(src: NodeSpec) -> Self {
        Self::new(src.endpoint, src.id, src.cordon_labels)
    }
}

impl ResourceUuid for NodeSpec {
    type Id = NodeId;
    fn uuid(&self) -> Self::Id {
        self.id.clone()
    }
}

/// Key used by the store to uniquely identify a NodeSpec structure.
pub struct NodeSpecKey(NodeId);

impl From<&NodeId> for NodeSpecKey {
    fn from(id: &NodeId) -> Self {
        Self(id.clone())
    }
}

impl ObjectKey for NodeSpecKey {
    fn key_type(&self) -> StorableObjectType {
        StorableObjectType::NodeSpec
    }

    fn key_uuid(&self) -> String {
        self.0.to_string()
    }
}

impl StorableObject for NodeSpec {
    type Key = NodeSpecKey;

    fn key(&self) -> Self::Key {
        NodeSpecKey(self.id.clone())
    }
}
