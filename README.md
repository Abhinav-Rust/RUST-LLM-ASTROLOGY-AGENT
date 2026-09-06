# RUST-LLM-ASTROLOGY-AGENT

[![clippy](https://img.shields.io/badge/clippy-passing-success.svg)](https://github.com/your-username/rust-llm-astrology-agent/actions)
[![Rust](https://img.shields.io/badge/Rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **⚠️ Domain IP Redacted** — All proprietary astrological logic (master prompts, Dasha calculations, Yoga detection, and the Vedic rules engine) has been stripped from this repository. Function signatures and data structures are preserved as stubs to demonstrate the architecture. This repo is published as a **structural showcase only**.

---

## About

**A fault-tolerant, multi-agent LLM pipeline built in Rust, showcased via a Vedic Astrology reasoning engine.**

This project demonstrates a high-performance system designed to orchestrate complex, multi-step AI workflows against rate-limited APIs. While the system was originally deployed for production Vedic astrology readings, the core patterns and architecture generalize to any complex multi-agent domain requiring:

- **Multi-agent orchestration** — Agent 1 extracts structured parameters from natural language; Agent 2 generates long-form analytical output conditioned on deterministic data.
- **Resilient API communication** — Custom exponential backoff with dynamic rate-limit parsing directly from error message bodies, `Retry-After` header respect, and configurable retry ceilings.
- **Connection lifecycle & pooling** — Connection pooling across HTTP API operations combined with deliberate connection tearing via `pool_idle_timeout`, `pool_max_idle_per_host`, and disabled TCP keepalive to survive long inter-request cooldowns on free-tier APIs.
- **Zero-copy prompt pipelines & PII anonymization** — Multi-stage prompt assembly with client data anonymization before external API submission.
- **Robust persistence & transactional integrity** — Embedded SQLite storage with atomic multi-table transactions (`conn.transaction()`), automated schema migrations, client search/fast-tracking, and robust filename sanitization.

---

## Architecture & Engineering

The comprehensive architecture (including our 3-tier API backoff strategy, multi-agent pipeline, and data anonymization layer) has been moved to our documentation folder.

Please see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full details and Mermaid.js diagrams.

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (Edition 2024) |
| Async Runtime | Tokio |
| HTTP Client | Reqwest (with JSON, connection pooling, and connection tuning) |
| LLM Provider | Google Gemini API (v1beta) |
| Database | SQLite via rusqlite (bundled, atomic transactions) |
| Geocoding | Nominatim (OpenStreetMap via URL query encoding) |
| Timezone Resolution | `tzf-rs` (offline, embedded TZ database) |
| Historical TZ Offsets | `chrono-tz` |
| TUI / Wizard | `dialoguer` + `console` |
| Cross-Platform Launch | `open` crate |
| Serialization | `serde` + `serde_json` |

---

## Project Structure

```
src/
├── lib.rs        # Shared library for all modules
├── main.rs       # Interactive TUI/CLI, orchestration, DB layer, transaction manager
├── api.rs        # Gemini API client, 3-tier backoff engine, rate limit parser
├── math.rs       # [STUBBED] Planetary position & house system types
├── rules.rs      # [STUBBED] Vedic astrology rules engine data structures
├── dasha.rs      # [STUBBED] Vimshottari Dasha timeline generator
├── geo.rs        # Geocoding (connection-pooled Nominatim) + historical timezone resolution
├── utils.rs      # Cross-cutting utilities (filename sanitization with underscore collapsing)
└── bin/
    └── verify_db.rs # Standalone DB inspection utility
```

---

## Running & Testing

```bash
# Set your Gemini API key
export GEMINI_API_KEY="your-key-here"

# Run all verification checks (unit tests, clippy, formatting)
cargo test
cargo check
cargo clippy --all-targets
cargo fmt --check

# Build and run interactive console program
cargo run --bin rust_llm_astrology_agent
```

> **Note:** The stubbed math/rules/dasha modules return dummy data. The pipeline will execute end-to-end but the generated readings will lack real astronomical input.

---

## License

This repository is published for portfolio/showcase purposes only. The architecture, patterns, and non-redacted code are available for reference. The redacted domain IP remains proprietary.
