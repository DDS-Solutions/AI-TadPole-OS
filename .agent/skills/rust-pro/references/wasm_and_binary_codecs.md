> [!IMPORTANT]
> **AI Context & Knowledge Heritage**
> - **Subsystem**: Agent Skills Registry / rust-pro / references
> - **Architecture**: `@docs ARCHITECTURE:Documentation`
> - **Failure Path**: WASM memory buffer leaks, postcard deserialization panic, or build target mismatch.
> - **Observability**: Traceability via `execution/parity_guard.py` (`[RUST_PRO]`)

# WebAssembly & High-Performance Binary Codecs (L3)

This reference documents the WebAssembly build pipeline and binary serialization protocol for [`wasm-codec/`](file:///d:/TadpoleOS-Dev/wasm-codec/).

---

## 1. Subsystem Architecture

The `wasm-codec` crate compiles Rust into WebAssembly (`wasm32-unknown-unknown`) to provide near-native deserialization performance in the browser/Tauri frontend for large agent telemetry and vector payloads.

```
┌────────────────────────┐      Uint8Array      ┌────────────────────────┐
│     React Frontend     │ <==================> │       wasm-codec       │
│ (Worker / Webpack/Vite)│  postcard serialization (wasm-bindgen memory) │
└────────────────────────┘                      └────────────────────────┘
```

---

## 2. Compilation Invariants

1. **Target**: Always compile using `wasm-pack` with the `web` target:
   ```powershell
   wasm-pack build wasm-codec --target web --release
   ```
2. **Crate Type**: `Cargo.toml` must declare both `cdylib` and `rlib` in `[lib]`.
3. **Randomness Support**: In browser WebAssembly, standard OS randomness is unavailable. Always enable the `js` feature on `getrandom`:
   ```toml
   getrandom = { version = "0.2", features = ["js"] }
   ```

---

## 3. High-Speed Serialization with `postcard`

Instead of expensive JSON parsing, `wasm-codec` utilizes `postcard` (a compact binary format designed for embedded systems and WebAssembly):

```rust
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct TelemetryPacket {
    pub timestamp: u64,
    pub agent_id: String,
    pub status: u8,
}

#[wasm_bindgen]
pub fn decode_packet(bytes: &[u8]) -> Result<JsValue, JsError> {
    let packet: TelemetryPacket = postcard::from_bytes(bytes)
        .map_err(|e| JsError::new(&format!("Postcard decode error: {}", e)))?;
    
    serde_wasm_bindgen::to_value(&packet)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))
}
```

---

## 4. Frontend WebAssembly Consumption

In React or Web Workers:

```typescript
import init, { decode_packet } from '@/wasm/wasm_codec';

export async function processBinaryStream(buffer: ArrayBuffer) {
  await init();
  const uint8View = new Uint8Array(buffer);
  const decoded = decode_packet(uint8View);
  return decoded;
}
```

---

## 5. Non-Negotiables & Memory Safety

- **No Panics Across the FFI Boundary**: Rust panics inside WASM unwind into unhandled JavaScript exceptions and corrupt the WebAssembly linear memory. Always catch errors and return `Result<T, JsError>`.
- **Zero-Copy Views**: When passing large typed arrays to WASM, pass views (`&[u8]`) rather than copying raw vectors whenever possible.
