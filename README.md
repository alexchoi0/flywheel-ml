# Flywheel-ML

ML pipeline framework in Rust. Flywheel-ML implements a self-improving loop where the same pipeline that runs inference also collects ground truth feedback, automatically generating labeled training data.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         FLYWHEEL-ML ARCHITECTURE                            │
│                                                                             │
│   ┌──────────────────────────────────────────────────────────────────────┐  │
│   │                      flywheel-ml-server                              │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐   │  │
│   │  │ gRPC API    │  │  Pipeline   │  │   Health    │  │  Metrics   │   │  │
│   │  │ (tonic)     │  │  Registry   │  │  Tracker    │  │ (prom)     │   │  │
│   │  └─────────────┘  └─────────────┘  └──────┬──────┘  └────────────┘   │  │
│   │                                           │                          │  │
│   │                                    ┌──────▼──────┐                   │  │
│   │                                    │   SeaORM    │                   │  │
│   │                                    │  (Postgres) │                   │  │
│   │                                    └─────────────┘                   │  │
│   └──────────────────────────────────────────────────────────────────────┘  │
│                              ▲                                              │
│                              │ gRPC                                         │
│   ┌──────────────────────────┴───────────────────────────────────────────┐  │
│   │                         flywheel-ml (CLI)                            │  │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐   │  │
│   │  │ health      │  │ logs        │  │ drift       │  │ model      │   │  │
│   │  │ stats       │  │ graph       │  │ export      │  │ validate   │   │  │
│   │  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘   │  │
│   └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   ┌──────────────────────────────────────────────────────────────────────┐  │
│   │                    PIPELINE EXECUTION                                │  │
│   │                                                                      │  │
│   │   ┌─────────┐   ┌──────────┐   ┌───────────┐   ┌─────────┐           │  │
│   │   │ Source  │──▶│ Feature  │──▶│ Inference │──▶│  Sinks  │           │  │
│   │   │         │   │ Extract  │   │ (gRPC→Py) │   │         │           │  │
│   │   └─────────┘   └──────────┘   └─────┬─────┘   └─────────┘           │  │
│   │                                      │                               │  │
│   │                     ┌────────────────┼────────────────┐              │  │
│   │                     │                ▼                │              │  │
│   │                     │   ┌─────────────────────┐       │              │  │
│   │                     │   │  Prediction Store   │       │              │  │
│   │                     │   └──────────┬──────────┘       │              │  │
│   │  ┌───────────┐      │              │                  │              │  │
│   │  │ Feedback  │──────┼──────────────┼──────────────────┤              │  │
│   │  │  Events   │      │              ▼                  │              │  │
│   │  └───────────┘      │   ┌─────────────────────┐       │              │  │
│   │                     │   │   Feedback Join     │       │              │  │
│   │                     │   │   (Label + Export)  │       │              │  │
│   │                     │   └──────────┬──────────┘       │              │  │
│   │                     │              │                  │              │  │
│   │                     │              ▼                  │              │  │
│   │                     │   ┌─────────────────────┐       │              │  │
│   │  ┌───────────┐      │   │  Training Export    │       │              │  │
│   │  │   Drift   │◀─────┼───│  (S3 Parquet)       │       │              │  │
│   │  │ Detection │      │   └─────────────────────┘       │              │  │
│   │  └───────────┘      │        THE FLYWHEEL             │              │  │
│   │                     └─────────────────────────────────┘              │  │
│   └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Features

- **Feature Extraction**: Extract and transform features from streaming data
- **ML Inference**: gRPC-based inference to Python model servers with circuit breaker
- **Drift Detection**: Statistical (PSI, KL divergence) and performance-based drift monitoring
- **Feedback Loop**: Automatic labeling from implicit signals (incidents, alerts, etc.)
- **Training Export**: Export labeled data to S3/Parquet for model retraining
- **Kubernetes Native**: Pipeline CRUD via `kubectl`, CRD-based configuration

## Building

```bash
cargo build --release
```

Binaries are output to `target/release/`:
- `flywheel-ml` - CLI client
- `flywheel-ml-server` - Server binary

## CLI Usage

Pipeline CRUD operations use `kubectl`:

```bash
kubectl apply -f pipeline.yaml
kubectl get flywheelpipelines
kubectl delete flywheelpipeline <name>
```

Flywheel-ML CLI provides operational commands:

```bash
# Health and monitoring
flywheel-ml health                              # Overall health
flywheel-ml health pipeline anomaly-detection   # Pipeline health
flywheel-ml logs anomaly-detection              # View logs
flywheel-ml logs -f anomaly-detection           # Follow logs

# Model management
flywheel-ml model list                          # List registered models
flywheel-ml model show isolation-forest-v3      # Model details + metrics
flywheel-ml model history isolation-forest-v3   # Performance history
flywheel-ml model compare v2 v3                 # Compare versions

# Drift monitoring
flywheel-ml drift status                        # Current drift status
flywheel-ml drift history -p anomaly-detection  # Drift event history

# Statistics
flywheel-ml stats                               # All pipeline stats
flywheel-ml stats predictions -p anomaly        # Prediction stats
flywheel-ml stats feedback -p anomaly           # Feedback stats
flywheel-ml stats training -p anomaly           # Training data stats

# Training data export
flywheel-ml export -p anomaly-detection -o ./data/

# Validation
flywheel-ml validate -f pipeline.yaml           # Validate manifest

# Visualization
flywheel-ml graph anomaly-detection             # Show DAG
flywheel-ml graph anomaly-detection -o dag.png  # Export as image
```

## Pipeline Manifest

```yaml
apiVersion: flywheel-ml.io/v1
kind: FlywheelPipeline
metadata:
  name: anomaly-detection
  namespace: production
spec:
  source: kafka-metrics

  stages:
    - id: features
      type: feature-extraction
      config:
        features:
          - name: cpu_usage
            source_field: $.metrics.cpu.usage_percent
            transform:
              normalize:
                min: 0
                max: 100
          - name: error_rate
            source_field: $.metrics.http.error_rate
            transform:
              clip:
                min: 0
                max: 1

    - id: inference
      type: ml-inference
      config:
        model_endpoint: model-server:50051
        model_id: isolation-forest-v3
        input_features: [cpu_usage, error_rate]
        output_field: anomaly_prediction
        timeout_ms: 100
        batch_size: 64
        fallback: passthrough

    - id: drift-monitor
      type: drift-detection
      config:
        mode: shadow
        baseline_uri: s3://ml-models/baselines/v3
        window_size: 10000
        thresholds:
          psi: 0.25
          kl_divergence: 0.1
        on_drift: alert

  feedback:
    source: incident-events
    join_key: $.metadata.trace_id
    max_delay_hours: 24
    labels:
      - event: incident_created
        label: anomaly
        confidence: 0.95
      - event: alert_dismissed
        label: normal
        confidence: 0.70

  training_export:
    destination_uri: s3://training-data/anomaly/
    format: parquet
    partition_by: [date, model_version]
    sampling:
      strategy: stratified
      positive_rate: 1.0
      negative_rate: 0.05

  sinks:
    - name: alertmanager
      condition: "anomaly_prediction.is_anomaly && anomaly_prediction.score > 0.8"
    - name: analytics
      all: true
```

## Stage Types

| Stage Type | Description |
|------------|-------------|
| `feature-extraction` | Extract and transform features from records |
| `ml-inference` | Run model inference via gRPC |
| `drift-detection` | Monitor feature/performance drift |
| `feedback-join` | Join predictions with ground truth |
| `training-export` | Export labeled training data |

## Server Configuration

```bash
flywheel-ml-server \
  --bind-address 0.0.0.0:50051 \
  --metrics-address 0.0.0.0:9090 \
  --db-url postgres://flywheel:password@localhost/flywheel
```

Or via config file:

```toml
[server]
bind_address = "0.0.0.0:50051"
metrics_address = "0.0.0.0:9090"

[database]
url = "postgres://flywheel:password@localhost/flywheel"

[conveyor]
router_endpoint = "conveyor-router:50051"
```

## Project Structure

```
flywheel/
├── crates/
│   ├── flywheel-ml/              # CLI binary
│   ├── flywheel-ml-server/       # Server binary
│   ├── flywheel-ml-core/         # Core traits and types
│   ├── flywheel-ml-proto/        # gRPC protocol definitions
│   ├── flywheel-ml-db/           # SeaORM database layer
│   ├── flywheel-ml-dsl/          # Pipeline DSL
│   ├── flywheel-ml-client/       # Client library
│   ├── flywheel-ml-inference/    # Inference runtime
│   ├── flywheel-ml-drift/        # Drift detection
│   ├── flywheel-ml-training/     # Training data export
│   ├── flywheel-ml-transform/    # Transform implementations
│   └── flywheel-ml-operator/     # Kubernetes operator
└── examples/
    └── anomaly-detection.yaml
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
