# feedparser
The XML parser that converts saved podcast feeds into intermediary files for SQL ingestion.  This is a community coding project.  PR's are highly encouraged.  All contributors will be recognized here for their work.

This project is for the next-gen feed parser for the Podcast Index.  The parser has two jobs:  extract data from a podcast XML feed saved on the file system and write the channel and item data to individual files for them to be picked up later by the database ingester.

- Input file format:  XML (RSS 2.0) with 4 header lines from [Aggrivator](https://github.com/Podcastindex-org/aggrivator)
- Output channel file format:  JSON encoded representation of SQL INSERT data in the newsfeeds table
- Output item file format:  JSON encoded representation of SQL INSERT data in the nfitems table

The code is Rust and uses a streaming XML parser in order to be as fast as it can.  Fast, parallel processing of input files is the goal.

This binary will be part of the larger aggregator process chain:
- [Aggrivator](https://github.com/Podcastindex-org/aggrivator) (the feed polling agent)
- Feedparser (this project)
- SQL statement builder (runs on each aggregator) - to be built
- Queue server (accepts objects from the SQL ingestor agents) - to be built
- SQL execution agent (picks object off the queue server and puts them in the database) - to be built

## Input file format
The input file format is a simple 4-line header followed by the XML payload.  The header lines are:
1. Unix timestamp of Last-Modified (or 0 if not available)
2. ETag header (or [[NO_ETAG]])
3. XML feed URL
4. Unix timestamp of when the XML was downloaded by Aggrivator

## Output channel file format
The output channel file format is a JSON object with the following fields:
- feed_id:  the feed_id from the input file name pattern (e.g. [feed id]_[http response code].txt)
- title:  the channel title
- link:  the channel link
- description:  the channel description

## Output item file format
The output item file format is a JSON object with the following fields:
- feed_id:  the feed_id from the input file name pattern (e.g. [feed id]_[http response code].txt)
- title:  the item title
- link:  the item link
- description:  the item description
- pub_date:  the item pub date (ISO 8601 format)
- itunes_image:  the item itunes:image URL (if available)
- podcast_funding_url:  the item podcast:funding URL (if available)
- podcast_funding_text:  the item podcast:funding text (if available)

## Sample data
Sample input and output files are available as compressed archives: `sample_inputs.tar.gz` and `sample_outputs.tar.gz`.

To extract the samples for testing:
```bash
tar -xzf sample_inputs.tar.gz
tar -xzf sample_outputs.tar.gz
```

The files from the `sample_inputs` directory can be moved into the [inputs](inputs) directory to be processed for testing.

## Development Setup

### Using Nix Flake (Recommended for NixOS)
This project includes a Nix flake for reproducible development environments:

```bash
# Enter the development shell
nix develop

# Or use direnv for automatic environment loading
direnv allow

# Build the project
nix build

# Run the project
nix run
```

The flake provides:
- Rust stable toolchain with rust-analyzer
- All necessary build dependencies
- Reproducible development environment

### Traditional Setup
Ensure you have Rust and Cargo installed, then:

```bash
# Build the project
cargo build

# Run the project
cargo run

# Run tests
cargo test
```
## AI/LLM
This project is being developed by the Podcasting 2.0 community.  We are using AI/LLM's to assist with project management, coding and code review.
The history of any LLM operations on this project can be found in the [.llm_history](.llm_history) folder.  Agent readable guidelines are in the
[.junie/guidelines.md](.junie/guidelines.md) file.

# Contributors
- Dave Jones (gh: [@daveajones](https://github.com/daveajones))
