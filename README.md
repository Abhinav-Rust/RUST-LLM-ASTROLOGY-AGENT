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
- **Configurable Gemini Model Selection** — Dynamic model resolution via `get_gemini_model()` checking `GEMINI_MODEL` environment variables (defaulting to `"gemini-3.1-flash-lite"`), allowing seamless model switches (e.g. `gemini-1.5-pro` or `gemini-2.0-flash`) without code modification.
- **Resilient API communication** — Custom exponential backoff with dynamic rate-limit parsing directly from error message bodies, `Retry-After` header respect, and configurable retry ceilings.
- **Connection lifecycle management** — Deliberate connection tearing via `pool_idle_timeout`, `pool_max_idle_per_host`, and disabled TCP keepalive to survive long inter-request cooldowns on free-tier APIs.
- **Robust UTF-8 Safe & Flexible Date/Time Parsing** — Zero-panic multi-window date extraction (`extract_target_date_from_text`) with strict `YYYY-MM-DD` pattern verification and flexible date/time parser utilities (`parse_flexible_date`, `parse_flexible_time`) supporting `DD/MM/YYYY`, `YYYY-MM-DD`, `DD-MM-YYYY`, `MM/DD/YYYY`, 12-hour AM/PM, and 24-hour time inputs.
- **Pure Timezone & Historical Offset Resolution** — Decoupled offline spatial timezone lookups (`tzf-rs`) and historical UTC offset resolution (`chrono-tz`) into testable pure functions (`resolve_timezone_and_offset`).
- **Interactive Client Reading History Inspection** — Complete TUI reading history browser (`view_client_reading_history`) backed by `get_client_readings` and structured `ReadingRecord` row mappings, allowing instant chronological review of past AI readings for any registered client.
- **Print-Optimized & Structured HTML Report Generation** — Centralized HTML report builder (`generate_html_report`) featuring structured markdown heading/paragraph parsing (`format_reading_html`), character escaping sanitization (`escape_html`), modern CSS card containers, one-click clipboard copying, `@media print` layout styles, mobile breakpoint queries, and resilient auto-launching via `open` with explicit error warning handling.
- **Zero-copy prompt pipelines & Data Anonymization** — Multi-stage prompt assembly with client PII anonymization before API submission.
- **Transactional SQLite Persistence & Dynamic Schema Migrations** — Atomic client profile management (`manage_client`) updating city, birth data, DOB, and time for repeat clients alongside reading deletions using explicit SQLite transactions (`conn.transaction()`), dynamic column migrations via `PRAGMA table_info` (`ensure_client_columns`), centralized table/index initialization (`create_tables`), indexed lookup queries, and clean row mapping abstractions (`ClientRecord::from_row`, `ReadingRecord::from_row`).
- **Zero-Panic TUI Input Safety** — Safe input prompt wrappers (`prompt_input`, `prompt_select`, `prompt_confirm`) handling user cancellations, interrupts, and EOF gracefully without panicking the interactive menu loop.
- **API Key Validation & Resilient Execution** — Early env validation for `GEMINI_API_KEY` returning structured `GeminiError::ServerError` before making HTTP calls.
- **Enhanced Database Verification Utility** — Modernized standalone DB inspector (`verify_db`) with structured `ClientView` and `ReadingView` row mapping displaying summary client/reading counts, formatted tabular client profiles, and chronological reading history logs using resilient `LEFT JOIN` and `COALESCE` query logic.
- **Clean Type Abstractions & Fully Tested Suite** — Structured location resolution outputs (`LocationData`), refactored interactive wizard parameters (`ReadingParams`), deduplicated prompt helpers (`prompt_target_words`), validated status selections, precise dependency pinning (`tzf-rs = "=1.2.0"`), robust filename sanitization (`sanitize_filename`), and comprehensive test coverage across API, database, geocoding, utilities, and stubbed domain modules.

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
| LLM Provider | Google Gemini API (v1beta, configurable via `GEMINI_MODEL`) |
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
├── main.rs       # CLI/TUI, orchestration, DB layer, reading history viewer, and DB tests
├── api.rs        # Gemini API client, configurable model selection, retry logic, and backoff engine
├── math.rs       # [STUBBED] Planetary position calculations and math engine tests
├── rules.rs      # [STUBBED] Vedic astrology rules engine and summary tests
├── dasha.rs      # [STUBBED] Vimshottari Dasha timeline generator and engine tests
├── geo.rs        # Geocoding, pure historical timezone resolution, and LocationData abstractions
├── utils.rs      # Structured HTML report generation, paragraph formatting, copy button, and escaping utilities
└── bin/
    └── verify_db.rs # Standalone DB inspection utility with structured row mapping
```

---

## Running

```bash
# Set your Gemini API key (and optionally override model)
export GEMINI_API_KEY="your-key-here"
export GEMINI_MODEL="gemini-3.1-flash-lite" # Optional, defaults to gemini-3.1-flash-lite

# Run tests
cargo test

# Build and run interactive wizard & TUI menu
cargo run --bin rust_llm_astrology_agent

# Run database verification utility
cargo run --bin verify_db
```

> **Note:** The stubbed math/rules/dasha modules return dummy data. The pipeline will execute end-to-end but the generated readings will lack real astronomical input.

---

## License

This repository is published for portfolio/showcase purposes only. The architecture, patterns, and non-redacted code are available for reference. The redacted domain IP remains proprietary.
