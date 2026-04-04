// k6 script for benchmarking MCP discovery performance
import http from 'k6/http';
import { sleep, check } from 'k6';

export const options = {
    vus: 5,
    duration: '30s',
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8000/v1';
const TOKEN = __ENV.NEURAL_TOKEN || 'tadpole-dev-token-2026';

export default function () {
    const params = {
        headers: {
            'Authorization': `Bearer ${TOKEN}`,
            'Content-Type': 'application/json',
        },
    };

    const res = http.get(`${BASE_URL}/api/mcp/tools`, params);

    check(res, {
        'status is 200': (r) => r.status === 200,
        'latency < 200ms': (r) => r.timings.duration < 200,
    });

    sleep(0.1); // Small gap between requests
}
