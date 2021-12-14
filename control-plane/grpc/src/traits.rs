use std::collections::HashMap;

/// Trait implemented by services which support pool operations.
/// This trait can only be implemented on types which support the PoolInfo trait.
#[tonic::async_trait]
pub trait PoolOperations {
    //async fn create(&self, pool: &(dyn PoolInfo + Sync + Send)) -> Result<(), ()>;
    async fn destroy(&self, pool_id: String);
}

/// Trait to extract pool information.
pub trait PoolInfo {
    fn pool_id(&self) -> String;
    fn node_id(&self) -> String;
    fn disks(&self) -> Option<Vec<String>>;
    fn labels(&self) -> Option<HashMap<String, String>>;
}
