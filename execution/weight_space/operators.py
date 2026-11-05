"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / weight_space
- **Primary Entrypoints**: `project_out_direction`, `difference_in_means_direction`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.
- `[Structural]` Operates via standard library Python with optional NumPy acceleration.
- `[Structural]` Input tensors/matrices are NEVER mutated in place; always returns a clean copy.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import math
from typing import List, Sequence, Union

try:
    import numpy as np  # type: ignore
    HAS_NUMPY = True
except ImportError:
    HAS_NUMPY = False


def _norm(vec: Sequence[float]) -> float:
    """Calculates L2 Euclidean norm of a 1D vector."""
    return math.sqrt(sum(x * x for x in vec))


def _dot(a: Sequence[float], b: Sequence[float]) -> float:
    """Calculates dot product between two vectors."""
    return sum(x * y for x, y in zip(a, b))


def difference_in_means_direction(
    a_pos: Sequence[Sequence[float]],
    a_neg: Sequence[Sequence[float]],
) -> List[float]:
    """Calculates unit direction separating two activation populations.

    a_pos, a_neg: list of activation vectors, each of length d_model.
    Returns unit direction vector of length d_model.
    """
    if not a_pos or not a_neg:
        raise ValueError("Activation populations cannot be empty")

    d_model = len(a_pos[0])
    n_pos = len(a_pos)
    n_neg = len(a_neg)

    # Compute mean vector for positive class
    mean_pos = [0.0] * d_model
    for vec in a_pos:
        for i in range(d_model):
            mean_pos[i] += vec[i]
    mean_pos = [x / n_pos for x in mean_pos]

    # Compute mean vector for negative class
    mean_neg = [0.0] * d_model
    for vec in a_neg:
        for i in range(d_model):
            mean_neg[i] += vec[i]
    mean_neg = [x / n_neg for x in mean_neg]

    # Direction = mean_pos - mean_neg
    diff = [p - n for p, n in zip(mean_pos, mean_neg)]
    norm = _norm(diff)
    if norm < 1e-12:
        return [0.0] * d_model

    return [x / norm for x in diff]


def project_out_direction(
    W: Sequence[Sequence[float]],
    direction: Sequence[float],
    col_idx: Sequence[int],
) -> List[List[float]]:
    """Orthogonalizes selected output columns of W against `direction`.

    W         : [d_model, n_cols] matrix
    direction : [d_model] vector
    col_idx   : iterable of column indices to modify

    Returns a deep copy of W with the directional component removed from specified columns.
    The input matrix is never mutated.
    """
    d_norm = _norm(direction)
    if d_norm < 1e-12:
        return [list(row) for row in W]

    r = [x / d_norm for x in direction]
    d_model = len(W)
    n_cols = len(W[0]) if d_model > 0 else 0
    target_cols = set(col_idx)

    # Deep copy W
    out = [list(row) for row in W]

    for c in target_cols:
        if c < 0 or c >= n_cols:
            continue

        # Extract column c: length d_model
        col_c = [out[r_idx][c] for r_idx in range(d_model)]

        # coeff = r @ col_c
        coeff = _dot(r, col_c)

        # col_c = col_c - coeff * r
        for r_idx in range(d_model):
            out[r_idx][c] -= coeff * r[r_idx]

    return out


def run_tests():
    print("[TEST] Running weight_space/operators.py self-test suite...")

    # 1. Test difference in means direction
    pos = [[1.0, 2.0, 3.0], [1.2, 2.2, 3.2]]
    neg = [[0.0, 0.0, 0.0], [0.2, 0.2, 0.2]]
    direction = difference_in_means_direction(pos, neg)
    assert len(direction) == 3, "Direction length mismatch"
    assert abs(_norm(direction) - 1.0) < 1e-6, "Direction must be unit norm"

    # 2. Test column-scoped projection
    # 3x3 matrix: W has 3 rows (d_model=3), 3 columns
    W = [
        [1.0, 4.0, 7.0],
        [2.0, 5.0, 8.0],
        [3.0, 6.0, 9.0],
    ]
    # Target col 1 only (middle column)
    W_patched = project_out_direction(W, direction, [1])

    # Invariant: Unselected columns 0 and 2 must remain identical
    assert [W_patched[r][0] for r in range(3)] == [W[r][0] for r in range(3)], "Column 0 was mutated!"
    assert [W_patched[r][2] for r in range(3)] == [W[r][2] for r in range(3)], "Column 2 was mutated!"

    # Invariant: Selected column 1 must now be mathematically orthogonal to direction
    col_1_patched = [W_patched[r][1] for r in range(3)]
    dot_prod = _dot(direction, col_1_patched)
    assert abs(dot_prod) < 1e-6, f"Patched column 1 is not orthogonal to direction (dot={dot_prod})"

    # Invariant: Original W was not modified
    assert W[0][1] == 4.0, "Original matrix W was mutated in place!"

    print("[TEST] All operators.py mathematical invariant proofs passed successfully!")


if __name__ == "__main__":
    import sys
    if "--test" in sys.argv:
        run_tests()
    else:
        print("Usage: python operators.py --test")
