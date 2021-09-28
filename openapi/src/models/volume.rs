#![allow(
    clippy::too_many_arguments,
    clippy::new_without_default,
    non_camel_case_types,
    unused_imports
)]
/*
 * Mayastor RESTful API
 *
 * The version of the OpenAPI document: v0
 *
 * Generated by: https://github.com/openebs/openapi-generator
 */

use crate::apis::IntoVec;

/// Volume : Volumes Volume information

/// Volumes Volume information
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Volume {
    #[serde(rename = "spec")]
    pub spec: crate::models::VolumeSpec,
    #[serde(rename = "state")]
    pub state: crate::models::VolumeState,
}

impl Volume {
    /// Volume using only the required fields
    pub fn new(
        spec: impl Into<crate::models::VolumeSpec>,
        state: impl Into<crate::models::VolumeState>,
    ) -> Volume {
        Volume {
            spec: spec.into(),
            state: state.into(),
        }
    }
    /// Volume using all fields
    pub fn new_all(
        spec: impl Into<crate::models::VolumeSpec>,
        state: impl Into<crate::models::VolumeState>,
    ) -> Volume {
        Volume {
            spec: spec.into(),
            state: state.into(),
        }
    }
}
