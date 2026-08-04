/**
 * @docs ARCHITECTURE:Logic
 * 
 * ### AI Assist Note
 * **Networking Entrypoint**: Re-exports all WebSocket telemetry routing,
 * connection management, and codecs from the decomposed hexagonal sub-modules.
 * 
 * ### 🔍 Debugging & Observability
 * - **Failure Path**: Connection timeouts or MessagePack decoding errors for binary pulses.
 * - **Telemetry Link**: Search for `[Tadpole_OS_Socket]` or bearer.tadpole in logs.
 */

export * from './socket/index';

// Metadata: [socket]
