"""
@docs ARCHITECTURE:Core:Execution

### AI Context Alignment
- **Subsystem**: Test Verification Suite / test_pollywog_debt_ledger
- **Primary Entrypoints**: `PollywogDebtLedgerTests`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic internal state integrity and strict interface contract compliance.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from execution.pollywog_debt_ledger import scan_files


class PollywogDebtLedgerTests(unittest.TestCase):
    def test_scans_source_and_ignores_compiled_bytecode(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "module.py"
            marker = "# " + "polly" + "wog: bounded cache, revisit above 100k entries\n"
            source.write_text(
                marker,
                encoding="utf-8",
            )
            cache_dir = root / "__pycache__"
            cache_dir.mkdir()
            (cache_dir / "module.cpython-314.pyc").write_bytes(
                b"\x00\x01# " + b"polly" + b"wog: binary garbage without trigger"
            )

            findings = scan_files(root)

            self.assertEqual(len(findings), 1)
            self.assertEqual(findings[0]["file"], "module.py")


if __name__ == "__main__":
    unittest.main()
