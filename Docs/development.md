# Layer 1 — Data Quality Layer
- [x] Refactor current architecture to scalable one - March 14, 2026
- [x] Introduce static client, test full flow - March 14, 2026
- [x] Use AppState across modules - March 14, 2026
- [x] Implement response streaming per client (unique client id) - March 14, 2026
- [ ] Implement chunking (fixed + overlap)
- [ ] Define document ingestion standards (PDF / MD / codebase)
- [ ] Build document preparation guide aligned with chunking strategy
- [ ] Add metadata strategy (source, file, section, timestamp)

# Layer 2 — Retrieval Quality Layer
- [ ] Implement dense retrieval (vector search via Qdrant)
- [ ] Add BM25 / keyword retrieval layer
- [ ] Merge dense + sparse retrieval (hybrid search)
- [ ] Add basic reranker
- [ ] Improve query rewriting (LLM-based query expansion)
- [ ] Implement filtering by metadata (source, type, tenant)
- [ ] Add similarity threshold tuning system
- [ ] Evaluate retrieval quality metrics (precision@k baseline)

# Layer 3 — Context Builder Layer
- [ ] Context builder (top-k selection + token budget control)
- [ ] Source attribution system (doc + chunk tracing)
- [ ] Anti-hallucination prompt system
- [ ] Guardrails (refuse / fallback responses)
- [ ] Context compression (reduce redundant chunks)
- [ ] Context ranking (score-based ordering of chunks)
- [ ] Token window management per model

# Layer 4 — LLM Orchestration Layer
- [x] Implement token streaming to client - March 15, 2026
- [x] Configure Ollama parallel execution (3 concurrent prompts) - March 15, 2026
- [ ] Prompt template system (task-specific prompts)
- [ ] Model routing layer (small vs large model fallback)
- [ ] Streaming abstraction layer for all LLM providers
- [ ] Add provider strategy abstraction (Ollama / OpenAI / etc.)
- [ ] Add evaluation system for response quality
- [ ] Add structured output modes (JSON / tool-like responses)
- [ ] Add cost-aware model selection

# Layer 5 — SaaS Layer
- [x] Containerize and publish to Docker - March 18, 2026
- [x] Create landing page - March 18, 2026
- [x] Test full flow (Docker pull + run) - March 18, 2026
- [x] Setup GitHub Actions for deployment (pages + docker publish) - March 18, 2026
- [ ] Multi-tenant architecture (org/user isolation in Qdrant)
- [ ] API authentication (API keys + JWT)
- [ ] Rate limiting per tenant
- [ ] Usage tracking (queries, embeddings, tokens)
- [ ] Billing system integration (Stripe / Chargebee)
- [ ] Cost per query tracking
- [ ] Resource usage tracking (CPU / RAM / storage)
- [ ] Logging strategy (what to store / discard)
- [ ] Observability dashboard (Prometheus metrics)
- [ ] Telegram notifications for weekly usage reports
- [ ] Deployment strategy for self-hosted + cloud
- [ ] Home server / infra cost optimization plan

# Layer 6 — Advanced Scale Layer
- [ ] Graph RAG
- [ ] Agentic RAG system
- [ ] Evaluation dashboard (retrieval + generation scoring)
- [ ] Replace RabbitMQ with scheduler/event system (if needed)
- [ ] Vector DB optimization strategy (sharding / scaling Qdrant)
- [ ] vLLM integration (high-throughput inference)
- [ ] Codebase indexing agent (GitHub integration → auto ingestion pipeline)