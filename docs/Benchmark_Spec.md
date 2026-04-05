# 🧪 Tadpole Engine: Benchmark Test Specification
**Intelligence Level**: High (ECC Optimized)
**Source of Truth**: `server-rs/src/agent/benchmarks.rs`, `server-rs/benches/`
**Last Hardened**: 2026-04-01
**Standard Compliance**: ECC-PERF (Enhanced Contextual Clarity - Performance Standards)

> [!IMPORTANT]
> **AI Assist Note (Performance Logic)**:
> This specification defines the technical success criteria for the Tadpole Engine.
> - **Primary Claims**: Parallel Swarming (70% latency reduction), O(1) DB Saves, Zero-Overhead Rate Limiting (<1ms).
> - **Execution**: Benchmarks are triggered via `POST /v1/benchmarks/run/:test_id` or `cargo bench`.
> - **Persistence**: Results are stored in the `benchmarks` SQLite table for historical delta analysis.
> - **Environment**: Always run in `--release` mode. SUPPRESS LOGS (`RUST_LOG=error`) to avoid I/O bottlenecks during measurement.

---

## 🧪 Benchmark Execution Loop

```mermaid
graph TD
    Trigger["Trigger (UI/CLI)"]
    Setup["Clean DB + Env Setup"]
    Run["Execute Test (Criterion/k6)"]
    Analyze["Delta Analysis (vs Baseline)"]
    Persist["Save to 'benchmarks' table"]
    UI["Update Analytics Dashboard"]

    Trigger --> Setup
    Setup --> Run
    Run --> Analyze
    Analyze --> Persist
    Persist --> UI
```

---

# 🧪 Tadpole Engine: Benchmark Test Specification


> **Status**: Stable  
> **Version**: 1.3.0  
> **Last Updated**: 2026-04-04  
> **Classification**: Sovereign  

---

## 📚 Table of Contents

- [1. Overview & Goals](#1-overview--goals)
- [2. Global Environment Setup](#2-global-environment-setup)
- [3. Tooling Installation](#3-tooling-installation)
- [4. Benchmark Categories](#4-benchmark-categories)
  - [BM-RUN — Agent Runner](#bm-run--agent-runner-runnerrs)
  - [BM-RL — Rate Limiter](#bm-rl--rate-limiter-rate_limiterrs)
  - [BM-DB — Persistence Layer](#bm-db--persistence-layer-persistencers)
  - [BM-MCP — MCP Host](#bm-mcp--mcp-host-mcprs)
  - [BM-WS — WebSocket & Event Bus](#bm-ws--websocket--event-bus-wsrs)
  - [BM-FS — FilesystemAdapter & Security](#bm-fs--filesystemadapter--security-filesystemrs)
  - [BM-HTTP — HTTP Client & Provider](#bm-http--http-client--provider-adapters)
  - [BM-FE — Frontend Performance](#bm-fe--frontend-performance)
  - [BM-SEC — Security & Integrity](#bm-sec--security--integrity-securityrs-auditrs-monitoringrs)
- [5. Execution Plan](#execution-plan-5-weeks)
- [6. Regression Policy](#regression-policy)

---


### 1. Overview & Goals
The Tadpole Engine makes several specific architectural performance claims. This spec exists to verify those claims with reproducible, recorded benchmarks during development — before they become assumptions in production. All results are persisted to the **Benchmark Analytics Hub** for historical comparison and delta analysis.

> [!TIP]
> **Interactive Triggering**: While all benchmarks can be run via CLI (`cargo bench`), the primary interface for triggering and visualizing performance metrics is the **Performance Analytics** dashboard in the Tadpole OS UI. From there, users can trigger specific tests (e.g., `BM-RUN-01`) with a single click — which hits the `/v1/benchmarks/run/:test_id` endpoint (Default Port: 8000) — and view immediate delta analysis against baseline runs.

Goal | Claimed Behavior | Pass Target
--- | --- | ---
Parallel swarming | `FuturesUnordered` reduces swarm latency | ≥ 70% reduction vs sequential
O(1) DB saves | `join_all()` parallelism in `save_agents()` | < 2x wall-clock time: 50 agents vs 5
Rate limiter no-op | Zero overhead when rpm/tpm not configured | < 1ms per call overhead
Swarm Pulse (10Hz) | Binary MessagePack maintains sub-ms state parity | < 1ms aggregation latency
WebSocket throughput | RAF batching + Binary Pulse maintains 60fps | 60fps at ≥ 500 events/sec
Sandbox containment | `canonicalize` blocks all escape attempts | 0 escapes across all attack vectors

2. Global Environment Setup
These steps must be completed once before any benchmark category is run. Record all output into bench_metadata.txt alongside your results.
Step 1 — Record System Info
bashecho "=== Bench Metadata ===" > bench_metadata.txt
rustc --version >> bench_metadata.txt
cargo --version >> bench_metadata.txt
uname -a >> bench_metadata.txt
lscpu | grep "Model name" >> bench_metadata.txt
free -h >> bench_metadata.txt
date >> bench_metadata.txt
Step 2 — Set CPU to Performance Mode (Linux)
bash# Disable frequency scaling for consistent results
sudo cpupower frequency-set -g performance

# Disable Intel turbo boost
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Verify
cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
# Expected output: performance
Step 3 — Build in Release Mode
bash# Always benchmark release builds — never debug
cd server-rs
cargo build --release

# Verify binary exists
ls -lh target/release/tadpole-engine
Step 4 — Prepare a Clean Bench Database
bash# Run before EVERY benchmark session
rm -f bench_test.db
export DATABASE_URL="sqlite://$(pwd)/bench_test.db"
sqlx migrate run

# Confirm DB is empty and migrations applied
sqlx database verify
Step 5 — Set Required Environment Variables
bashexport NEURAL_TOKEN="tadpole-dev-token-2026"
export DATABASE_URL="sqlite://$(pwd)/bench_test.db"
export LEGACY_JSON_BACKUP=false
export RUST_LOG=error           # suppress logs during bench — critical
export TOKIO_WORKER_THREADS=4   # pin thread count for reproducibility
export MCP_CACHE_ENABLED=true
export MCP_SUBPROCESS_TIMEOUT_SECS=60
Step 6 — Close Non-Essential Processes
Before benchmarking, ensure no background processes are competing for CPU/IO:
bash# Check what's using CPU
top -bn1 | head -20

# Kill any dev servers, hot-reloaders, or file watchers
pkill -f "vite"
pkill -f "cargo watch"

3. Tooling Installation
Install all required tools once before starting. Verify each is accessible before proceeding.
Rust Benchmarking (criterion)
toml# Add to server-rs/Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["async_tokio"] }
tokio = { version = "1", features = ["full"] }

[profile.bench]
opt-level = 3
lto = "thin"
codegen-units = 1
bash# Verify criterion is available
cargo bench --help
HTTP Load Testing (k6)
bash# macOS
brew install k6

# Ubuntu/Debian
sudo apt install k6

# Verify
k6 version
WebSocket Stress Testing (websocat)
bash# macOS
brew install websocat

# Linux (download binary)
wget https://github.com/vi/websocat/releases/latest/download/websocat.x86_64-unknown-linux-musl
chmod +x websocat.x86_64-unknown-linux-musl
sudo mv websocat.x86_64-unknown-linux-musl /usr/local/bin/websocat

# Verify
websocat --version
Frontend Tools
bash# Lighthouse CLI
npm install -g lighthouse

# React DevTools (Chrome extension — install manually)
# https://chrome.google.com/webstore/detail/react-developer-tools

# Verify Lighthouse
lighthouse --version
sqlx CLI (for DB setup)
bashcargo install sqlx-cli --no-default-features --features sqlite

# Verify
sqlx --version

4. Benchmark Categories

BM-RUN — Agent Runner (runner.rs)
Purpose: Verify the execution engine's performance under concurrent mission load and validate the parallel swarming latency claim.
Setup Parameters
rust// server-rs/benches/runner_bench.rs

use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;

criterion_group!(
    name = runner_benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets =
        bench_single_agent_baseline,
        bench_concurrent_agents,
        bench_parallel_vs_sequential_swarm,
        bench_subagent_depth_curve,
        bench_neural_handoff_overhead
);

criterion_main!(runner_benches);
envTOKIO_WORKER_THREADS=4
RUST_LOG=error
NEURAL_TOKEN=bench-token-local
Tests & Step-by-Step Execution
BM-RUN-01 — Single agent, single tool call baseline
bash# Step 1: Ensure clean DB
rm -f bench_test.db && sqlx migrate run

# Step 2: Run isolated benchmark
cargo bench --bench runner_bench -- bench_single_agent_baseline

# Step 3: Record mean latency from Criterion output
# Expected output location: target/criterion/bench_single_agent_baseline/
BM-RUN-02 — 10 concurrent agents, independent missions
rust// Parameters
concurrent_agents = 10
mission_type = "single_tool_call"   // identical missions, isolated contexts
measure = "wall_clock_total_ms"
bashcargo bench --bench runner_bench -- bench_concurrent_agents
BM-RUN-03 — Parallel (FuturesUnordered) vs sequential swarming ⭐ Key claim
rust// Parameters
tool_call_count = 5         // minimum to trigger meaningful difference
parallel_strategy = "FuturesUnordered"
sequential_strategy = "await each in loop"
runs = 50
measure = "wall_clock_ms per full swarm completion"
bash# Step 1: Run parallel variant
cargo bench --bench runner_bench -- bench_parallel_swarm

# Step 2: Run sequential variant
cargo bench --bench runner_bench -- bench_sequential_swarm

# Step 3: Compare means
# Pass condition: parallel mean ≤ 30% of sequential mean (≥70% reduction)
BM-RUN-04 — Sub-agent recruitment depth curve (1→5 levels)
rust// Parameters
depth_levels = [1, 2, 3, 4, 5]     // 5 is the enforced max
lineage_check = true                 // must remain enabled — never bypass
measure = "total_latency_ms per depth level"
bashcargo bench --bench runner_bench -- bench_subagent_depth_curve
# Plot output: latency should scale roughly linearly, not exponentially
BM-RUN-05 — Neural Handoff (SEC-04) overhead per spawn
rust// Parameters
handoff_enabled = true      // inject parent strategic thoughts
handoff_disabled = false    // baseline spawn without injection
prompt_size_tokens = 200    // representative strategic thought size
measure = "overhead_ms = handoff_enabled_mean - handoff_disabled_mean"
bashcargo bench --bench runner_bench -- bench_neural_handoff_overhead

BM-RL — Rate Limiter (rate_limiter.rs)
Purpose: Confirm the dual RPM/TPM limiter is both correct and imposes zero overhead in no-op mode.
Setup Parameters
rust// server-rs/benches/rate_limiter_bench.rs

// BM-RL-01: No-op config
let no_op_config = ModelConfig { rpm: None, tpm: None, ..Default::default() };

// BM-RL-02: RPM sliding window
let rpm_config = ModelConfig { rpm: Some(10), tpm: None, ..Default::default() };

// BM-RL-03: TPM fixed window
let tpm_config = ModelConfig { rpm: None, tpm: Some(50_000), ..Default::default() };

// BM-RL-04: Concurrent contention
let contention_config = ModelConfig { rpm: Some(10), tpm: None, ..Default::default() };
let concurrent_tasks: usize = 50;

// Shared constants
const TPM_WINDOW_SECONDS: u64 = 60;
const RPM_SEMAPHORE_HOLD_SECONDS: u64 = 60;
const WARMUP_REQUESTS: usize = 10;
const BENCH_ITERATIONS: usize = 100;
const REPEAT_RUNS: usize = 5;   // take median
Tests & Step-by-Step Execution
BM-RL-01 — No-op overhead
bash# Step 1: Run with no-op config
cargo bench --bench rate_limiter_bench -- bench_noop_overhead

# Step 2: Verify mean < 1ms
# If > 1ms: check for accidental mutex acquisition in the no-op path
BM-RL-02 — RPM sliding window accuracy
bash# Step 1: Configure RPM=10, fire 100 requests in a burst
cargo bench --bench rate_limiter_bench -- bench_rpm_sliding_window

# Step 2: Verify that requests 11-100 were blocked/queued
# Step 3: Verify no more than 10 requests completed in any 60s window
# Check logs: RUST_LOG=debug cargo bench ... to see semaphore acquire/release
BM-RL-03 — TPM fixed window park/resume
bash# Step 1: Set TPM=50_000, send requests totalling 55,000 tokens
cargo bench --bench rate_limiter_bench -- bench_tpm_fixed_window

# Step 2: Confirm task was "parked" via tokio::time::sleep
# Step 3: Confirm task resumed after window reset (60s boundary)
# Step 4: Record: time_parked_ms, time_resumed_ms, accuracy_delta_tokens
BM-RL-04 — Concurrent permit contention
bash# Step 1: Spawn 50 tasks simultaneously against RPM=10 limiter
cargo bench --bench rate_limiter_bench -- bench_concurrent_contention

# Step 2: Verify no panics or deadlocks
# Step 3: Confirm only 10 tasks acquired permits in first window
# Step 4: Record max queue depth and total completion time
BM-RL-05 — Post-hoc correction accuracy
bash# Step 1: Run 20 requests with known token counts
# Step 2: After each, call record_usage() with provider's actual usage
cargo bench --bench rate_limiter_bench -- bench_posthoc_correction

# Step 3: Compare estimated vs actual counter
# Pass condition: delta < 5% across all 20 requests

BM-DB — Persistence Layer (persistence.rs)
Purpose: Validate O(1) wall-clock save time and SQLite throughput under concurrent load.
Setup Parameters
bash# Fresh DB before every DB benchmark run
rm -f bench_test.db
export DATABASE_URL="$(pwd)/bench_test.db"
sqlx migrate run

# SQLite WAL mode (enable for concurrent write tests)
sqlite3 bench_test.db "PRAGMA journal_mode=WAL;"
sqlite3 bench_test.db "PRAGMA synchronous=NORMAL;"

> [!NOTE]
> **Schema Isolation**: The persistence stress benchmarks (BM-DB-01) write to the `benchmark_logs` table (added in migration `20260304000200`). This ensures stress tests do not clutter core agent or mission history tables.
rust// sqlx pool config
max_connections = 10
acquire_timeout = Duration::from_secs(5)

// Agent count ladder (BM-DB-01)
agent_counts = [1, 5, 10, 25, 50]

// BM-DB-02 mission CRUD
concurrent_writers = 20
inserts_per_writer = 100

// BM-DB-03 log throughput
log_entries_per_second_target = 1000
test_duration_secs = 30
Tests & Step-by-Step Execution
BM-DB-01 — O(1) parallel save validation ⭐ Key claim
bash# Step 1: Reset DB
rm -f bench_test.db && sqlx migrate run

# Step 2: Run save benchmark across all agent count tiers
cargo bench --bench persistence_bench -- bench_parallel_agent_saves

# Step 3: Record wall-clock time for each tier:
#   1 agent   → ___ ms
#   5 agents  → ___ ms
#   10 agents → ___ ms
#   25 agents → ___ ms
#   50 agents → ___ ms

# Pass condition: 50-agent time < 2x 5-agent time
# If linear growth is observed: join_all() parallelism is broken — investigate
BM-DB-02 — Mission CRUD throughput
bash# Step 1: Spawn 20 concurrent writers, each inserting 100 missions
cargo bench --bench persistence_bench -- bench_mission_crud_throughput

# Step 2: Record inserts/sec
# Step 3: Check for lock contention errors in RUST_LOG=warn output
BM-DB-03 — Log append throughput
bash# Step 1: Simulate high-frequency telemetry (agent thinking/idle events)
cargo bench --bench persistence_bench -- bench_log_append_throughput

# Step 2: Record sustained append rate (entries/sec)
# Step 3: Verify no dropped log entries (count inserted vs count attempted)
BM-DB-04 — Read latency during active mission
bash# Step 1: Start a background mission writing logs continuously
# Step 2: Measure agent config fetch latency concurrently
cargo bench --bench persistence_bench -- bench_read_under_write_load

# Step 3: Record p50, p95, p99 read latency
# Pass condition: p99 read latency < 10ms
BM-DB-05 — SQLite vs JSON fallback overhead
bash# Step 1: Run with SQLite (default)
LEGACY_JSON_BACKUP=false cargo bench --bench persistence_bench -- bench_write_overhead

# Step 2: Run with JSON fallback enabled
LEGACY_JSON_BACKUP=true cargo bench --bench persistence_bench -- bench_write_overhead

# Step 3: Record overhead delta
# JSON fallback should be measurably slower — document the cost

BM-MCP — MCP Host (mcp.rs)
Purpose: Validate skill cache performance, zero-downtime hot-swap behavior, and **McpToolStats** telemetry accuracy.
Setup Parameters
bashexport MCP_CACHE_ENABLED=true
export MCP_SUBPROCESS_TIMEOUT_SECS=60

# Create a minimal no-op bench skill for BM-MCP-03
cat > server-rs/data/skills/echo_skill.sh << 'EOF'
#!/bin/bash
echo "BENCH_OK"
EOF
chmod +x server-rs/data/skills/echo_skill.sh
rust// BM-MCP-04 concurrent execution config
concurrent_tool_calls = 20
tool_type = "legacy_skill"      // subprocess path — most realistic

// BM-MCP-05 hot-swap config
active_request_rate = 50        // requests/sec during registry swap
swap_iterations = 10
acceptable_dropped_requests = 0
Tests & Step-by-Step Execution
BM-MCP-01 — Cold start skill load
bash# Step 1: Clear skill cache
# Step 2: Time the server startup to first skill response
cargo bench --bench mcp_bench -- bench_cold_start_cache_load

# Record: time_to_cache_ready_ms
# Pass condition: < 500ms (cache must be ready before first agent request)
BM-MCP-02 — Tool discovery cache hit latency
bash# Step 1: Pre-warm cache (run one discovery call first)
# Step 2: Benchmark subsequent cache-hit discovery calls
cargo bench --bench mcp_bench -- bench_tool_discovery_cache_hit

# Record: mean latency ms
# Pass condition: < 5ms per discovery call (in-memory, no disk I/O)
BM-MCP-03 — Legacy skill subprocess spawn baseline
bash# Step 1: Use echo_skill.sh (fast no-op) to isolate spawn overhead
cargo bench --bench mcp_bench -- bench_subprocess_spawn

# Record: spawn_overhead_ms (time from call to first byte of output)
# This becomes the baseline deducted from real skill measurements
BM-MCP-04 — 20 concurrent tool executions
bash# Step 1: Fire 20 simultaneous tool calls via FuturesUnordered
cargo bench --bench mcp_bench -- bench_concurrent_tool_execution

# Step 2: Record wall-clock completion time
# Step 3: Compare vs 20 sequential tool calls (run bench_sequential_tool_execution)
# Pass condition: concurrent < 2x single tool call time
BM-MCP-05 — Registry hot-swap under load (zero downtime)
bash# Step 1: Start engine with 50 req/sec load via k6
k6 run --vus 5 --duration 60s scripts/k6_mcp_discovery.js

# Step 2: While k6 is running, trigger a registry reload
curl -X POST http://localhost:8000/v1/api/mcp/reload \
  -H "Authorization: Bearer tadpole-dev-token-2026"

# Step 3: Check k6 output for any failed requests during swap window
# Pass condition: 0 failed requests during hot-swap

BM-WS — WebSocket & Event Bus (ws.rs)
Purpose: Verify high-speed binary telemetry throughput and that the Swarm Visualizer maintains 60fps under load.
Setup Parameters
rust// Server-side broadcast channel capacity (must match AppState)
broadcast_channel_capacity = 1024

// Test message payload (approximate average LogEntry size)
message_payload_bytes = 256

// BM-WS-03 concurrent clients
client_count = 10
active_swarm_agents = 50
test_duration_secs = 30

// BM-WS-06 Swarm Pulse (Binary)
pulse_cadence_ms = 100
binary_header = 0x02
serialization = "MessagePack"

// BM-WS-05 circular buffer
buffer_slots = 1000             // match eventBus.ts CIRCULAR_BUFFER_SIZE
write_iterations = 100_000
typescript// Frontend measurement config
const TARGET_FPS = 60;
const RAF_FLUSH_STRATEGY = "requestAnimationFrame";
const PERF_OBSERVER_ENTRY_TYPE = "measure";
Tests & Step-by-Step Execution
BM-WS-01 — Max event throughput before message drop
bash# Step 1: Start the engine
cargo run --release

# Step 2: Connect websocat and count received messages
websocat --text "ws://localhost:3000/ws?token=bench-token-local" \
  | pv -l -r > /dev/null &

# Step 3: Inject events at increasing rates via internal test endpoint
# Target rates: 100 / 500 / 1000 / 5000 events/sec

# Step 4: Record max rate before first dropped message
# Check broadcast channel: if receiver lags > capacity (1024), drops occur
BM-WS-02 — Multiplexed channel interleave latency
bash# Step 1: Start engine and connect client
# Step 2: Send simultaneous LogEntry + EngineEvent via tokio::select!
cargo bench --bench ws_bench -- bench_multiplexed_channel_latency

# Record: timestamp delta between LogEntry send and EngineEvent send
# Pass condition: interleave latency < 1ms (select! should be near-instant)
BM-WS-03 — 10 simultaneous WebSocket clients during swarm
bash# Step 1: Start engine with 10 active agents running missions
# Step 2: Open 10 WebSocket connections simultaneously
for i in {1..10}; do
  websocat --text "ws://localhost:8000/engine/ws?token=tadpole-dev-token-2026" \
    > /tmp/ws_client_$i.log 2>&1 &
done

# Step 3: Run for 30 seconds, then check each client log for errors
sleep 30
for i in {1..10}; do
  echo "Client $i lines: $(wc -l < /tmp/ws_client_$i.log)"
done

# Pass condition: all 10 clients receive identical event counts (no drops)
BM-WS-04 — Frontend RAF batching at 100/500/1000 events/sec ⭐ Key claim
bash# Step 1: Build frontend in production mode
cd src && vite build --mode production

# Step 2: Serve production build
npx serve dist -p 3000

# Step 3: Open Chrome, open DevTools → Performance tab
# Step 4: Start recording, trigger swarm at target event rates
# Step 5: Stop recording and inspect FPS chart

# Pass condition at 500 events/sec: FPS must not drop below 55fps
# Measure: Chrome DevTools FPS meter + React Profiler "actualDuration"
BM-WS-05 — Circular EventBus ring buffer O(1) write
bash# Step 1: Run the ring buffer write benchmark
cargo bench --bench ws_bench -- bench_circular_buffer_writes

# Step 2: Plot write latency across 100,000 iterations
# Pass condition: flat latency curve (no growth as buffer fills/wraps)

BM-WS-06 — Swarm Pulse Binary Aggregation Latency
bash# Step 1: Benchmark the pulse loop's ability to map 50 agents to MessagePack
cargo bench --bench ws_bench -- bench_swarm_pulse_aggregation

# Step 2: Record mean latency ms
# Pass condition: < 1ms for 50 agents (Zero-allocation target)

BM-FS — FilesystemAdapter & Security (filesystem.rs)
Purpose: Measure I/O throughput AND verify the sandbox is impenetrable. Security tests are zero-tolerance.
Setup Parameters
bash# Create bench workspace
export BENCH_WORKSPACE="/tmp/tadpole_bench/cluster_bench_001"
mkdir -p $BENCH_WORKSPACE

# Create a symlink attack target outside the sandbox (for BM-FS-03)
mkdir -p /tmp/outside_sandbox
echo "SECRET" > /tmp/outside_sandbox/secret.txt
ln -s /tmp/outside_sandbox $BENCH_WORKSPACE/symlink_to_outside
rust// BM-FS-01 throughput
file_sizes_kb = [1, 10, 100, 1024]
concurrent_ops = 10

// BM-FS-02 canonicalize overhead
iterations = 10_000
path_depth_levels = [1, 3, 5, 10]

// BM-FS-03 escape attack vectors — ALL must return Err()
attack_paths = [
    "../../../etc/passwd",
    "subdir/../../secret",
    "/etc/hosts",                          // absolute path outside workspace
    "symlink_to_outside/secret.txt",       // symlink escape
    "subdir/../../../etc/shadow",
    "....//....//etc/passwd",              // double-dot variant
    "%2e%2e%2f%2e%2e%2fetc%2fpasswd",    // URL-encoded traversal
]

// BM-FS-04 concurrent agents
agent_count = 50
writes_per_agent = 20
file_size_kb = 10
Tests & Step-by-Step Execution
BM-FS-01 — File I/O throughput
bash# Step 1: Pre-create test files at each size
for size in 1 10 100 1024; do
  dd if=/dev/urandom of=$BENCH_WORKSPACE/test_${size}kb.bin bs=1k count=$size 2>/dev/null
done

# Step 2: Run throughput benchmark
cargo bench --bench filesystem_bench -- bench_file_io_throughput

# Step 3: Record read MB/s and write MB/s per file size tier
BM-FS-02 — canonicalize overhead
bash# Step 1: Benchmark canonicalize at varying path depths
cargo bench --bench filesystem_bench -- bench_canonicalize_overhead

# Record: mean time per call at each depth level
# Pass condition: < 100µs per call even at depth 10
BM-FS-03 — Symlink escape battery ⭐ Zero-tolerance security test
bash# Step 1: Run attack vector suite
cargo test --release -- sandbox_escape_tests --nocapture

# Expected output for EVERY attack path:
# [BLOCKED] "../../../etc/passwd" → Err(SandboxViolation)
# [BLOCKED] "symlink_to_outside/secret.txt" → Err(SandboxViolation)
# ... (all 7 vectors must show BLOCKED)

# Step 2: Verify /tmp/outside_sandbox/secret.txt was never read
cat /tmp/outside_sandbox/secret.txt   # must still contain "SECRET" untouched

# FAIL condition: ANY Err() that contains file content = immediate blocker
BM-FS-04 — 50 concurrent agents writing to isolated workspaces
bash# Step 1: Create 50 isolated workspace dirs
for i in {1..50}; do
  mkdir -p /tmp/tadpole_bench/cluster_bench_$i
done

# Step 2: Run concurrent write stress
cargo bench --bench filesystem_bench -- bench_concurrent_workspace_writes

# Step 3: Verify no cross-workspace file leakage
# Check: files written by agent N are ONLY in cluster_bench_N
for i in {1..50}; do
  echo "Workspace $i files: $(ls /tmp/tadpole_bench/cluster_bench_$i | wc -l)"
done

# Pass condition: each workspace has exactly (writes_per_agent = 20) files
BM-FS-05 — list_files sorted at scale
bash# Step 1: Populate workspace with 1,000 files
for i in $(seq -w 1 1000); do touch $BENCH_WORKSPACE/file_$i.txt; done

# Step 2: Benchmark list at 1k files
cargo bench --bench filesystem_bench -- bench_list_files_1k

# Step 3: Populate to 10,000 files and repeat
for i in $(seq -w 1001 10000); do touch $BENCH_WORKSPACE/file_$i.txt; done
cargo bench --bench filesystem_bench -- bench_list_files_10k

# Record: mean list_files latency at 1k vs 10k
# Pass condition: sub-100ms at 10k files

BM-HTTP — HTTP Client & Provider Adapters
Purpose: Confirm the shared reqwest::Client connection pool eliminates TLS overhead and delivers efficient concurrent LLM calls.
Setup Parameters
rust// Match production AppState client config exactly
pool_max_idle_per_host = 20
connection_timeout = Duration::from_secs(10)
request_timeout = Duration::from_secs(120)

// BM-HTTP-01
cold_start_iterations = 5      // new reqwest::Client each iteration
warm_iterations = 50           // reuse shared AppState client
measure = "time_to_first_byte_ms"

// BM-HTTP-02
concurrent_requests = 20
use_mock_endpoint = true       // local mock server to isolate pool perf from network

// BM-HTTP-03
inject_tool_use_failed = true
max_correction_retries = 3
measure = "total_retry_cycle_ms"

// BM-HTTP-04 provider comparison
bench_prompt = "Return only the word PONG"   // minimal, deterministic
runs_per_provider = 20
bash# Start a local mock LLM server for BM-HTTP-01 and BM-HTTP-02
# (Eliminates network variance — isolates pool behavior)
npx json-server --port 9999 --watch mock_llm_responses.json
Tests & Step-by-Step Execution
BM-HTTP-01 — Cold vs warm connection (pool reuse validation) ⭐ Key claim
bash# Step 1: Benchmark cold path (new client each time)
cargo bench --bench http_bench -- bench_cold_tls_handshake

# Step 2: Benchmark warm path (AppState shared client)
cargo bench --bench http_bench -- bench_warm_pool_reuse

# Step 3: Calculate reduction percentage:
# reduction = (cold_mean - warm_mean) / cold_mean * 100
# Pass condition: reduction ≥ 90%

# If < 90%: verify pool_max_idle_per_host=20 is set in AppState
BM-HTTP-02 — 20 concurrent LLM calls via shared pool
bash# Step 1: Start mock LLM server
npx json-server --port 9999 mock_llm_responses.json &

# Step 2: Run concurrent calls against mock
cargo bench --bench http_bench -- bench_concurrent_pool_calls

# Step 3: Verify no new TLS handshakes after first 20 (pool is saturated)
# Check: RUST_LOG=debug output should not show "connect" after warmup

# Record: total_time_ms for 20 concurrent calls
# Compare vs: 20 * single_call_mean_ms to confirm parallelism
BM-HTTP-03 — Groq self-healing retry overhead
bash# Step 1: Configure a test that injects tool_use_failed on first attempt
cargo test --release -- groq_self_healing_retry --nocapture

# Step 2: Measure total cycle time: initial call + correction + retry
cargo bench --bench http_bench -- bench_groq_retry_overhead

# Record: baseline_call_ms vs retry_cycle_ms
# Pass condition: retry overhead < 2x baseline (correction prompt must be minimal)
BM-HTTP-04 — Gemini vs Groq latency comparison
bash# Step 1: Run 20 identical minimal prompts against Groq
PROVIDER=groq cargo bench --bench http_bench -- bench_provider_latency

# Step 2: Run same 20 prompts against Gemini
PROVIDER=gemini cargo bench --bench http_bench -- bench_provider_latency

# Step 3: Record p50, p95, p99 for each provider
# This data informs model config defaults — no pass/fail, purely informational

BM-FE — Frontend Performance
Purpose: Verify React rendering stays efficient under live swarm load, vendor chunks are correctly split, and the RAF batching strategy prevents UI jank.
Setup Parameters
bash# Always test production builds
cd src
vite build --mode production

# Serve production build locally
npx serve dist -p 4000

# Confirm bundle chunks exist (vendor splitting validation)
ls dist/assets/ | grep vendor
# Expected: vendor-react.js, vendor-framer.js, vendor-forcegraph.js, etc.
typescript// React Profiler measurement targets
const PROFILER_THRESHOLDS = {
  singleAgentUpdateMs: 2,       // BM-FE-02: max re-render for 1 agent update
  hierarchyNodeCascadeMs: 0,    // BM-FE-03: zero cascade re-renders
  mapIndexBuildMs: {
    agents26: 1,
    agents100: 5,
    agents500: 20,
  }
};

// BM-FE-05 live swarm config
const ACTIVE_AGENTS = 26;
const TELEMETRY_EVENTS_PER_SEC = 50;
const TARGET_FPS = 60;
Tests & Step-by-Step Execution
BM-FE-01 — Bundle load time + chunk splitting
bash# Step 1: Run Lighthouse against production build
lighthouse http://localhost:4000 \
  --output json \
  --output html \
  --output-path ./bench_results/lighthouse_report \
  --chrome-flags="--headless" \
  --preset desktop

# Step 2: Open bench_results/lighthouse_report.html
# Check: "Eliminate render-blocking resources" — must show 0 blocking scripts

# Step 3: Analyze chunk sizes
npx vite-bundle-analyzer dist/stats.json

# Pass condition:
# - Total initial JS < 500kb gzipped
# - No single vendor chunk > 200kb gzipped
# - Each vendor (react, framer-motion, react-force-graph-2d, lucide, zustand) in own chunk
BM-FE-02 — Single agent state update re-render scope
bash# Step 1: Open Chrome with React DevTools installed
# Step 2: Navigate to dashboard with 26 agents loaded
# Step 3: Open React DevTools → Profiler tab → Start recording

# Step 4: Trigger a single agent token count update (simulate via WebSocket)
# Step 5: Stop recording

# In Profiler output:
# - Find the AgentCard component that updated
# - Record "actualDuration" ms
# - Verify NO other AgentCard components show re-renders (gray = no render)

# Pass condition: actualDuration < 2ms, 0 cascade re-renders on sibling cards
BM-FE-03 — HierarchyNode memoization validation
bash# Step 1: Open React Profiler with Hierarchy view active
# Step 2: Update one leaf node's token count
# Step 3: Inspect which HierarchyNode components re-rendered

# Expected: ONLY the directly updated node renders
# Fail condition: parent or sibling HierarchyNode components show re-renders

# If failing: check React.memo custom comparison logic in HierarchyNode.tsx
# The comparator must be checking agent-level identity, not reference equality
BM-FE-04 — commandProcessor.ts Map index build time
bash# Step 1: Add measurement to commandProcessor.ts (temp bench code)
# Insert before and after buildIndex():
const t0 = performance.now();
buildIndex(agents);
const t1 = performance.now();
console.log(`Index build: ${t1 - t0}ms for ${agents.length} agents`);

# Step 2: Test at each agent count tier
# Load dashboard with 26 agents → record console output
# Mock 100 agents in store → record
# Mock 500 agents in store → record

# Pass conditions:
# 26 agents:  < 1ms
# 100 agents: < 5ms
# 500 agents: < 20ms
BM-FE-05 — Full dashboard under live swarm load
bash# Step 1: Start engine with 26 agents
cargo run --release

# Step 2: Open production dashboard in Chrome
# Step 3: Open DevTools → Performance → Enable FPS meter (top right)

# Step 4: Start all 26 agents on missions simultaneously
# Watch FPS meter for 60 seconds

# Step 5: Open DevTools → Performance → Record 10 seconds of swarm activity
# After recording: inspect the FPS chart and Long Tasks (red bars)

# Pass condition:
# - FPS stays between 55-60fps throughout
# - No Long Tasks > 50ms
# Pass condition:
# - FPS stays between 55-60fps throughout
# - No Long Tasks > 50ms
# - Memory usage stable (no upward drift indicating leak)

# Step 6: Record: min_fps, avg_fps, long_task_count, memory_delta_mb

BM-SEC — Security & Integrity (security.rs, audit.rs, monitoring.rs)
Purpose: Measure the performance impact of cryptographic audit trails and real-time resource guarding.

BM-SEC-01 — Merkle Signature Latency (ED25519)
- **Scenario**: Generate 1,000 signed audit entries.
- **Action**: Benchmark `sign_record()` with ED25519.
- **Pass Target**: Mean latency < 500µs per signature.

BM-SEC-02 — Merkle Chain Verification Throughput
- **Scenario**: Verify a chain of 10,000 entries.
- **Action**: Run `verify_chain()`.
- **Pass Target**: Total verification time < 200ms.

BM-SEC-03 — Resource Guard Telemetry Overhead
- **Scenario**: System under high load (BM-RUN-02).
- **Action**: measure CPU usage with and without `SecurityMonitor` active.
- **Pass Target**: Overhead < 3% of total CPU.

BM-SEC-04 — Sandbox Detection Latency
- **Scenario**: Engine startup in containerized vs non-containerized environments.
- **Action**: Measure `detect_sandbox()` during setup.
- **Pass Target**: Latency < 10ms.

BM-SEC-05 — Mission-Level Quota Enforcement
- **Scenario**: Active mission with a $0.10 quota.
- **Action**: Simulate rapid token usage and verify automatic pause.
- **Pass Target**: Kill signal within < 500ms of quota breach.

BM-SEC-06 — Oversight Gate Flag Overhead
- **Scenario**: 100 tool calls with `Requires Oversight` enabled vs disabled.
- **Action**: Measure time to initiate tool call.
- **Pass Target**: Overhead < 1ms per call (flag check should be O(1) in-memory).

BM-QUAL — Quality & Mission Success
Purpose: Formalize the evaluation of the **Mission Analysis** engine (Agent 99) and ensure it meets industry standards for prescriptive auditing.

BM-QUAL-01 — Analysis Latency
- **Scenario**: Mission succeeds with 50 log entries.
- **Action**: Trigger Mission Analysis.
- **Pass Target**: Analysis result generated in < 20s (depending on provider).

BM-QUAL-02 — Insight Relevance
- **Scenario**: Use a mission that deliberately fails a goal (e.g., target file not found).
- **Action**: Run Mission Analysis.
- **Pass Target**: Success Auditor (Agent 99) correctly identifies the failure and suggests a corrective skill (e.g., `list_files`).

BM-QUAL-03 — Schema Adherence
- **Scenario**: Collect 10 analysis reports.
- **Action**: Validate results against the Success Auditor's prescriptive schema.
- **Pass Target**: 100% adherence to reporting structure.

5. Execution Plan
Week 1 — Baseline Setup

 Complete all steps in Section 2 (Global Environment Setup)
 Complete all steps in Section 3 (Tooling Installation)
 Create server-rs/benches/ directory and add Criterion to Cargo.toml
 Run BM-RUN-01 (single agent baseline)
 Run BM-DB-01 at agent_count=1 only
 Run BM-RL-01 (no-op overhead)
 Save all outputs to bench_results/week1_baseline/
 Commit bench_metadata.txt to version control

Week 2 — Core Engine

 Run all BM-RUN tests (01–05)
 Run all BM-RL tests (01–05)
 Run all BM-MCP tests (01–05)
 Validate key claim: BM-RUN-03 parallel swarming ≥ 70% improvement
 Validate key claim: BM-DB-01 O(1) save time
 Save results to bench_results/week2_core/

Week 3 — I/O & Security

 Run all BM-FS tests (01–05)
 Run full BM-FS-03 symlink escape battery — document every result
 Run all BM-HTTP tests (01–04)
 Validate key claim: BM-HTTP-01 ≥ 90% TLS reduction
 Save results to bench_results/week3_io_security/

Week 4 — Real-Time & Frontend

 Run all BM-WS tests (01–05)
 Build frontend in production mode, run all BM-FE tests (01–05)
 Full integration test: 26 agents, active swarm, dashboard live for 60s
 Validate key claim: BM-WS-04 60fps at 500 events/sec
 Save results to bench_results/week4_realtime_fe/

Week 5 — Report & Optimize

 Compile all results into bench_results/final_summary.csv
 Mark each test PASS / FAIL / NEEDS INVESTIGATION
 File GitHub issues for any failing benchmarks with reproduction steps
 Schedule optimization sprint for failures
 Archive bench_results/ folder with git tag bench-v1.0


6. Output & Reporting Format
Per-Run Outputs
OutputLocationFormatCriterion reportsserver-rs/target/criterion/HTML + JSONLighthouse reportsbench_results/lighthouse_report.*HTML + JSONSecurity test logbench_results/security_BM-FS-03.logPlain textProvider comparisonbench_results/http_provider_comparison.csvCSVFinal summarybench_results/final_summary.csvCSV
CSV Summary Schema
csvtest_id,description,mean_ms,p95_ms,p99_ms,target,pass_fail,notes
BM-RUN-03,Parallel vs sequential swarm,45.2,52.1,58.3,"≥70% reduction",PASS,"72% improvement observed"
BM-FS-03,Symlink escape battery,N/A,N/A,N/A,"0 escapes",PASS,"All 7 vectors blocked"

7. Regression Policy
RuleThresholdActionPerformance regression> 10% from recorded baselineBlocking PR review requiredSandbox escape (BM-FS-03)Any escapeImmediate blocker — no mergeRate limiter no-op (BM-RL-01)> 1ms in release buildBlocking — check for accidental mutexFPS drop (BM-WS-04)Below 55fps at 500 events/secBlocking — investigate RAF flushDB save regression (BM-DB-01)50-agent time > 2x 5-agent timeBlocking — join_all() likely broken

Review this spec alongside `agent/runner/mod.rs`, `agent/rate_limiter.rs`, `agent/mcp.rs`, and `adapter/filesystem.rs` before beginning Week 1.