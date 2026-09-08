# RUST-LLM-ASTROLOGY-AGENT

[![clippy](https://img.shields.io/badge/clippy-passing-success.svg)](https://github.com/your-username/rust-llm-astrology-agent/actions)
[![Rust](https://img.shields.io/badge/Rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **⚠️ Domain IP Redacted** — All proprietary astrological logic (master prompts, Dasha calculations, Yoga detection, and the Vedic rules engine) has been stripped from this repository. Function signatures and data structures are preserved as stubs to demonstrate the architecture. This repo is published as a **structural showcase only**.

---

## About

**A fault-tolerant, multi-agent LLM pipeline built in Rust, showcased via a Vedic Astrology reasoning engine.**

This project demonstrates a high-performance system designed to orchestrate complex, multi-step AI workflows against rate-limited APIs. While the system was originally deployed for production Vedic astrology readings, the core patterns and architecture generalize to any complex multi-agent domain requiring:

- **Multi-Agent Orchestration** — Agent 1 extracts structured parameters (temporal targets) from natural language; Agent 2 generates long-form analytical output conditioned on deterministic data with an inter-agent rate-limit cooldown.
- **Resilient API Communication** — Custom 3-tier dynamic backoff parsing `"retry in Xs"` directly from JSON error payloads, respecting `Retry-After` HTTP headers, with exponential jitter fallbacks up to configurable retry ceilings.
- **Connection Lifecycle Management & Shared Pooling** — Deliberate HTTP connection tuning (`pool_idle_timeout`, `pool_max_idle_per_host`, disabled TCP keepalive) and shared `reqwest::Client` connection pooling across geocoding and LLM requests.
- **Data Anonymization & Security** — Client PII is stripped from prompt payloads before API submission; geocoding query parameters are strictly URL-encoded using `.query(...)` builders.
- **Transactional Persistence & Utility Safety** — Atomic SQLite database transactions (`conn.transaction()`) for client and reading record lifecycles, non-destructive migrations, and filename sanitization (`sanitize_filename`).
- **Interactive Terminal & HTML Presentation** — Rich CLI/TUI experience built with `dialoguer` and `console`, with automatic HTML reading generation and cross-platform browser opening via `open`.

---

## Architecture & Engineering

The comprehensive architecture (including our 3-tier API backoff strategy, multi-agent pipeline, data anonymization layer, and database schema) is detailed in our documentation folder.

Please see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full details and Mermaid.js diagrams.

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (Edition 2024) |
| Async Runtime | Tokio |
| HTTP Client | Reqwest (with JSON, connection pooling & tuning) |
| LLM Provider | Google Gemini API (v1beta) |
| Database | SQLite via rusqlite (bundled with transaction safety) |
| Geocoding | Nominatim (OpenStreetMap with query encoding) |
| Timezone Resolution | `tzf-rs` (offline embedded TZ database) |
| Historical TZ Offsets | `chrono-tz` |
| TUI / CLI | `dialoguer` + `console` |
| File & Launch Utils | `open` (cross-platform) + custom filename sanitization |
| Serialization | `serde` + `serde_json` |

---

## Project Structure

```
src/
├── lib.rs        # Shared library module exports
├── main.rs       # Interactive CLI/TUI, orchestration, transactional DB layer, HTML presentation
├── api.rs        # Gemini API client, rate limit parsing & 3-tier backoff engine
├── math.rs       # [STUBBED] Planetary position & house calculation signatures
├── rules.rs      # [STUBBED] Vedic astrology rules engine & dignity evaluation signatures
├── dasha.rs      # [STUBBED] Vimshottari Dasha timeline generator signature
├── geo.rs        # Geocoding via Nominatim with connection pooling & offline timezone resolution
├── utils.rs      # String and filename sanitization utilities
└── bin/
    └── verify_db.rs # Standalone DB inspection utility
```

---

## Running & Verification

```bash
# Set your Gemini API key
export GEMINI_API_KEY="your-key-here"

# Run all unit and integration tests
cargo test

# Check code formatting & lints
cargo fmt --check
cargo clippy --all-targets

# Run the interactive application
cargo run --bin rust_llm_astrology_agent
```

> **Note:** The stubbed math/rules/dasha modules return structural dummy data. The pipeline executes end-to-end and outputs formatted HTML readings, but generated readings will reflect redacted domain logic.

---

## License

This repository is published for portfolio/showcase purposes only. The architecture, patterns, and non-redacted code are available for reference. The redacted domain IP remains proprietary.
