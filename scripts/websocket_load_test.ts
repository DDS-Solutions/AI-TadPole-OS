/**
 * @docs ARCHITECTURE:Quality:Verification
 *
 * ### AI Context Alignment
 * - **Subsystem**: Developer Scripts / websocket_load_test
 *
 * ### ⚠️ Invariants & Non-Negotiables
 * - `[Structural]` Deterministic execution without side effects outside declared scope.
 *
 * ### 🔍 Debugging & Observability
 * - **Local Errors**: none
 * - **Telemetry Targets**: none declared
 * - **Witness Tests**: none declared
 */

import WebSocket from 'ws';
import { performance } from 'perf_hooks';

// Load testing parameters
const CONCURRENCY = parseInt(process.env.WS_CONCURRENCY || '50', 10);
const DURATION_MS = parseInt(process.env.WS_DURATION_MS || '10000', 10);
const TARGET_URL = process.env.WS_TARGET_URL || 'ws://localhost:8000/v1/engine/ws';
const AUTH_TOKEN = process.env.NEURAL_TOKEN || 'tadpole-2026-dev';

interface ClientStats {
    clientId: number;
    connected: boolean;
    authenticated: boolean;
    errored: boolean;
    closed: boolean;
    messagesReceived: number;
    bytesReceived: number;
    pingsSent: number;
    pongsReceived: number;
    latencies: number[];
}

async function runBenchmark() {
    console.log(`====================================================`);
    console.log(`🚀 STARTING WEBSOCKET LOAD TEST & BENCHMARK`);
    console.log(`====================================================`);
    console.log(`Target URL:     ${TARGET_URL}`);
    console.log(`Concurrency:    ${CONCURRENCY} connections`);
    console.log(`Duration:       ${DURATION_MS / 1000} seconds`);
    console.log(`Subprotocol:    tadpole-pulse-v1`);
    console.log(`====================================================\n`);

    const clients: WebSocket[] = [];
    const stats: ClientStats[] = [];

    let activeConnections = 0;
    const startBenchmarkTime = performance.now();

    // Helper to sleep
    const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

    // Initialize stats
    for (let i = 0; i < CONCURRENCY; i++) {
        stats.push({
            clientId: i,
            connected: false,
            authenticated: false,
            errored: false,
            closed: false,
            messagesReceived: 0,
            bytesReceived: 0,
            pingsSent: 0,
            pongsReceived: 0,
            latencies: []
        });
    }

    // Connect a single client
    const connectClient = (id: number): Promise<void> => {
        return new Promise((resolve) => {
            const clientStats = stats[id];
            let pingTimer: NodeJS.Timeout | null = null;
            let lastPingTime = 0;

            const ws = new WebSocket(TARGET_URL, ['tadpole-pulse-v1']);
            clients[id] = ws;

            ws.on('open', () => {
                clientStats.connected = true;
                activeConnections++;
                
                // Send post-connect auth frame
                const authPayload = JSON.stringify({
                    type: 'auth',
                    token: AUTH_TOKEN
                });
                ws.send(authPayload);
            });

            ws.on('message', (data: WebSocket.Data) => {
                clientStats.messagesReceived++;
                
                let isBinary = false;
                let len = 0;
                let messageString = '';

                if (data instanceof Buffer) {
                    len = data.length;
                    isBinary = data.length > 0 && (data[0] === 1 || data[0] === 2);
                    if (!isBinary) {
                        messageString = data.toString('utf8');
                    }
                } else if (typeof data === 'string') {
                    len = Buffer.byteLength(data);
                    messageString = data;
                } else if (data instanceof ArrayBuffer) {
                    len = data.byteLength;
                    const view = new Uint8Array(data);
                    isBinary = view.length > 0 && (view[0] === 1 || view[0] === 2);
                    if (!isBinary) {
                        messageString = Buffer.from(data).toString('utf8');
                    }
                }

                clientStats.bytesReceived += len;

                if (id === 0 && clientStats.messagesReceived <= 5 && messageString) {
                    console.log(`[Client 0 Message ${clientStats.messagesReceived}]:`, messageString);
                }

                if (!isBinary && messageString) {
                    try {
                        const parsed = JSON.parse(messageString);
                        if (parsed.type === 'auth_ok') {
                            clientStats.authenticated = true;
                            
                            // Setup periodic ping-pong latency check
                            pingTimer = setInterval(() => {
                                if (ws.readyState === WebSocket.OPEN) {
                                    lastPingTime = performance.now();
                                    clientStats.pingsSent++;
                                    ws.ping();
                                }
                            }, 1000);

                            resolve(); // Successfully connected and authenticated
                        } else if (parsed.type === 'auth_error') {
                            console.error(`[Client ${id}] Auth Error: ${parsed.message}`);
                            ws.close();
                        }
                    } catch {
                        // Regular JSON messages or updates from server
                    }
                }
            });

            ws.on('pong', () => {
                if (lastPingTime > 0) {
                    const rtt = performance.now() - lastPingTime;
                    clientStats.pongsReceived++;
                    clientStats.latencies.push(rtt);
                }
            });

            ws.on('error', (err) => {
                clientStats.errored = true;
                console.error(`[Client ${id}] Socket Error:`, err.message);
                resolve(); // Resolve to avoid blocking benchmark startup
            });

            ws.on('close', () => {
                if (clientStats.connected) {
                    activeConnections--;
                }
                clientStats.connected = false;
                clientStats.closed = true;
                if (pingTimer) {
                    clearInterval(pingTimer);
                }
                resolve();
            });
        });
    };

    // Connect clients in staggered bursts to prevent handshake stampedes
    console.log(`Connecting ${CONCURRENCY} clients...`);
    const connectionStart = performance.now();
    for (let i = 0; i < CONCURRENCY; i++) {
        connectClient(i);
        if (i % 10 === 0 && i > 0) {
            await sleep(100); // 100ms throttle every 10 connections
        }
    }

    // Wait a brief period for all connections to stabilize
    await sleep(2000);
    const connectionTime = (performance.now() - connectionStart) / 1000;
    console.log(`Connections initialized. Active: ${activeConnections}/${CONCURRENCY} in ${connectionTime.toFixed(2)}s\n`);

    console.log(`Running throughput benchmark for ${DURATION_MS / 1000}s...`);
    await sleep(DURATION_MS);

    // Tear down connections
    console.log(`\nTearing down benchmark...`);
    for (const ws of clients) {
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.close();
        }
    }
    
    await sleep(1000); // Wait for closes to finish

    // Aggregate statistics
    const totalDuration = (performance.now() - startBenchmarkTime) / 1000;
    const connectedCount = stats.filter(s => s.authenticated).length;
    const errorCount = stats.filter(s => s.errored).length;
    const closedCount = stats.filter(s => s.closed).length;
    
    const totalMessages = stats.reduce((acc, s) => acc + s.messagesReceived, 0);
    const totalBytes = stats.reduce((acc, s) => acc + s.bytesReceived, 0);
    
    let allLatencies: number[] = [];
    stats.forEach(s => {
        allLatencies = allLatencies.concat(s.latencies);
    });

    const averageLatency = allLatencies.length > 0 
        ? allLatencies.reduce((acc, l) => acc + l, 0) / allLatencies.length
        : 0;

    const minLatency = allLatencies.length > 0 ? Math.min(...allLatencies) : 0;
    const maxLatency = allLatencies.length > 0 ? Math.max(...allLatencies) : 0;
    
    const throughputMsgPerSec = totalMessages / totalDuration;
    const throughputKbPerSec = (totalBytes / 1024) / totalDuration;

    console.log(`\n====================================================`);
    console.log(`📊 BENCHMARK RESULTS SUMMARY`);
    console.log(`====================================================`);
    console.log(`Total Connections Attempted:  ${CONCURRENCY}`);
    console.log(`Successful Authentications:  ${connectedCount} (${((connectedCount / CONCURRENCY) * 100).toFixed(1)}%)`);
    console.log(`Connection Errors:           ${errorCount}`);
    console.log(`Closed Connections:          ${closedCount}`);
    console.log(`Benchmark Duration:          ${totalDuration.toFixed(2)}s`);
    console.log(`----------------------------------------------------`);
    console.log(`Total Messages Received:     ${totalMessages}`);
    console.log(`Total Bytes Received:        ${(totalBytes / (1024 * 1024)).toFixed(2)} MB`);
    console.log(`Message Throughput:          ${throughputMsgPerSec.toFixed(2)} msg/sec`);
    console.log(`Byte Throughput:             ${throughputKbPerSec.toFixed(2)} KB/sec`);
    console.log(`----------------------------------------------------`);
    console.log(`Average Ping-Pong Latency:   ${averageLatency.toFixed(2)} ms`);
    console.log(`Min Ping-Pong Latency:       ${minLatency.toFixed(2)} ms`);
    console.log(`Max Ping-Pong Latency:       ${maxLatency.toFixed(2)} ms`);
    console.log(`====================================================\n`);

    // Verification check for exit status
    if (connectedCount < CONCURRENCY * 0.9) {
        console.error(`❌ Benchmark Failed: Connection success rate is below 90%!`);
        process.exit(1);
    }
    
    console.log(`✅ Benchmark Completed Successfully!`);
    process.exit(0);
}

runBenchmark().catch(err => {
    console.error('Fatal benchmark error:', err);
    process.exit(1);
});
