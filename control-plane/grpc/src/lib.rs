pub mod pool_transport;
pub mod traits;

pub mod grpc {
    tonic::include_proto!("grpc");
}
