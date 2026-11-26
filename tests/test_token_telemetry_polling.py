"""
@docs ARCHITECTURE:Core:Audit

### AI Context Alignment
- **Subsystem**: Test Verification Suite / test_token_telemetry_polling
- **Primary Entrypoints**: `extract_telemetry_metrics`, `TestTokenTelemetryParity`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic internal state integrity and strict interface contract compliance.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
"""

import unittest
from unittest.mock import MagicMock


def extract_telemetry_metrics(agent_dict: dict) -> tuple[int, float]:
    """
    Extracts lifetime tokens and cost from an agent payload,
    supporting both camelCase and snake_case representations.
    """
    tokens = agent_dict.get("tokensUsed")
    if tokens is None:
        tokens = agent_dict.get("tokens_used", 0)

    cost = agent_dict.get("costUsd")
    if cost is None:
        cost = agent_dict.get("cost_usd", 0.0)

    return int(tokens), float(cost)


def poll_for_token_accumulation(state_getter, initial_state: dict, timeout_secs: float = 10.0, interval_secs: float = 0.1) -> tuple[bool, int, float]:
    """
    Simulated adaptive polling engine.
    Returns (success, accumulated_tokens, elapsed_seconds).
    """
    max_polls = max(1, int(timeout_secs / interval_secs))
    for i in range(max_polls):
        current = state_getter()
        delta = sum(current[aid]["tokens_used"] - initial_state[aid]["tokens_used"] for aid in initial_state)
        elapsed = (i + 1) * interval_secs
        if delta > 0:
            return True, delta, elapsed
    return False, 0, timeout_secs


class TestTokenTelemetryParity(unittest.TestCase):

    def test_dual_case_extraction(self):
        # Case A: Pure camelCase (REST API DTO)
        camel_agent = {"tokensUsed": 2741640, "costUsd": 5.1966}
        t, c = extract_telemetry_metrics(camel_agent)
        self.assertEqual(t, 2741640)
        self.assertAlmostEqual(c, 5.1966, places=4)

        # Case B: Pure snake_case (SQLite representation)
        snake_agent = {"tokens_used": 3924829, "cost_usd": 8.2274}
        t, c = extract_telemetry_metrics(snake_agent)
        self.assertEqual(t, 3924829)
        self.assertAlmostEqual(c, 8.2274, places=4)

        # Case C: Dual-case payload
        dual_agent = {"tokensUsed": 1000, "tokens_used": 1000, "costUsd": 0.05, "cost_usd": 0.05}
        t, c = extract_telemetry_metrics(dual_agent)
        self.assertEqual(t, 1000)
        self.assertAlmostEqual(c, 0.05, places=4)

    def test_adaptive_polling_success(self):
        initial = {"1": {"tokens_used": 100}}

        # State sequence: 2 polls unchanged, then +500 tokens
        states = [
            {"1": {"tokens_used": 100}},
            {"1": {"tokens_used": 100}},
            {"1": {"tokens_used": 600}},
        ]
        getter = MagicMock(side_effect=states)

        success, delta, elapsed = poll_for_token_accumulation(getter, initial, timeout_secs=5.0, interval_secs=0.01)
        self.assertTrue(success)
        self.assertEqual(delta, 500)
        self.assertLessEqual(elapsed, 0.05)
        self.assertEqual(getter.call_count, 3)

    def test_adaptive_polling_timeout(self):
        initial = {"1": {"tokens_used": 100}}
        getter = MagicMock(return_value={"1": {"tokens_used": 100}})

        success, delta, elapsed = poll_for_token_accumulation(getter, initial, timeout_secs=0.05, interval_secs=0.01)
        self.assertFalse(success)
        self.assertEqual(delta, 0)
        self.assertEqual(getter.call_count, 5)


if __name__ == "__main__":
    unittest.main()
