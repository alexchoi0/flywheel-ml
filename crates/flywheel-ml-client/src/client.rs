use flywheel_ml_proto::{
    control_service_client::ControlServiceClient,
    health_service_client::HealthServiceClient,
    inference_service_client::InferenceServiceClient,
    CreatePipelineRequest, CreatePipelineResponse, DeletePipelineRequest, DeletePipelineResponse,
    DisablePipelineRequest, DisablePipelineResponse, EnablePipelineRequest, EnablePipelineResponse,
    FeatureValue, GetDriftStatusRequest, GetDriftStatusResponse, GetHealthRequest,
    GetHealthResponse, GetModelRequest, GetModelResponse, GetPipelineHealthRequest,
    GetPipelineHealthResponse, GetPipelineRequest, GetPipelineResponse, HealthCheckRequest,
    HealthCheckResponse, ListDriftEventsRequest, ListDriftEventsResponse, ListModelsRequest,
    ListModelsResponse, ListPipelinesRequest, ListPipelinesResponse, ModelInfoRequest,
    ModelInfoResponse, PredictBatchRequest, PredictBatchResponse, PredictRequest, PredictResponse,
    RegisterModelRequest, RegisterModelResponse, UnregisterModelRequest, UnregisterModelResponse,
    UpdatePipelineRequest, UpdatePipelineResponse,
};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tonic::transport::Channel;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Request failed: {0}")]
    Request(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<tonic::Status> for ClientError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => ClientError::NotFound(status.message().to_string()),
            tonic::Code::InvalidArgument => {
                ClientError::InvalidArgument(status.message().to_string())
            }
            tonic::Code::DeadlineExceeded => ClientError::Timeout(0),
            tonic::Code::Unauthenticated | tonic::Code::PermissionDenied => {
                ClientError::Unauthorized(status.message().to_string())
            }
            tonic::Code::Unavailable => ClientError::Connection(status.message().to_string()),
            tonic::Code::Internal => ClientError::Internal(status.message().to_string()),
            _ => ClientError::Request(status.message().to_string()),
        }
    }
}

impl From<tonic::transport::Error> for ClientError {
    fn from(err: tonic::transport::Error) -> Self {
        ClientError::Connection(err.to_string())
    }
}

pub struct FlywheelClient {
    endpoint: String,
    channel: Option<Channel>,
}

impl FlywheelClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            channel: None,
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub async fn connect(&mut self) -> Result<(), ClientError> {
        let channel = Channel::from_shared(self.endpoint.clone())
            .map_err(|e| ClientError::Connection(e.to_string()))?
            .connect()
            .await?;
        self.channel = Some(channel);
        tracing::info!(endpoint = %self.endpoint, "Connected to Flywheel-ML server");
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.channel.is_some()
    }

    fn get_channel(&self) -> Result<Channel, ClientError> {
        self.channel
            .clone()
            .ok_or_else(|| ClientError::Connection("Not connected".to_string()))
    }

    pub async fn get_health(&self) -> Result<GetHealthResponse, ClientError> {
        let mut client = HealthServiceClient::new(self.get_channel()?);
        let response = client.get_health(GetHealthRequest {}).await?;
        Ok(response.into_inner())
    }

    pub async fn get_pipeline_health(
        &self,
        pipeline_id: impl Into<String>,
    ) -> Result<GetPipelineHealthResponse, ClientError> {
        let mut client = HealthServiceClient::new(self.get_channel()?);
        let response = client
            .get_pipeline_health(GetPipelineHealthRequest {
                pipeline_id: pipeline_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn get_drift_status(
        &self,
        pipeline_id: impl Into<String>,
        model_id: impl Into<String>,
    ) -> Result<GetDriftStatusResponse, ClientError> {
        let mut client = HealthServiceClient::new(self.get_channel()?);
        let response = client
            .get_drift_status(GetDriftStatusRequest {
                pipeline_id: pipeline_id.into(),
                model_id: model_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn list_drift_events(
        &self,
        pipeline_id: impl Into<String>,
        model_id: Option<String>,
        limit: i32,
    ) -> Result<ListDriftEventsResponse, ClientError> {
        let mut client = HealthServiceClient::new(self.get_channel()?);
        let response = client
            .list_drift_events(ListDriftEventsRequest {
                pipeline_id: pipeline_id.into(),
                model_id: model_id.unwrap_or_default(),
                limit,
                cursor: String::new(),
                since: None,
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn create_pipeline(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        spec_yaml: impl Into<String>,
    ) -> Result<CreatePipelineResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .create_pipeline(CreatePipelineRequest {
                name: name.into(),
                namespace: namespace.into(),
                spec_yaml: spec_yaml.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn update_pipeline(
        &self,
        pipeline_id: impl Into<String>,
        spec_yaml: impl Into<String>,
    ) -> Result<UpdatePipelineResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .update_pipeline(UpdatePipelineRequest {
                pipeline_id: pipeline_id.into(),
                spec_yaml: spec_yaml.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn delete_pipeline(
        &self,
        pipeline_id: impl Into<String>,
    ) -> Result<DeletePipelineResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .delete_pipeline(DeletePipelineRequest {
                pipeline_id: pipeline_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn get_pipeline(
        &self,
        pipeline_id: impl Into<String>,
    ) -> Result<GetPipelineResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .get_pipeline(GetPipelineRequest {
                pipeline_id: pipeline_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn list_pipelines(
        &self,
        namespace: Option<String>,
        limit: i32,
        cursor: Option<String>,
    ) -> Result<ListPipelinesResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .list_pipelines(ListPipelinesRequest {
                namespace: namespace.unwrap_or_default(),
                limit,
                cursor: cursor.unwrap_or_default(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn enable_pipeline(
        &self,
        pipeline_id: impl Into<String>,
    ) -> Result<EnablePipelineResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .enable_pipeline(EnablePipelineRequest {
                pipeline_id: pipeline_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn disable_pipeline(
        &self,
        pipeline_id: impl Into<String>,
    ) -> Result<DisablePipelineResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .disable_pipeline(DisablePipelineRequest {
                pipeline_id: pipeline_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn register_model(
        &self,
        model_id: impl Into<String>,
        model_name: impl Into<String>,
        version: impl Into<String>,
        model_type: impl Into<String>,
        endpoint: impl Into<String>,
        input_features: Vec<String>,
        output_field: impl Into<String>,
        labels: HashMap<String, String>,
    ) -> Result<RegisterModelResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .register_model(RegisterModelRequest {
                model_id: model_id.into(),
                model_name: model_name.into(),
                version: version.into(),
                model_type: model_type.into(),
                endpoint: endpoint.into(),
                input_features,
                output_field: output_field.into(),
                labels,
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn unregister_model(
        &self,
        model_id: impl Into<String>,
    ) -> Result<UnregisterModelResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .unregister_model(UnregisterModelRequest {
                model_id: model_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn get_model(
        &self,
        model_id: impl Into<String>,
    ) -> Result<GetModelResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .get_model(GetModelRequest {
                model_id: model_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn list_models(
        &self,
        limit: i32,
        cursor: Option<String>,
    ) -> Result<ListModelsResponse, ClientError> {
        let mut client = ControlServiceClient::new(self.get_channel()?);
        let response = client
            .list_models(ListModelsRequest {
                limit,
                cursor: cursor.unwrap_or_default(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn predict(
        &self,
        model_id: impl Into<String>,
        features: HashMap<String, FeatureValue>,
    ) -> Result<PredictResponse, ClientError> {
        let mut client = InferenceServiceClient::new(self.get_channel()?);
        let response = client
            .predict(PredictRequest {
                request_id: uuid::Uuid::new_v4().to_string(),
                model_id: model_id.into(),
                features,
                timestamp: None,
                metadata: HashMap::new(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn predict_batch(
        &self,
        model_id: impl Into<String>,
        requests: Vec<PredictRequest>,
    ) -> Result<PredictBatchResponse, ClientError> {
        let mut client = InferenceServiceClient::new(self.get_channel()?);
        let response = client
            .predict_batch(PredictBatchRequest {
                batch_id: uuid::Uuid::new_v4().to_string(),
                model_id: model_id.into(),
                requests,
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn predict_stream(
        &self,
        requests: impl futures::Stream<Item = PredictRequest> + Send + 'static,
    ) -> Result<impl futures::Stream<Item = Result<PredictResponse, ClientError>>, ClientError>
    {
        let mut client = InferenceServiceClient::new(self.get_channel()?);
        let response = client.predict_stream(requests).await?;
        Ok(futures::StreamExt::map(response.into_inner(), |r| {
            r.map_err(ClientError::from)
        }))
    }

    pub async fn get_model_info(
        &self,
        model_id: impl Into<String>,
    ) -> Result<ModelInfoResponse, ClientError> {
        let mut client = InferenceServiceClient::new(self.get_channel()?);
        let response = client
            .get_model_info(ModelInfoRequest {
                model_id: model_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn health_check(
        &self,
        model_id: impl Into<String>,
    ) -> Result<HealthCheckResponse, ClientError> {
        let mut client = InferenceServiceClient::new(self.get_channel()?);
        let response = client
            .health_check(HealthCheckRequest {
                model_id: model_id.into(),
            })
            .await?;
        Ok(response.into_inner())
    }
}

pub struct FlywheelClientBuilder {
    endpoint: String,
    timeout: Duration,
}

impl FlywheelClientBuilder {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn build(self) -> Result<FlywheelClient, ClientError> {
        let mut client = FlywheelClient::new(self.endpoint);
        client.connect().await?;
        Ok(client)
    }
}
