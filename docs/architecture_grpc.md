# Sumbawa Digital Twin: Architecture

This document describes the high-performance Axum (Rust) + PyTorch (Python) gRPC mesh.

## System Architecture

```mermaid
graph TD
    %% Define Styles
    classDef rust fill:#dea584,stroke:#333,stroke-width:2px,color:#000;
    classDef python fill:#4b8bbe,stroke:#333,stroke-width:2px,color:#fff;
    classDef data fill:#2d5d7b,stroke:#333,stroke-width:2px,color:#fff;
    classDef external fill:#f0f0f0,stroke:#333,stroke-width:2px,color:#000,stroke-dasharray: 5 5;

    %% Client/External
    Client([Client / ZeroClaw LLM]):::external
    S3[(Local S3 / MinIO \n STAC & COG)]:::data
    IoT([IoT Sensor Network]):::external

    %% Rust Gateway
    subgraph Rust Gateway Node [API Gateway & Data Lake]
        Axum[Axum REST/WebSocket API]:::rust
        TonicClient[Tonic gRPC Client]:::rust
        GDAL[GDAL-RS In-Memory Cropper]:::rust
    end

    %% Python Inference Node
    subgraph Python Inference Node [Deep Learning Engine]
        gRPCServer[gRPC Server]:::python
        PyTorch[PyTorch / CUDA Inference]:::python
        PINN[Physics-Informed Neural Network \n (Micro-Scale)]:::python
        FNO[Fourier Neural Operator \n (Macro-Scale)]:::python
    end

    %% Connections
    Client -->|HTTP/REST| Axum
    IoT -->|Kafka / MQTT| Axum
    
    Axum <-->|Fetch BBox| GDAL
    GDAL <-->|Byte-Range Requests| S3
    
    Axum -->|Send Matrices| TonicClient
    TonicClient <-->|gRPC Protobuf (Low Latency)| gRPCServer
    
    gRPCServer -->|Tensors| PyTorch
    PyTorch --> PINN
    PyTorch --> FNO
```
