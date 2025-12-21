use flywheel_ml_db::Database;
use flywheel_ml_proto::control_service_server::ControlService;
use flywheel_ml_proto::{
    CreatePipelineRequest, CreatePipelineResponse,
    UpdatePipelineRequest, UpdatePipelineResponse,
    DeletePipelineRequest, DeletePipelineResponse,
    GetPipelineRequest, GetPipelineResponse,
    ListPipelinesRequest, ListPipelinesResponse,
    EnablePipelineRequest, EnablePipelineResponse,
    DisablePipelineRequest, DisablePipelineResponse,
    RegisterModelRequest, RegisterModelResponse,
    UnregisterModelRequest, UnregisterModelResponse,
    GetModelRequest, GetModelResponse,
    ListModelsRequest, ListModelsResponse,
};
use tonic::{Request, Response, Status};

pub struct ControlServiceImpl {
    _db: Database,
}

impl ControlServiceImpl {
    pub fn new(db: Database) -> Self {
        Self { _db: db }
    }
}

#[tonic::async_trait]
impl ControlService for ControlServiceImpl {
    async fn create_pipeline(
        &self,
        _request: Request<CreatePipelineRequest>,
    ) -> Result<Response<CreatePipelineResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn update_pipeline(
        &self,
        _request: Request<UpdatePipelineRequest>,
    ) -> Result<Response<UpdatePipelineResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn delete_pipeline(
        &self,
        _request: Request<DeletePipelineRequest>,
    ) -> Result<Response<DeletePipelineResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_pipeline(
        &self,
        _request: Request<GetPipelineRequest>,
    ) -> Result<Response<GetPipelineResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_pipelines(
        &self,
        _request: Request<ListPipelinesRequest>,
    ) -> Result<Response<ListPipelinesResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn enable_pipeline(
        &self,
        _request: Request<EnablePipelineRequest>,
    ) -> Result<Response<EnablePipelineResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn disable_pipeline(
        &self,
        _request: Request<DisablePipelineRequest>,
    ) -> Result<Response<DisablePipelineResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn register_model(
        &self,
        _request: Request<RegisterModelRequest>,
    ) -> Result<Response<RegisterModelResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn unregister_model(
        &self,
        _request: Request<UnregisterModelRequest>,
    ) -> Result<Response<UnregisterModelResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_model(
        &self,
        _request: Request<GetModelRequest>,
    ) -> Result<Response<GetModelResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_models(
        &self,
        _request: Request<ListModelsRequest>,
    ) -> Result<Response<ListModelsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
}
