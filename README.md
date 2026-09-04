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
- **Resilient API communication** — Custom 3-tier exponential backoff with dynamic rate-limit parsing directly from error message bodies (`"retry in"`, `"retry after"`, `"try again in"`), `Retry-After` header respect, transient HTTP 5xx retry handling, and configurable retry ceilings.
- **Type-safe Geocoding & Query Encoding** — Structured `LocationData` model paired with URL query-string parameter encoding for safe handling of international location queries.
- **Connection lifecycle management** — Deliberate connection tearing via `pool_idle_timeout`, `pool_max_idle_per_host`, and disabled TCP keepalive to survive long inter-request cooldowns on free-tier APIs.
- **Zero-copy prompt pipelines** — Multi-stage prompt assembly with data anonymization layers before API submission.
- **Robust testing & persistence** — In-memory SQLite integration tests, client updates & row-affected validations, comprehensive API backoff verification, and unit test coverage across all codebase modules.

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
| HTTP Client | Reqwest (with JSON, connection tuning) |
| LLM Provider | Google Gemini API (v1beta) |
| Database | SQLite via rusqlite (bundled) |
| Geocoding | Nominatim (OpenStreetMap) |
| Timezone Resolution | `tzf-rs` (offline, embedded TZ database) |
| Historical TZ Offsets | `chrono-tz` |
| TUI | `dialoguer` + `console` |
| Serialization | `serde` + `serde_json` |

---

## Project Structure

```
src/
├── lib.rs        # Shared library for all modules
├── main.rs       # CLI/TUI, orchestration, SQLite DB persistence & client management
├── api.rs        # Gemini API client with 3-tier backoff & 5xx server error retry
├── math.rs       # [STUBBED] Planetary position calculations
├── rules.rs      # [STUBBED] Vedic astrology rules engine
├── dasha.rs      # [STUBBED] Vimshottari Dasha timeline generator
├── geo.rs        # Geocoding (`LocationData`), query encoding & historical timezone resolution
├── utils.rs      # Filename sanitization and string utilities
└── bin/
    └── verify_db.rs # Standalone DB inspection utility
```

---

## Running

```bash
# Set your Gemini API key
export GEMINI_API_KEY="your-key-here"

# Run tests
cargo test

# Build and run
cargo run --bin rust_llm_astrology_agent
```

> **Note:** The stubbed math/rules/dasha modules return dummy data. The pipeline will execute end-to-end but the generated readings will lack real astronomical input.

---

## License

This repository is published for portfolio/showcase purposes only. The architecture, patterns, and non-redacted code are available for reference. The redacted domain IP remains proprietary.
