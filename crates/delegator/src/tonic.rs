use tonic::{transport::Server, Request, Response, Status};

pub mod proto {
    tonic::include_proto!("delegator");
}
use crate::tonic::proto::delegator_service_server::DelegatorService;
use proto::{DelegateRequest, DelegateResponse};

#[derive(Default)]
pub struct DelegatorGRPCServer {}

#[tonic::async_trait]
impl DelegatorService for DelegatorGRPCServer {
    async fn delegate(
        &self,
        request: Request<DelegateRequest>,
    ) -> Result<Response<DelegateResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = DelegateResponse { message: format!("Hello {}!", request.into_inner().name) };
        Ok(Response::new(reply))
    }
}
