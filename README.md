# RUST-LLM-ASTROLOGY-AGENT

[![clippy](https://img.shields.io/badge/clippy-passing-success.svg)](https://github.com/your-username/rust-llm-astrology-agent/actions)
[![Rust](https://img.shields.io/badge/Rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **⚠️ Domain IP Redacted** — All proprietary astrological logic (master prompts, Dasha calculations, Yoga detection, and the Vedic rules engine) has been stripped from this repository. Function signatures and data structures are preserved as stubs to demonstrate the architecture. This repo is published as a **structural showcase only**.

---

## About

**A fault-tolerant, multi-agent LLM pipeline built in Rust, showcased via a Vedic Astrology reasoning engine.**

This project demonstrates a high-performance system designed to orchestrate complex, multi-step AI workflows against rate-limited APIs. While the system was originally deployed for production Vedic astrology readings, the core patterns and architecture generalize to any complex multi-agent domain requiring:

- **Multi-Agent Orchestration** — Agent 1 extracts structured temporal parameters from natural language queries; Agent 2 generates long-form analytical output conditioned on deterministic chart data.
- **Resilient API Communication** — Custom exponential backoff with dynamic rate-limit parsing directly from Gemini JSON error payloads (`retry in Xs`), `Retry-After` HTTP header respect, and configurable retry ceilings.
- **Connection Lifecycle Management** — Deliberate connection tearing via `pool_idle_timeout(5s)`, `pool_max_idle_per_host(1)`, and disabled TCP keepalive to survive long inter-request cooldowns on free-tier APIs.
- **Robust UTF-8 Safe & Flexible Date/Time Parsing** — Zero-panic multi-window date extraction (`extract_target_date_from_text`) with strict `YYYY-MM-DD` pattern verification and flexible parser utilities (`parse_flexible_date`, `parse_flexible_time`) supporting `DD/MM/YYYY`, `YYYY-MM-DD`, `DD-MM-YYYY`, `MM/DD/YYYY`, 12-hour AM/PM, and 24-hour time inputs.
- **Pure Timezone & Historical Offset Resolution** — Decoupled offline spatial timezone lookups (`tzf-rs`) and historical UTC offset resolution (`chrono-tz`) into testable pure functions (`resolve_timezone_and_offset`).
- **Enhanced Responsive & Print-Ready HTML Reports** — Centralized HTML report builder (`generate_html_report`) featuring HTML character sanitization (`escape_html`), responsive card layouts, dark header gradients, mobile breakpoints, generation timestamp metadata, print media query styles (`@media print`), and auto-opening with explicit error logging.
- **Zero-Copy Prompt Pipelines & Data Anonymization** — Multi-stage prompt assembly with client PII anonymization before API submission.
- **Transactional SQLite Persistence & Dynamic Schema Migrations** — Atomic client profile management (`manage_client`), full TUI editing (`edit_client`) for Name, City, DOB, Time of Birth, and Status with interactive format validation, reading deletions in single transactions (`conn.transaction()`), dynamic column migrations via `PRAGMA table_info` (`ensure_client_columns`), centralized table/index initialization (`create_tables`), indexed lookup queries, and clean row mapping abstractions (`ClientRecord::from_row`).
- **Harden DB Verification Utility** — Modernized standalone DB inspector (`verify_db`) with `LEFT JOIN` and `COALESCE` mapping to display summary client/reading counts, formatted tabular client profiles, and chronological reading history logs.
- **Clean Type Abstractions & Full Test Suite** — Structured location resolution outputs (`LocationData`), interactive wizard parameters (`ReadingParams`), deduplicated prompt helpers (`prompt_target_words`), precise dependency pinning (`tzf-rs = "=1.2.0"`), robust filename sanitization (`sanitize_filename`), and test coverage across API, database, geocoding, utilities, and stubbed domain modules.

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
| Cross-Platform Launch | `open` crate |
| Serialization | `serde` + `serde_json` |

---

## Project Structure

```
src/
├── lib.rs        # Shared library exposing API, dasha, geo, math, rules, and utils
├── main.rs       # CLI/TUI, orchestration, client management, DB layer, report generation, and tests
├── api.rs        # Gemini API client, retry logic, UTF-8 safe date parsing, and backoff engine
├── math.rs       # [STUBBED] Planetary position calculations and math engine tests
├── rules.rs      # [STUBBED] Vedic astrology rules engine and summary tests
├── dasha.rs      # [STUBBED] Vimshottari Dasha timeline generator and engine tests
├── geo.rs        # Geocoding, pure historical timezone resolution, and LocationData abstractions
├── utils.rs      # HTML report generation, print styles, filename sanitization, and escaping utilities with tests
└── bin/
    └── verify_db.rs # Standalone DB inspection utility with LEFT JOIN row mapping
```

---

## Running

```bash
# Set your Gemini API key
export GEMINI_API_KEY="your-key-here"

# Run all workspace unit tests
cargo test

# Build and run interactive wizard & client management TUI
cargo run --bin rust_llm_astrology_agent

# Run database verification utility
cargo run --bin verify_db
```

> **Note:** The stubbed math/rules/dasha modules return dummy data. The pipeline will execute end-to-end but the generated readings will lack real astronomical input.

---

## License

This repository is published for portfolio/showcase purposes only. The architecture, patterns, and non-redacted code are available for reference. The redacted domain IP remains proprietary.
