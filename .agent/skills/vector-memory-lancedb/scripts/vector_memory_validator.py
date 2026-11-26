#!/usr/bin/env python3
"""
@docs ARCHITECTURE:Documentation

### AI Context Alignment
- **Subsystem**: Agent Skills / vector-memory-lancedb / vector_memory_validator
- **Primary Entrypoints**: `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import sys
import math
import argparse

if hasattr(sys.stdout, 'reconfigure'):
    sys.stdout.reconfigure(encoding='utf-8')


def cosine_similarity(v1: list[float], v2: list[float]) -> float:
    if len(v1) != len(v2):
        raise ValueError(f"Dimension mismatch: {len(v1)} vs {len(v2)}")

    dot = sum(a * b for a, b in zip(v1, v2))
    norm1 = math.sqrt(sum(a * a for a in v1))
    norm2 = math.sqrt(sum(b * b for b in v2))

    if norm1 == 0.0 or norm2 == 0.0:
        return 0.0
    return dot / (norm1 * norm2)


def check_deduplication(v1: list[float], v2: list[float], threshold: float = 0.92) -> tuple[bool, float]:
    sim = cosine_similarity(v1, v2)
    return sim >= threshold, sim


def run_self_tests():
    # Identical vectors
    v1 = [1.0, 0.0, 0.0]
    v2 = [1.0, 0.0, 0.0]
    is_dupe, sim = check_deduplication(v1, v2, threshold=0.92)
    assert is_dupe is True and math.isclose(sim, 1.0)

    # Orthogonal vectors
    v3 = [0.0, 1.0, 0.0]
    is_dupe2, sim2 = check_deduplication(v1, v3, threshold=0.92)
    assert is_dupe2 is False and math.isclose(sim2, 0.0)

    # High similarity above threshold
    v4 = [1.0, 0.05, 0.0]
    is_dupe3, sim3 = check_deduplication(v1, v4, threshold=0.92)
    assert is_dupe3 is True and sim3 > 0.95

    print("[PASS] vector_memory_validator self-tests passed successfully.")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Vector Memory & Similarity Validator")
    parser.add_argument("--test", action="store_true", help="Run self-tests")

    args = parser.parse_args()
    if args.test:
        sys.exit(run_self_tests())

    print("[vector_memory_validator] Ready. Use --test to verify similarity logic.")


if __name__ == "__main__":
    main()
