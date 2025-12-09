# Project Guidelines — feedparser

## Overview
This Rust project parses RSS-style feed input files and emits normalized JSON records. It is optimized for simple, event-driven processing and clean separation of concerns via per-tag handler modules.

Input files live in the `inputs` directory and use a 4-line header followed by the XML payload:

1. Unix timestamp of Last-Modified (or 0 if header is missing)
2. E-Tag header (or `[[NO_ETAG]]` if header is missing)
3. Source feed URL
4. Unix timestamp of when the XML was downloaded
5..end. The complete XML document

Outputs are written as individual JSON files into a timestamped run folder under `outputs/<startup_unix>/`. Each file name is `<counter>_<table>_<feed_id>.json` (with `NULL` if the id is unknown).

Two logical tables are emitted:
- newsfeeds: one record per `<channel>`
- nfitems: one record per `<item>`

## Project Structure
- Cargo.toml — crate metadata and dependencies
- src/
  - main.rs — entry point, input reading, XML dispatch loop, tests
  - parser_state.rs — shared parsing state and flags
  - outputs.rs — JSON record construction and file writes
  - tags/
    - mod.rs — dispatch functions that route XML events to tag handlers
    - channel.rs — `<channel>` lifecycle and newsfeeds write
    - item.rs — `<item>` lifecycle and nfitems write
    - title.rs — `<title>` text handling (channel/item aware)
    - link.rs — `<link>` text handling (channel/item aware)
    - description.rs — `<description>` text handling (channel/item aware)
    - pub_date.rs — `<pubDate>` text handling for items
    - image.rs — channel `<image>` scope tracking
    - itunes_image.rs — `itunes:image` attribute capture within items
    - podcast_funding.rs — `podcast:funding` url/text capture within items
- inputs/ — place source files to be parsed
- outputs/ — generated per-run subfolders with JSON outputs
- sample_inputs.tar.gz, sample_outputs.tar.gz — compressed example files (extract with `tar -xzf`)
- .llm_history/ — session logs from LLM-assisted development (format: `YYYYMMDD_N.md`)

## Build, Run, Test
- Build:
  - PowerShell (Windows):
    - `cargo build`

- Run:
  - Ensure `inputs` contains files in the expected header+XML format.
  - `cargo run`
  - Results are written under `outputs\<startup_unix>\`.

- Tests:
  - `cargo test`
  - Tests run in-process and use a global `OnceLock` (`OUTPUT_SUBDIR`) to isolate outputs to a temp folder. Avoid setting `OUTPUT_SUBDIR` more than once per process; follow the existing test pattern in `src\main.rs` (`ensure_output_dir()` helper) for additional tests.
  - Run a specific test: `cargo test writes_channel_title_to_newsfeeds_output -- --nocapture`

## JSON Schemas
- newsfeeds
  - columns: `["feed_id", "title", "link", "description", "generator"]`
  - values: `[feed_id or null, channel_title.trim(), channel_link.trim(), channel_description.trim(), channel_generator.trim()]`

- nfitems
  - columns: `["feed_id", "title", "link", "description", "pub_date", "itunes_image", "podcast_funding_url", "podcast_funding_text"]`
  - values: `[feed_id or null, title, link, description.trim(), pub_date, itunes_image, podcast_funding_url, podcast_funding_text.trim()]`

Note: CDATA sections are treated like normal text; both `Characters` and `CData` XML events are accumulated.

## Adding/Modifying Tag Handlers
To add support for a new XML element:
1. Create a new module under `src\tags\`, e.g. `author.rs`.
2. Implement functions such as `on_start`, `on_text`, and/or `on_end` that operate on `ParserState`.
3. Register the new module in `src\tags\mod.rs`:
   - `pub mod author;`
   - Update `dispatch_start`, `dispatch_text`, and/or `dispatch_end` match arms to call it.
4. If the tag contributes to output, ensure the correct place writes it:
   - Channel-level fields via `outputs::write_newsfeeds`.
   - Item-level fields via `outputs::write_nfitems`.
5. Add unit tests demonstrating the new behavior, following the existing test style in `src\main.rs`.

## Code Style and Conventions
- Use Rust 2021 edition defaults; run `rustfmt` (via `cargo fmt`) for formatting.
- Follow existing naming and module patterns; keep handlers small and tag-focused.
- Avoid global mutable state beyond the provided `GLOBAL_COUNTER` and `OUTPUT_SUBDIR`.
- Prefer trimming of channel-level strings at write time; item strings are kept as-is except where otherwise specified.
  - Update: item `description` is now trimmed; `podcast_funding_text` remains trimmed.

## Performance Notes
- The async `main` function enumerates input files and executes parsing on a blocking thread via `tokio::task::spawn_blocking`.
- Parsing is streaming/event-based using `xml-rs` and accumulates text only for relevant tags.

## When coding with LLM's
- AI agents should write an overview of all changes made, including the prompts given which triggered the changes, to a new markdown file in the .llm_history directory.
- AI agents should modify the .junie/guidelines.md file to reflect any changes made to the project.


## Submission Checklist for Changes
- If code was modified:
  - Build succeeds: `cargo build`.
  - Relevant tests updated/added; `cargo test` passes.
  - Manually spot-check by placing a small input into `inputs` and running `cargo run` if applicable.
- If documentation-only changes: no build/test required unless noted.
