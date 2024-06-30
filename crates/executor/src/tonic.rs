pub mod proto {
    tonic::include_proto!("executor");
}
use proto::executor_service_server::ExecutorService;
use proto::{ExecutorRequest, ExecutorResponse};
use tonic::{Request, Response, Status};

pub struct ExecutorGRPCServer {}

impl ExecutorGRPCServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for ExecutorGRPCServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl ExecutorService for ExecutorGRPCServer {
    async fn executor(
        &self,
        request: Request<ExecutorRequest>,
    ) -> Result<Response<ExecutorResponse>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = ExecutorResponse { msg: format!("Hello {}!", request.into_inner().msg) };
        Ok(Response::new(reply))
    }
}
