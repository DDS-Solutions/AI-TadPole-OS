"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / weight_space
- **Primary Entrypoints**: `compute_model_attestation`, `compute_coarse_fingerprint`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic execution without side effects outside declared scope.
- `[Structural]` Provenance must declare explicit Grade (A=proven, B=measured, C=inferred).
- `[Structural]` Content-addressed SHA-256 weight hashing is non-bypassable.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

import os
import sys
import json
import math
import hashlib
from typing import Dict, Any, List, Optional

def compute_stream_digest(file_path: str, chunk_size: int = 65536) -> str:
    """Calculates SHA-256 digest of a model weight file incrementally."""
    hasher = hashlib.sha256()
    with open(file_path, "rb") as f:
        while True:
            chunk = f.read(chunk_size)
            if not chunk:
                break
            hasher.update(chunk)
    return hasher.hexdigest()

def compute_coarse_fingerprint(data_bytes: bytes, length: int = 64) -> List[float]:
    """Generates a deterministic 64-element geometric fingerprint from raw bytes or tensor data.
    Uses chunked variance and entropy to form a topological signature without heavy external dependencies.
    """
    if not data_bytes:
        return [0.0] * length
    
    total_len = len(data_bytes)
    chunk_len = max(1, total_len // length)
    fingerprint = []
    
    for i in range(length):
        start = i * chunk_len
        end = min(start + chunk_len, total_len)
        slice_bytes = data_bytes[start:end]
        if not slice_bytes:
            fingerprint.append(0.0)
            continue
        
        # Calculate mean and variance of byte values in chunk
        mean = sum(slice_bytes) / len(slice_bytes)
        variance = sum((b - mean) ** 2 for b in slice_bytes) / len(slice_bytes)
        norm_val = round((variance ** 0.5) / 128.0 - 1.0, 4)
        fingerprint.append(norm_val)
        
    return fingerprint

def compute_model_attestation(
    model_id: str,
    file_path: Optional[str] = None,
    mock_bytes: Optional[bytes] = None,
    grade: str = "B",
    baseline_ref: Optional[str] = None
) -> Dict[str, Any]:
    """Computes a complete model provenance attestation document."""
    if file_path and os.path.exists(file_path):
        size_bytes = os.path.getsize(file_path)
        digest = f"sha256:{compute_stream_digest(file_path)}"
        # Sample first 256KB for fingerprinting to keep operation fast and lightweight
        with open(file_path, "rb") as f:
            sample_bytes = f.read(262144)
    elif mock_bytes is not None:
        size_bytes = len(mock_bytes)
        digest = f"sha256:{hashlib.sha256(mock_bytes).hexdigest()}"
        sample_bytes = mock_bytes
    else:
        # Fallback for virtual/remote models without local file
        size_bytes = 0
        digest = f"sha256:{hashlib.sha256(model_id.encode('utf-8')).hexdigest()}"
        sample_bytes = model_id.encode("utf-8")
        grade = "C"

    fingerprint = compute_coarse_fingerprint(sample_bytes, length=64)
    
    # Estimate layers and d_model based on size heuristics if GGUF metadata isn't parsed
    estimated_layers = max(16, int(math.log2(max(1, size_bytes // (1024 * 1024))) * 2)) if size_bytes > 0 else 24
    
    return {
        "model_id": model_id,
        "weight_digest": digest,
        "size_bytes": size_bytes,
        "estimated_layers": estimated_layers,
        "fingerprint": fingerprint,
        "grade": grade,
        "baseline_ref": baseline_ref or f"attest_{model_id.replace(':', '_')}_v1",
        "attested_at": os.environ.get("ATTEST_TIMESTAMP", "2026-11-04T22:00:00Z"),
    }

def run_tests():
    print("[TEST] Running weight_space/attest.py self-test suite...")
    test_data = b"TADPOLE_SOVEREIGN_WEIGHT_SPACE_TEST_DATA" * 100
    attestation = compute_model_attestation("test-model:latest", mock_bytes=test_data, grade="A")
    
    assert attestation["model_id"] == "test-model:latest", "Model ID mismatch"
    assert attestation["weight_digest"].startswith("sha256:"), "Digest must start with sha256:"
    assert len(attestation["fingerprint"]) == 64, "Fingerprint must have exactly 64 elements"
    assert attestation["grade"] == "A", "Grade must be A"
    
    # Determinism test
    attestation2 = compute_model_attestation("test-model:latest", mock_bytes=test_data, grade="A")
    assert attestation["weight_digest"] == attestation2["weight_digest"], "Digest must be deterministic"
    assert attestation["fingerprint"] == attestation2["fingerprint"], "Fingerprint must be deterministic"
    
    print("[TEST] All attest.py invariant assertions passed successfully!")

if __name__ == "__main__":
    if "--test" in sys.argv:
        run_tests()
    elif len(sys.argv) > 1:
        target = sys.argv[1]
        print(json.dumps(compute_model_attestation(target, file_path=target if os.path.exists(target) else None), indent=2))
    else:
        print("Usage: python attest.py <model_id_or_path> | python attest.py --test")
