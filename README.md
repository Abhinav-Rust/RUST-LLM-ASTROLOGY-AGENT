# RUST-LLM-ASTROLOGY-AGENT

[![clippy](https://img.shields.io/badge/clippy-passing-success.svg)](https://github.com/your-username/rust-llm-astrology-agent/actions)
[![Rust](https://img.shields.io/badge/Rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **⚠️ Domain IP Redacted** — All proprietary astrological logic (master prompts, Dasha calculations, Yoga detection, and the Vedic rules engine) has been stripped from this repository. Function signatures and data structures are preserved as stubs to demonstrate the architecture. This repo is published as a **structural showcase only**.

---

## Overview

**A production-grade, fault-tolerant, multi-agent AI orchestration pipeline built in Rust, demonstrated via a Vedic Astrology reasoning engine.**

This codebase showcases a resilient system designed to execute multi-step LLM workflows against rate-limited AI APIs with maximum reliability. Originally built for production Vedic astrology readings, the core patterns and architecture generalize to any complex multi-agent domain requiring structured extraction, deterministic rules execution, prompt anonymization, persistent storage, and interactive presentation.

---

## Core Capabilities & Features

- 🤖 **Multi-Agent AI Pipeline**
  - **Agent 1 (Temporal Extractor)**: Extracts precise target date parameters from natural language user queries (`gemini-3.1-flash-lite`).
  - **Agent 2 (Synthesis Engine)**: Generates comprehensive analytical readings conditioned on deterministic chart summaries and active Dasha timelines.
- ⚡ **Resilient Network Architecture**
  - **3-Tier Backoff Engine**: Regex-parses error response bodies for retry wait hints (`retry in Xs`), respects HTTP `Retry-After` headers, and falls back to exponential backoff with jitter.
  - **Connection Lifecycle Management**: Aggressive connection recycling via `pool_idle_timeout(5s)` and disabled TCP keepalive to prevent connection reset errors over extended cooldowns.
  - **Nominatim Geocoding Retries**: Automatic exponential retry on geocoding requests with location & historical timezone resolution (`tzf-rs` + `chrono-tz`).
- 🛡️ **Data Privacy & Anonymization Layer**
  - Automatically replaces client PII (names, sensitive parameters) with anonymous identifiers before LLM submission.
- 📂 **SQLite Persistence & Client Management**
  - Manages client profiles, repeat customer detection, status tracking (Active/Refused), and full reading history in SQLite (`astrology_journal.db`).
- 🎨 **Rich Terminal & HTML Output**
  - Features an interactive menu system powered by `dialoguer` and `console`.
  - Automatically generates styled HTML reports in the `readings/` directory and launches them in the default system browser via cross-platform system calls.

---

## Tech Stack

| Component | Technology |
|-----------|-----------|
| **Language** | Rust (Edition 2024) |
| **Async Runtime** | Tokio |
| **HTTP Client** | Reqwest (custom connection tuning & JSON) |
| **LLM Provider** | Google Gemini API (`gemini-3.1-flash-lite`) |
| **Database** | SQLite via `rusqlite` (bundled) |
| **Geocoding** | Nominatim (OpenStreetMap API with retry strategy) |
| **Timezone Resolution** | `tzf-rs` (offline, embedded TZ spatial engine) |
| **Historical Offsets** | `chrono-tz` |
| **TUI / CLI** | `dialoguer` + `console` |
| **Serialization** | `serde` + `serde_json` |

---

## Architecture & Engineering Patterns

The detailed architecture (including the 3-tier API backoff strategy, multi-agent flow, and data anonymization layer) is documented with Mermaid diagrams in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Project Structure

```
.
├── Cargo.toml
├── README.md
├── docs/
│   └── ARCHITECTURE.md     # Architectural diagrams and deep-dive notes
└── src/
    ├── lib.rs              # Shared library exports
    ├── main.rs             # CLI/TUI application, DB layer, workflow orchestrator
    ├── api.rs              # Gemini API client, 3-tier backoff & retry engine
    ├── geo.rs              # Nominatim geocoding & tzf-rs historical TZ resolver
    ├── math.rs             # [STUBBED] Planetary position and house system stubs
    ├── rules.rs            # [STUBBED] Vedic rules engine & dignity evaluation stubs
    ├── dasha.rs            # [STUBBED] Vimshottari Dasha timeline engine stub
    ├── utils.rs            # Text sanitization and utility functions
    └── bin/
        └── verify_db.rs    # Standalone database inspection tool
```

---

## Getting Started

### Prerequisites

- Rust 1.80+ (2024 Edition)
- Google Gemini API Key

### Running the Application

Set your Gemini API key and run the main interactive CLI:

```bash
# Set your API Key
export GEMINI_API_KEY="your-gemini-api-key"

# Build and run
cargo run --bin rust_llm_astrology_agent
```

You can also pass arguments directly for non-interactive CLI executions:

```bash
cargo run --bin rust_llm_astrology_agent -- "Client Name" "15/08/1990" "10:45 AM" "London" "Will I move abroad next year?" 500
```

### Inspecting Database Storage

To inspect saved clients and archived readings in `astrology_journal.db`:

```bash
cargo run --bin verify_db
```

### Running Tests & Quality Checks

```bash
# Run unit tests
cargo test

# Run clippy for zero warnings
cargo clippy --all-targets

# Check code formatting
cargo fmt -- --check
```

---

## License

This repository is published for portfolio and showcase purposes only. The architecture, patterns, and non-redacted code are available for reference. The redacted domain IP remains proprietary.
