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
- **Connection lifecycle management** — Deliberate connection tearing via `pool_idle_timeout`, `pool_max_idle_per_host`, and disabled TCP keepalive to survive long inter-request cooldowns on free-tier APIs.
- **Robust UTF-8 Safe Date Parsing** — Zero-panic multi-window date extraction (`extract_target_date_from_text`) with strict `YYYY-MM-DD` pattern verification, gracefully handling conversational LLM responses and multi-byte UTF-8 character boundaries.
- **Pure Timezone & Historical Offset Resolution** — Decoupled offline spatial timezone lookups (`tzf-rs`) and historical UTC offset resolution (`chrono-tz`) into testable pure functions (`resolve_timezone_and_offset`).
- **Standardized HTML Report Generation** — Centralized HTML report builder (`generate_html_report`) with character escaping sanitization (`escape_html`) for secure document generation and browser opening via `open`.
- **Zero-copy prompt pipelines & Data Anonymization** — Multi-stage prompt assembly with client PII anonymization before API submission.
- **Transactional SQLite Persistence & Robust Client Management** — Atomic client profile management (`manage_client`) and reading deletions using explicit SQLite transactions (`conn.transaction()`), centralized table/index initialization (`create_tables`), indexed lookup queries, and clean row mapping abstractions (`ClientRecord::from_row`).
- **Clean Type Abstractions & Refactored Wizard UX** — Structured location resolution outputs (`LocationData`), refactored interactive wizard parameters (`ReadingParams`), deduplicated prompt helpers (`prompt_target_words`), validated status selections, precise dependency pinning (`tzf-rs = "=1.2.0"`), robust filename sanitization (`sanitize_filename`), and test coverage across API, database, geocoding, utilities, and stubbed domain modules.

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
├── lib.rs        # Shared library exposing API, dasha, geo, math, rules, and utils
├── main.rs       # CLI/TUI, orchestration, DB layer, presentation, and DB tests
├── api.rs        # Gemini API client, retry logic, UTF-8 safe date parsing, and backoff engine
├── math.rs       # [STUBBED] Planetary position calculations and math engine tests
├── rules.rs      # [STUBBED] Vedic astrology rules engine and summary tests
├── dasha.rs      # [STUBBED] Vimshottari Dasha timeline generator and engine tests
├── geo.rs        # Geocoding, pure historical timezone resolution, and LocationData abstractions
├── utils.rs      # HTML report generation, filename sanitization, and escaping utilities with edge-case tests
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
