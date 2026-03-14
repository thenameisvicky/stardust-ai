# Architecture

```mermaid
sequenceDiagram
    participant Client
    participant REST_API as REST API (Axum)
    participant WS as WebSocket /ws
    participant Chat as Chat Agent Module
    participant AppState
    participant LLM as LLM Module (Ollama API)

    Client->>REST_API: POST /chat with prompt
    REST_API->>Chat: Forward prompt
    Chat->>AppState: Publish Job to Queue
    AppState->>LLM: Send prompt for inference
    LLM-->>AppState: Return response
    AppState-->>WS: Broadcast response
    WS-->>Client: Receive processed response
