use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Pipelines::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Pipelines::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Pipelines::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Pipelines::Namespace).string_len(255).not_null())
                    .col(ColumnDef::new(Pipelines::SpecHash).string_len(64).not_null())
                    .col(ColumnDef::new(Pipelines::SpecYaml).text().not_null())
                    .col(ColumnDef::new(Pipelines::Status).string_len(32).not_null())
                    .col(ColumnDef::new(Pipelines::ConveyorPipelineId).string_len(255))
                    .col(ColumnDef::new(Pipelines::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Pipelines::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_pipelines_name_namespace")
                    .table(Pipelines::Table)
                    .col(Pipelines::Name)
                    .col(Pipelines::Namespace)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PipelineRuns::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PipelineRuns::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(PipelineRuns::PipelineId).uuid().not_null())
                    .col(ColumnDef::new(PipelineRuns::Status).string_len(32).not_null())
                    .col(ColumnDef::new(PipelineRuns::StartedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(PipelineRuns::EndedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(PipelineRuns::RecordsProcessed).big_integer().not_null().default(0))
                    .col(ColumnDef::new(PipelineRuns::RecordsFailed).big_integer().not_null().default(0))
                    .col(ColumnDef::new(PipelineRuns::ErrorMessage).text())
                    .foreign_key(
                        ForeignKey::create()
                            .from(PipelineRuns::Table, PipelineRuns::PipelineId)
                            .to(Pipelines::Table, Pipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ModelVersions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ModelVersions::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ModelVersions::ModelId).string_len(255).not_null())
                    .col(ColumnDef::new(ModelVersions::Version).string_len(64).not_null())
                    .col(ColumnDef::new(ModelVersions::ModelType).string_len(64).not_null())
                    .col(ColumnDef::new(ModelVersions::Endpoint).string_len(255).not_null())
                    .col(ColumnDef::new(ModelVersions::Status).string_len(32).not_null())
                    .col(ColumnDef::new(ModelVersions::Accuracy).double())
                    .col(ColumnDef::new(ModelVersions::LatencyP99Ms).big_integer())
                    .col(ColumnDef::new(ModelVersions::DeployedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_model_versions_model_id")
                    .table(ModelVersions::Table)
                    .col(ModelVersions::ModelId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(DriftEvents::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(DriftEvents::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(DriftEvents::PipelineId).uuid().not_null())
                    .col(ColumnDef::new(DriftEvents::ModelId).string_len(255).not_null())
                    .col(ColumnDef::new(DriftEvents::DriftType).string_len(32).not_null())
                    .col(ColumnDef::new(DriftEvents::Severity).string_len(32).not_null())
                    .col(ColumnDef::new(DriftEvents::PsiScore).double())
                    .col(ColumnDef::new(DriftEvents::KlDivergence).double())
                    .col(ColumnDef::new(DriftEvents::AccuracyDelta).double())
                    .col(ColumnDef::new(DriftEvents::DetectedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(DriftEvents::ResolvedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .from(DriftEvents::Table, DriftEvents::PipelineId)
                            .to(Pipelines::Table, Pipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Predictions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Predictions::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Predictions::PipelineId).uuid().not_null())
                    .col(ColumnDef::new(Predictions::ModelId).string_len(255).not_null())
                    .col(ColumnDef::new(Predictions::ModelVersion).string_len(64).not_null())
                    .col(ColumnDef::new(Predictions::FeaturesJson).json().not_null())
                    .col(ColumnDef::new(Predictions::PredictionJson).json().not_null())
                    .col(ColumnDef::new(Predictions::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Predictions::FeedbackId).uuid())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Predictions::Table, Predictions::PipelineId)
                            .to(Pipelines::Table, Pipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_predictions_pipeline_created")
                    .table(Predictions::Table)
                    .col(Predictions::PipelineId)
                    .col(Predictions::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Feedback::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Feedback::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Feedback::PredictionId).uuid().not_null())
                    .col(ColumnDef::new(Feedback::GroundTruth).string_len(255).not_null())
                    .col(ColumnDef::new(Feedback::Source).string_len(32).not_null())
                    .col(ColumnDef::new(Feedback::Confidence).double().not_null())
                    .col(ColumnDef::new(Feedback::ReceivedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Feedback::Exported).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_feedback_exported")
                    .table(Feedback::Table)
                    .col(Feedback::Exported)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Feedback::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Predictions::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(DriftEvents::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(ModelVersions::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(PipelineRuns::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Pipelines::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(Iden)]
enum Pipelines {
    Table,
    Id,
    Name,
    Namespace,
    SpecHash,
    SpecYaml,
    Status,
    ConveyorPipelineId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PipelineRuns {
    Table,
    Id,
    PipelineId,
    Status,
    StartedAt,
    EndedAt,
    RecordsProcessed,
    RecordsFailed,
    ErrorMessage,
}

#[derive(Iden)]
enum ModelVersions {
    Table,
    Id,
    ModelId,
    Version,
    ModelType,
    Endpoint,
    Status,
    Accuracy,
    LatencyP99Ms,
    DeployedAt,
}

#[derive(Iden)]
enum DriftEvents {
    Table,
    Id,
    PipelineId,
    ModelId,
    DriftType,
    Severity,
    PsiScore,
    KlDivergence,
    AccuracyDelta,
    DetectedAt,
    ResolvedAt,
}

#[derive(Iden)]
enum Predictions {
    Table,
    Id,
    PipelineId,
    ModelId,
    ModelVersion,
    FeaturesJson,
    PredictionJson,
    CreatedAt,
    FeedbackId,
}

#[derive(Iden)]
enum Feedback {
    Table,
    Id,
    PredictionId,
    GroundTruth,
    Source,
    Confidence,
    ReceivedAt,
    Exported,
}
