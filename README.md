# Architecture

src
│
├── main.rs
├── state.rs
│
├── api
│   └── router.rs
│
├── core
│   ├── queue
│   │   ├── producer.rs
│   │   └── consumer.rs
│   │
│   ├── websocket
│   │   └── handler.rs
│   │
│   └── metrics
│       └── prometheus.rs
│
└── modules
    ├── agent
    │   ├── handlers.rs
    │   └── routes.rs
    │
    ├── campaigns
    ├── feed
    └── analytics
