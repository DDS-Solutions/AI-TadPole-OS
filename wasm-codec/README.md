# wasm-codec

`wasm-codec` is a Rust crate that compiles to WebAssembly (Wasm) providing high-throughput binary serialization for `SwarmPulse` telemetry structures using the [Postcard](https://github.com/jamesmunns/postcard) binary format.

## Architecture & Purpose

- **Codec**: Postcard variable-length binary serialization.
- **Targets**: `PulseNode`, `PulseConnection`, and `SwarmPulse`.
- **Primary Functionality**: Sub-millisecond binary encoding and decoding for dense swarm graphs (60 FPS rendering target).

## Relationship to Production Socket Codec

In the active Tadpole OS frontend:
- **WebSocket Protocol**: Production telemetry streaming uses MessagePack via `@msgpack/msgpack` (see `src/services/socket/codec/protocol_codec.ts`).
- **`wasm-codec` Status**: Decoupled high-performance alternative/reference implementation for Wasm-based pulse processing. It is maintained and verified independently with Rust unit tests.

## Running Tests

```bash
cargo test --manifest-path wasm-codec/Cargo.toml
```
