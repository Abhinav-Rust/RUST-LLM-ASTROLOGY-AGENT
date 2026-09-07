# RUST-LLM-ASTROLOGY-AGENT

[![clippy](https://img.shields.io/badge/clippy-passing-success.svg)](https://github.com/your-username/rust-llm-astrology-agent/actions)
[![Rust](https://img.shields.io/badge/Rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **⚠️ Domain IP Redacted** — All proprietary astrological logic (master prompts, Dasha calculations, Yoga detection, and the Vedic rules engine) has been stripped from this repository. Function signatures and data structures are preserved as stubs to demonstrate the architecture. This repo is published as a **structural showcase only**.

---

## About

**A fault-tolerant, multi-agent LLM pipeline built in Rust, showcased via a Vedic Astrology reasoning engine.**

This project demonstrates a high-performance system designed to orchestrate complex, multi-step AI workflows against rate-limited APIs. While the system was originally deployed for production Vedic astrology readings, the core patterns and architecture generalize to any complex multi-agent domain requiring:

- **Multi-Agent Orchestration** — Agent 1 extracts structured temporal parameters from natural language; Agent 2 generates long-form analytical output conditioned on deterministic astronomical data.
- **Resilient API Cadence Control** — Custom exponential backoff with dynamic rate-limit parsing directly from error message bodies, `Retry-After` header respect, and configurable retry ceilings.
- **Connection Lifecycle & HTTP Pooling** — Deliberate connection pooling and lifecycle management (`pool_idle_timeout`, `pool_max_idle_per_host`, disabled TCP keepalive) passed across geocoding and LLM requests to survive rate-limited endpoints.
- **Atomic Persistence Layer** — Robust SQLite database operations with atomic transactions (`conn.transaction()`) ensuring consistency across client profiles and historical readings.
- **Secure Input & Path Sanitization** — Query parameter URL encoding (`.query(...)`) for external geocoding calls and clean filename sanitization with underscore collapsing and trimming.
- **Zero-Copy Prompt Pipelines & Privacy** — Multi-stage prompt assembly with PII anonymization layers before API submission.
- **Comprehensive Unit Testing** — In-memory SQLite transaction tests, API backoff priority verification, offline timezone resolution tests, and filename sanitization coverage.

---

## Architecture & Engineering

The comprehensive architecture (including our 3-tier API backoff strategy, multi-agent pipeline, connection lifecycle, and data anonymization layer) is detailed in our documentation.

Please see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full details and Mermaid.js diagrams.

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (Edition 2024) |
| Async Runtime | Tokio |
| HTTP Client | Reqwest (connection pooling & URL parameter encoding) |
| LLM Provider | Google Gemini API (`gemini-3.1-flash-lite`) |
| Database | SQLite via rusqlite (with atomic transactions) |
| Geocoding | Nominatim (OpenStreetMap) |
| Timezone Resolution | `tzf-rs` (offline, embedded TZ database) |
| Historical TZ Offsets | `chrono-tz` |
| TUI | `dialoguer` + `console` |
| Serialization | `serde` + `serde_json` |

---

## Project Structure

```
src/
├── lib.rs        # Shared library module declarations
├── main.rs       # CLI/TUI, orchestration, atomic SQLite DB layer, HTML report exporter
├── api.rs        # Gemini API client, 3-tier backoff engine, Agent 1 date extractor
├── geo.rs        # Nominatim geocoding with connection pooling & offline timezone resolution
├── utils.rs      # Filename sanitization utilities (collapse & trim)
├── math.rs       # [STUBBED] Planetary position calculations
├── rules.rs      # [STUBBED] Vedic astrology rules engine
├── dasha.rs      # [STUBBED] Vimshottari Dasha timeline generator
└── bin/
    └── verify_db.rs # Standalone DB inspection utility
```

---

## Running

```bash
# Set your Gemini API key
export GEMINI_API_KEY="your-key-here"

# Run full test suite (unit tests for DB, API, Geo, and Utils)
cargo test

# Check code health & lints
cargo check && cargo clippy --all-targets && cargo fmt --check

# Build and run interactive TUI
cargo run --bin rust_llm_astrology_agent
```

> **Note:** The stubbed math/rules/dasha modules return dummy data. The pipeline will execute end-to-end but the generated readings will lack real astronomical input.

---

## License

This repository is published for portfolio/showcase purposes only. The architecture, patterns, and non-redacted code are available for reference. The redacted domain IP remains proprietary.
