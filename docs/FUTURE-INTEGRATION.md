# Future Integration: lever-runner-carapace

## Current State
Native Rust performance layer for lever-runner — BLAKE2b hashing (128ns), position-aware embedding (1.73µs), cosine search, and the three-gate pipeline compiled to a native binary. Zero Python overhead for hot paths.

## Integration Opportunities

### With construct-core Layer 0/1
The carapace's native Rust with zero allocation (BLAKE2b in 128ns, embedding in 1.73µs) runs on construct-core's Layer 0 (BareMetalConstruct) and Layer 1 (SyncConstruct). The three-gate pipeline IS the construct's query mechanism: exact match at Layer 0, fuzzy match at Layer 1, fallback at Layer 2.

### With room-as-codespace
The carapace provides the room's command matching engine. When an ensign needs to respond to a room event, it runs through the carapace's three gates in <2µs — instant response, no LLM. This is the room's reflex layer.

### With lever-runner-wasm
The carapace compiles to both native binary (for Codespaces and Jetson) and WASM (for browser rooms). The same Rust codebase targets every hardware tier. The WASM build runs the three-gate pipeline in the browser with near-native speed.

## Dormant Ideas Now Unlockable
The carapace was optimized for shell commands. Now it optimizes for room queries. The same BLAKE2b hashing that matches "check disk usage" → "df -h" matches "sensor pattern 0x3F" → "trigger bearing alert". The optimization target changes; the engine stays the same.

## Potential in Mature Systems
The carapace becomes the fleet's universal query engine. Every room, every construct, every agent uses it for fast pattern matching. Sub-microsecond response times mean rooms can process millions of events per second without LLM calls.

## Cross-Pollination Ideas
- **tile-compiler**: Tile compilation uses carapace's BLAKE2b for state hashing
- **position-aware-embed**: Embedding algorithm shared between carapace and standalone crate
- **tile-neon**: ARM NEON optimization applies to carapace on Jetson

## Dependencies for Next Steps
- Compile to no_std for ESP32 deployment
- WASM build for browser rooms
- Room query DSL on top of three-gate pipeline
