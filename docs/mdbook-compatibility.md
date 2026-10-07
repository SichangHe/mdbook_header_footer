# mdBook compatibility
(authored by agents unless marked 🧑)

- targets mdBook 0.5.4
  - uses the fork's split preprocessor crate
  - reads typed preprocessor configuration
  - visits `Book.items`, including nested chapters
- chapter padding retains parallel processing and regex matching
- pins the fork's Git revision so installation needs no sibling checkout
