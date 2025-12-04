# feedparser
The XML parser that converts saved podcast feeds into intermediary files for SQL ingestion.

This project is for the next-gen feed parser for the Podcast Index.  The parser has two jobs:  extract data from a podcast XML feed saved on the file system and write the channel and item data to individual files for them to be picked up later by the database ingester.

Input file format:  XML (RSS 2.0)
Output channel file format:  JSON 
Output item file format:  JSON

The code is Rust and uses a streaming XML parser in order to be as fast as it can.  Fast, parallel processing of input files is the goal.

This binary will be part of the larger aggregator process chain:
- Aggrivator (the feed polling agent)
- Feedparser (this project)
- SQL ingestor agent (runs on each aggregator)
- Queue server (accepts objects from the SQL ingestor agents)
- SQL execution agent (picks object off the queue server and puts them in the database)

# Note
This is a community coding project.  PR's are highly encouraged.  All contributors will be recognized here for their work.

# Contributors
- Dave Jones (gh: @daveajones)
