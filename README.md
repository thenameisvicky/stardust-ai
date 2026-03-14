# Architecture

```mermaid
flowchart TD
    REST[REST API\n(Axum Router)]
    WS[WebSocket Endpoint\n(/ws)]
    AppState[AppState\n- RabbitMQ\n- WS Broadcast\n- HTTP Client\n- Config\n- Prometheus]
    LLM[LLM Module\n(Ollama API)]
    Chat[Chat Agent Module\n(routes/logic)]
    Analytics[Analytics Module]
    Campaigns[Campaigns Module]
    Feed[Feed Module]

    REST -->|Routes| Chat
    REST -->|Routes| Analytics
    REST -->|Routes| Campaigns
    REST -->|Routes| Feed

    Chat --> AppState
    Analytics --> AppState
    Campaigns --> AppState
    Feed --> AppState

    WS --> AppState
    AppState --> LLM
    LLM --> AppState
    WS -->|Broadcast| AppState
