use crate::{
    grpc::{
        core_grpc_client::CoreGrpcClient,
        core_grpc_server::{CoreGrpc, CoreGrpcServer},
        CreatePoolRequest, DestroyPoolRequest, PoolLabel,
    },
    traits::{PoolInfo, PoolOperations},
};

use std::{collections::HashMap, sync::Arc};
use tonic::{transport::Server, Request, Response};

// RPC Pool Client
pub struct PoolClient {}

impl PoolClient {
    pub fn init() -> impl PoolOperations {
        Self {}
    }
}

/// Implement pool operations supported by the Pool RPC client.
/// This converts the client side data into a RPC request.
#[tonic::async_trait]
impl PoolOperations for PoolClient {
    // /// Issue the pool create operation over RPC.
    // async fn create(&self, pool: &(dyn PoolInfo + Sync + Send)) -> Result<(), ()> {
    //     let mut client = CoreGrpcClient::connect("http://[::1]:50051").await.unwrap();
    //     let req: CreatePoolRequest = pool.into();
    //     let _response = client.create_pool(req).await.unwrap();
    //     Ok(())
    // }

    /// Issue the pool destroy operation over RPC.
    async fn destroy(&self, pool_id: String) {
        let mut client = CoreGrpcClient::connect("http://10.1.0.4:50051")
            .await
            .unwrap();
        let req = DestroyPoolRequest { pool_id };
        let _response = client.destroy_pool(req).await.unwrap();
    }
}

impl From<&(dyn PoolInfo + Send + Sync)> for CreatePoolRequest {
    fn from(data: &(dyn PoolInfo + Send + Sync)) -> Self {
        Self {
            node_id: data.node_id(),
            pool_id: data.pool_id(),
            disks: data.disks().unwrap(),
            labels: data.labels().map(|l| PoolLabel { labels: l }),
        }
    }
}

// impl From<&(dyn PoolInfo + Send + Sync)> for DestroyPoolRequest {
//     fn from(data: &(dyn PoolInfo + Send + Sync)) -> Self {
//         Self {
//             node_id: data.node_id(),
//             pool_id: data.pool_id(),
//         }
//     }
// }

// RPC Pool Server
pub struct PoolServer {
    // Service which executes the operations.
    service: Arc<dyn PoolOperations + Send + Sync>,
}

impl Drop for PoolServer {
    fn drop(&mut self) {
        println!("DROPPING POOL SERVER")
    }
}

impl PoolServer {
    /// Create a new pool RPC server.
    /// This registers the service that should be called to satisfy the pool operations.
    pub async fn init(service: Arc<dyn PoolOperations + Send + Sync>) {
        println!("Starting Pool Server");
        tokio::spawn(async {
            let server = Self { service };
            let _ = server.start().await;
        });
        println!("Finished starting Pool Server");
    }

    /// Start the RPC server.
    async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = "10.1.0.4:50051".parse()?;
        Server::builder()
            .add_service(CoreGrpcServer::new(self))
            .serve(addr)
            .await?;
        Ok(())
    }
}

// Implementation of the RPC methods.
#[tonic::async_trait]
impl CoreGrpc for PoolServer {
    // async fn create_pool(
    //     &self,
    //     request: Request<CreatePoolRequest>,
    // ) -> Result<Response<Pool>, Status> {
    //     let req = request.into_inner().clone();
    //     // Dispatch the create call to the registered service.
    //     let pool_id = req.pool_id.clone();
    //     let result = self.service.create(&req).await;
    //
    //     // TODO: Fill in response
    //     Ok(Response::new(Pool {
    //         definition: None,
    //         state: None,
    //     }))
    // }

    async fn destroy_pool(
        &self,
        request: Request<DestroyPoolRequest>,
    ) -> Result<Response<()>, tonic::Status> {
        let req = request.into_inner();
        // Dispatch the destroy call to the registered service.
        self.service.destroy(req.pool_id).await;
        Ok(Response::new(()))
    }
}

impl PoolInfo for CreatePoolRequest {
    fn pool_id(&self) -> String {
        self.pool_id.clone()
    }

    fn node_id(&self) -> String {
        self.node_id.clone()
    }

    fn disks(&self) -> Option<Vec<String>> {
        if self.disks.is_empty() {
            Some(self.disks.clone())
        } else {
            None
        }
    }

    fn labels(&self) -> Option<HashMap<String, String>> {
        self.labels
            .as_ref()
            .map(|pool_label| pool_label.labels.clone())
    }
}

impl PoolInfo for DestroyPoolRequest {
    fn pool_id(&self) -> String {
        self.pool_id.clone()
    }

    fn node_id(&self) -> String {
        "".into()
    }

    fn disks(&self) -> Option<Vec<String>> {
        None
    }

    fn labels(&self) -> Option<HashMap<String, String>> {
        None
    }
}
